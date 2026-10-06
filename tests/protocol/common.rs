//! Shared test support: a stateful, recording backend and server helpers.
//!
//! `TestBackend` keeps a real (in-memory) catalog, tracks one transaction per
//! session, records every call it receives as a text event, and can be told
//! to fail or to block at chosen points. Statements are keywords:
//!
//! | Statement                          | Result                                        |
//! | ---------------------------------- | --------------------------------------------- |
//! | `ECHO`                             | one row: the parameters, sorted by name       |
//! | `ROWS <n>`                         | `n` integer rows in batches of 1000           |
//! | `WRITE`                            | a summary with write counters                 |
//! | `FAIL`                             | `execute` returns a GQLSTATUS error           |
//! | `FAIL_MIDSTREAM`                   | header, a row, an error, then more frames     |
//! | `TWO_SUMMARIES`                    | header, summary, then a row and a summary     |
//! | `STREAM_FOREVER`                   | header, then batches without end              |
//! | `CREATE GRAPH [IF NOT EXISTS] <g>` | creates `g` in schema `default`               |
//! | `DROP GRAPH [IF EXISTS] <g>`       | drops `g` from schema `default`               |
//! | anything else                      | omitted result                                |

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};
use std::time::Duration;

use tokio::sync::Notify;
use tonic::transport::Channel;

use gwp::error::GqlError;
use gwp::proto;
use gwp::proto::gql_service_client::GqlServiceClient;
use gwp::proto::session_service_client::SessionServiceClient;
use gwp::server::{
    AdminServiceImpl, CatalogServiceImpl, CreateGraphConfig, GqlBackend, GqlServiceImpl, GraphInfo,
    GraphTypeInfo, GraphTypeSpec, ResetTarget, ResultFrame, ResultStream, SchemaInfo,
    SearchServiceImpl, SessionConfig, SessionHandle, SessionManager, SessionProperty,
    SessionServiceImpl, TransactionHandle, TransactionManager,
};
use gwp::status;
use gwp::types::Value;

/// Name of the schema that always exists.
pub const DEFAULT_SCHEMA: &str = "default";

/// Lock a mutex, ignoring poisoning (a failed test must not hide others).
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A point where the backend waits until the test lets it continue.
#[derive(Clone, Default)]
pub struct Gate {
    entered: Arc<Notify>,
    release: Arc<Notify>,
}

impl Gate {
    /// Wait until the backend has reached the gate.
    pub async fn reached(&self) {
        tokio::time::timeout(Duration::from_secs(10), self.entered.notified())
            .await
            .expect("backend never reached the gate");
    }

    /// Let the backend continue.
    pub fn open(&self) {
        self.release.notify_one();
    }

    async fn pass(&self) {
        self.entered.notify_one();
        self.release.notified().await;
    }
}

#[derive(Default)]
struct State {
    sessions: BTreeSet<String>,
    events: Vec<String>,
    schemas: BTreeSet<String>,
    graphs: BTreeMap<(String, String), GraphInfo>,
    graph_types: BTreeSet<(String, String)>,
    /// Active transaction per session.
    transactions: HashMap<String, String>,
}

#[derive(Default)]
struct Inner {
    next_id: AtomicUsize,
    state: Mutex<State>,
    fail_next_commit: AtomicBool,
    fail_rollback: AtomicBool,
    begin_gate: Mutex<Option<Gate>>,
    commit_gate: Mutex<Option<Gate>>,
    frames_polled: Arc<AtomicUsize>,
    stream_dropped: Arc<AtomicBool>,
}

/// Stateful, recording test backend. Cheap to clone: clones share state.
#[derive(Clone, Default)]
pub struct TestBackend {
    inner: Arc<Inner>,
}

impl TestBackend {
    pub fn new() -> Self {
        let backend = Self::default();
        lock(&backend.inner.state)
            .schemas
            .insert(DEFAULT_SCHEMA.to_owned());
        backend
    }

    fn state(&self) -> MutexGuard<'_, State> {
        lock(&self.inner.state)
    }

    fn record(&self, event: String) {
        self.state().events.push(event);
    }

    fn next_id(&self, prefix: &str) -> String {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}-{id}")
    }

    /// All events recorded so far.
    pub fn events(&self) -> Vec<String> {
        self.state().events.clone()
    }

    /// Events that start with `prefix`.
    pub fn events_with(&self, prefix: &str) -> Vec<String> {
        self.events()
            .into_iter()
            .filter(|e| e.starts_with(prefix))
            .collect()
    }

    /// Whether the backend still has the session open.
    pub fn has_session(&self, session_id: &str) -> bool {
        self.state().sessions.contains(session_id)
    }

    /// The backend's active transaction for a session.
    pub fn transaction_of(&self, session_id: &str) -> Option<String> {
        self.state().transactions.get(session_id).cloned()
    }

    /// Whether a graph exists in the backend catalog.
    pub fn has_graph(&self, schema: &str, name: &str) -> bool {
        self.state()
            .graphs
            .contains_key(&(schema.to_owned(), name.to_owned()))
    }

    /// Make the next commit fail (the backend then cancels the transaction).
    pub fn fail_next_commit(&self) {
        self.inner.fail_next_commit.store(true, Ordering::SeqCst);
    }

    /// Make every rollback fail.
    pub fn fail_rollbacks(&self) {
        self.inner.fail_rollback.store(true, Ordering::SeqCst);
    }

    /// Make the next `begin_transaction` wait at a gate.
    pub fn gate_next_begin(&self) -> Gate {
        let gate = Gate::default();
        *lock(&self.inner.begin_gate) = Some(gate.clone());
        gate
    }

    /// Make the next `commit` wait at a gate.
    pub fn gate_next_commit(&self) -> Gate {
        let gate = Gate::default();
        *lock(&self.inner.commit_gate) = Some(gate.clone());
        gate
    }

    /// Number of frames the endless stream has produced.
    pub fn frames_polled(&self) -> usize {
        self.inner.frames_polled.load(Ordering::SeqCst)
    }

    /// Whether the endless stream has been dropped.
    pub fn stream_dropped(&self) -> bool {
        self.inner.stream_dropped.load(Ordering::SeqCst)
    }

    fn create_graph_in_catalog(&self, config: &CreateGraphConfig) -> Result<GraphInfo, GqlError> {
        let mut state = self.state();
        let schema = if config.schema.is_empty() {
            DEFAULT_SCHEMA.to_owned()
        } else {
            config.schema.clone()
        };
        if !state.schemas.contains(&schema) {
            return Err(GqlError::Session(format!("schema '{schema}' not found")));
        }
        let graph_type = match &config.type_spec {
            Some(GraphTypeSpec::Named(name)) => {
                if !state.graph_types.contains(&(schema.clone(), name.clone())) {
                    return Err(GqlError::Session(format!("graph type '{name}' not found")));
                }
                name.clone()
            }
            Some(GraphTypeSpec::Open) | None => String::new(),
        };
        let key = (schema.clone(), config.name.clone());
        if let Some(existing) = state.graphs.get(&key) {
            if config.if_not_exists {
                return Ok(existing.clone());
            }
            if !config.or_replace {
                return Err(GqlError::Session(format!(
                    "graph '{}' already exists",
                    config.name
                )));
            }
        }
        let info = GraphInfo {
            schema,
            name: config.name.clone(),
            node_count: 0,
            edge_count: 0,
            graph_type,
            storage_mode: config.storage_mode.clone(),
            memory_limit_bytes: config.memory_limit_bytes,
            backward_edges: config.backward_edges,
            threads: config.threads,
        };
        state.graphs.insert(key, info.clone());
        Ok(info)
    }

    fn execute_statement(
        &self,
        statement: &str,
        parameters: &HashMap<String, Value>,
    ) -> Result<Pin<Box<dyn ResultStream>>, GqlError> {
        let words: Vec<&str> = statement.split_whitespace().collect();
        let upper: Vec<String> = words.iter().map(|w| w.to_uppercase()).collect();
        let upper: Vec<&str> = upper.iter().map(String::as_str).collect();

        match upper.as_slice() {
            ["ECHO"] => Ok(echo_result(parameters)),
            ["ROWS", count] => count
                .parse()
                .map(rows_result)
                .map_err(|_| GqlError::status(status::INVALID_SYNTAX, "bad row count")),
            ["WRITE"] => {
                let counters = HashMap::from([
                    ("nodes_created".to_owned(), 2),
                    ("properties_set".to_owned(), 5),
                    ("labels_added".to_owned(), 2),
                    ("execution_time_ms".to_owned(), 7),
                ]);
                Ok(FrameStream::boxed(vec![
                    Ok(omitted_header()),
                    Ok(summary(status::success(), 0, counters)),
                ]))
            }
            ["FAIL"] => Err(GqlError::status(
                status::INVALID_SYNTAX,
                "requested failure",
            )),
            ["FAIL_MIDSTREAM"] => Ok(FrameStream::boxed(vec![
                Ok(header(vec![column("n")])),
                Ok(batch(vec![int_row(1)])),
                Err(GqlError::status(
                    status::DIVISION_BY_ZERO,
                    "division by zero",
                )),
                Ok(batch(vec![int_row(2)])),
                Ok(summary(status::success(), 2, HashMap::new())),
            ])),
            ["TWO_SUMMARIES"] => Ok(FrameStream::boxed(vec![
                Ok(header(vec![column("n")])),
                Ok(summary(status::success(), 0, HashMap::new())),
                Ok(batch(vec![int_row(1)])),
                Ok(summary(status::success(), 1, HashMap::new())),
            ])),
            ["STREAM_FOREVER"] => Ok(Box::pin(EndlessStream {
                polled: Arc::clone(&self.inner.frames_polled),
                dropped: Arc::clone(&self.inner.stream_dropped),
                header_sent: false,
            })),
            ["CREATE", "GRAPH", rest @ ..] => Ok(self.create_graph_statement(statement, rest)),
            ["DROP", "GRAPH", rest @ ..] => Ok(self.drop_graph_statement(statement, rest)),
            _ => Ok(omitted_result()),
        }
    }

    /// `CREATE GRAPH [IF NOT EXISTS] <name>`; `rest` follows `CREATE GRAPH`.
    fn create_graph_statement(&self, statement: &str, rest: &[&str]) -> Pin<Box<dyn ResultStream>> {
        // Names keep their case: take them from the original statement.
        let words: Vec<&str> = statement.split_whitespace().collect();
        let (if_not_exists, name) = match rest {
            ["IF", "NOT", "EXISTS", _] => (true, words[5]),
            [_] => (false, words[2]),
            _ => {
                return error_result(
                    status::INVALID_SYNTAX,
                    &format!("syntax error in '{statement}'"),
                );
            }
        };
        let config = CreateGraphConfig {
            schema: DEFAULT_SCHEMA.to_owned(),
            name: name.to_owned(),
            if_not_exists,
            or_replace: false,
            type_spec: None,
            copy_of: None,
            storage_mode: String::new(),
            memory_limit_bytes: None,
            backward_edges: None,
            threads: None,
            wal_enabled: None,
            wal_durability: None,
        };
        match self.create_graph_in_catalog(&config) {
            Ok(_) => omitted_result(),
            Err(err) => error_result(status::DUPLICATE_DEFINITION, &err.to_string()),
        }
    }

    /// `DROP GRAPH [IF EXISTS] <name>`; `rest` follows `DROP GRAPH`.
    fn drop_graph_statement(&self, statement: &str, rest: &[&str]) -> Pin<Box<dyn ResultStream>> {
        let words: Vec<&str> = statement.split_whitespace().collect();
        let (if_exists, name) = match rest {
            ["IF", "EXISTS", _] => (true, words[4]),
            [_] => (false, words[2]),
            _ => return error_result(status::INVALID_SYNTAX, "syntax error"),
        };
        let removed = self
            .state()
            .graphs
            .remove(&(DEFAULT_SCHEMA.to_owned(), name.to_owned()))
            .is_some();
        if removed || if_exists {
            omitted_result()
        } else {
            error_result(
                status::INVALID_REFERENCE,
                &format!("graph '{name}' not found"),
            )
        }
    }
}

#[tonic::async_trait]
impl GqlBackend for TestBackend {
    async fn create_session(&self, config: &SessionConfig) -> Result<SessionHandle, GqlError> {
        let id = self.next_id("session");
        let principal = config
            .auth_info
            .as_ref()
            .map_or("-", |info| info.principal.as_str());
        self.record(format!("create {id} principal={principal}"));
        self.state().sessions.insert(id.clone());
        Ok(SessionHandle(id))
    }

    async fn close_session(&self, session: &SessionHandle) -> Result<(), GqlError> {
        self.record(format!("close {}", session.0));
        let mut state = self.state();
        state.sessions.remove(&session.0);
        state.transactions.remove(&session.0);
        Ok(())
    }

    async fn configure_session(
        &self,
        session: &SessionHandle,
        property: SessionProperty,
    ) -> Result<(), GqlError> {
        if let SessionProperty::Graph(name) = &property {
            if !self.has_graph(DEFAULT_SCHEMA, name) && name != DEFAULT_SCHEMA {
                return Err(GqlError::Session(format!("graph '{name}' not found")));
            }
        }
        self.record(format!("configure {} {property:?}", session.0));
        Ok(())
    }

    async fn reset_session(
        &self,
        session: &SessionHandle,
        target: ResetTarget,
    ) -> Result<(), GqlError> {
        self.record(format!("reset {} {target:?}", session.0));
        Ok(())
    }

    async fn execute(
        &self,
        session: &SessionHandle,
        statement: &str,
        parameters: &HashMap<String, Value>,
        transaction: Option<&TransactionHandle>,
    ) -> Result<Pin<Box<dyn ResultStream>>, GqlError> {
        let tx = transaction.map_or("-", |t| t.0.as_str());
        self.record(format!("execute {} tx={tx} {statement}", session.0));
        self.execute_statement(statement, parameters)
    }

    async fn begin_transaction(
        &self,
        session: &SessionHandle,
        mode: proto::TransactionMode,
    ) -> Result<TransactionHandle, GqlError> {
        let gate = lock(&self.inner.begin_gate).take();
        if let Some(gate) = gate {
            gate.pass().await;
        }
        let id = self.next_id("tx");
        self.record(format!("begin {} {mode:?} {id}", session.0));
        let mut state = self.state();
        if !state.sessions.contains(&session.0) {
            return Err(GqlError::Session(format!(
                "session '{}' not found",
                session.0
            )));
        }
        state.transactions.insert(session.0.clone(), id.clone());
        Ok(TransactionHandle(id))
    }

    async fn commit(
        &self,
        session: &SessionHandle,
        transaction: &TransactionHandle,
    ) -> Result<(), GqlError> {
        let gate = lock(&self.inner.commit_gate).take();
        if let Some(gate) = gate {
            gate.pass().await;
        }
        self.record(format!("commit {} {}", session.0, transaction.0));
        if self.inner.fail_next_commit.swap(false, Ordering::SeqCst) {
            // Like a real engine: a failed commit cancels the transaction.
            self.state().transactions.remove(&session.0);
            return Err(GqlError::Transaction("write conflict".to_owned()));
        }
        match self.state().transactions.remove(&session.0) {
            Some(active) if active == transaction.0 => Ok(()),
            _ => Err(GqlError::Transaction("no active transaction".to_owned())),
        }
    }

    async fn rollback(
        &self,
        session: &SessionHandle,
        transaction: &TransactionHandle,
    ) -> Result<(), GqlError> {
        self.record(format!("rollback {} {}", session.0, transaction.0));
        let removed = self.state().transactions.remove(&session.0);
        if self.inner.fail_rollback.load(Ordering::SeqCst) {
            return Err(GqlError::Transaction("rollback failed".to_owned()));
        }
        match removed {
            Some(active) if active == transaction.0 => Ok(()),
            _ => Err(GqlError::Transaction("no active transaction".to_owned())),
        }
    }

    // Catalog

    async fn list_schemas(&self) -> Result<Vec<SchemaInfo>, GqlError> {
        let state = self.state();
        Ok(state
            .schemas
            .iter()
            .map(|name| SchemaInfo {
                name: name.clone(),
                graph_count: u32::try_from(state.graphs.keys().filter(|(s, _)| s == name).count())
                    .unwrap_or(u32::MAX),
                graph_type_count: u32::try_from(
                    state.graph_types.iter().filter(|(s, _)| s == name).count(),
                )
                .unwrap_or(u32::MAX),
            })
            .collect())
    }

    async fn create_schema(&self, name: &str, if_not_exists: bool) -> Result<(), GqlError> {
        let mut state = self.state();
        if state.schemas.contains(name) {
            return if if_not_exists {
                Ok(())
            } else {
                Err(GqlError::Session(format!("schema '{name}' already exists")))
            };
        }
        state.schemas.insert(name.to_owned());
        Ok(())
    }

    async fn drop_schema(&self, name: &str, if_exists: bool) -> Result<bool, GqlError> {
        let mut state = self.state();
        if !state.schemas.contains(name) {
            return if if_exists {
                Ok(false)
            } else {
                Err(GqlError::Session(format!("schema '{name}' not found")))
            };
        }
        if state.graphs.keys().any(|(s, _)| s == name) {
            return Err(GqlError::Session(format!(
                "schema '{name}' still contains graphs"
            )));
        }
        state.schemas.remove(name);
        Ok(true)
    }

    async fn list_graphs(&self, schema: &str) -> Result<Vec<GraphInfo>, GqlError> {
        let state = self.state();
        if !state.schemas.contains(schema) {
            return Err(GqlError::Session(format!("schema '{schema}' not found")));
        }
        Ok(state
            .graphs
            .iter()
            .filter(|((s, _), _)| s == schema)
            .map(|(_, info)| info.clone())
            .collect())
    }

    async fn create_graph(&self, config: CreateGraphConfig) -> Result<GraphInfo, GqlError> {
        self.record(format!(
            "create_graph {}.{} copy_of={:?}",
            config.schema, config.name, config.copy_of
        ));
        self.create_graph_in_catalog(&config)
    }

    async fn drop_graph(
        &self,
        schema: &str,
        name: &str,
        if_exists: bool,
    ) -> Result<bool, GqlError> {
        let removed = self
            .state()
            .graphs
            .remove(&(schema.to_owned(), name.to_owned()))
            .is_some();
        if removed || if_exists {
            Ok(removed)
        } else {
            Err(GqlError::Session(format!("graph '{name}' not found")))
        }
    }

    async fn get_graph_info(&self, schema: &str, name: &str) -> Result<GraphInfo, GqlError> {
        self.state()
            .graphs
            .get(&(schema.to_owned(), name.to_owned()))
            .cloned()
            .ok_or_else(|| GqlError::Session(format!("graph '{name}' not found")))
    }

    async fn list_graph_types(&self, schema: &str) -> Result<Vec<GraphTypeInfo>, GqlError> {
        Ok(self
            .state()
            .graph_types
            .iter()
            .filter(|(s, _)| s == schema)
            .map(|(s, n)| GraphTypeInfo {
                schema: s.clone(),
                name: n.clone(),
            })
            .collect())
    }

    async fn create_graph_type(
        &self,
        schema: &str,
        name: &str,
        if_not_exists: bool,
        or_replace: bool,
    ) -> Result<(), GqlError> {
        let mut state = self.state();
        let key = (schema.to_owned(), name.to_owned());
        if state.graph_types.contains(&key) && !if_not_exists && !or_replace {
            return Err(GqlError::Session(format!(
                "graph type '{name}' already exists"
            )));
        }
        state.graph_types.insert(key);
        Ok(())
    }

    async fn drop_graph_type(
        &self,
        schema: &str,
        name: &str,
        if_exists: bool,
    ) -> Result<bool, GqlError> {
        let removed = self
            .state()
            .graph_types
            .remove(&(schema.to_owned(), name.to_owned()));
        if removed || if_exists {
            Ok(removed)
        } else {
            Err(GqlError::Session(format!("graph type '{name}' not found")))
        }
    }
}

// ============================================================================
// Result streams
// ============================================================================

/// Yields a fixed list of frames (or errors) in order.
struct FrameStream {
    frames: VecDeque<Result<ResultFrame, GqlError>>,
}

impl FrameStream {
    fn boxed(frames: Vec<Result<ResultFrame, GqlError>>) -> Pin<Box<dyn ResultStream>> {
        Box::pin(Self {
            frames: frames.into(),
        })
    }
}

impl ResultStream for FrameStream {
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<ResultFrame, GqlError>>> {
        Poll::Ready(self.frames.pop_front())
    }
}

/// A header followed by an endless supply of 1 KiB rows. Counts every frame
/// it produces and records when it is dropped.
struct EndlessStream {
    polled: Arc<AtomicUsize>,
    dropped: Arc<AtomicBool>,
    header_sent: bool,
}

impl ResultStream for EndlessStream {
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<ResultFrame, GqlError>>> {
        let n = self.polled.fetch_add(1, Ordering::SeqCst);
        if !self.header_sent {
            self.header_sent = true;
            return Poll::Ready(Some(Ok(header(vec![column("payload")]))));
        }
        let payload = format!("{n:0>1024}");
        Poll::Ready(Some(Ok(batch(vec![proto::Row {
            values: vec![proto::Value::from(Value::String(payload))],
        }]))))
    }
}

impl Drop for EndlessStream {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

fn column(name: &str) -> proto::ColumnDescriptor {
    proto::ColumnDescriptor {
        name: name.to_owned(),
        r#type: None,
    }
}

fn header(columns: Vec<proto::ColumnDescriptor>) -> ResultFrame {
    ResultFrame::Header(proto::ResultHeader {
        result_type: proto::ResultType::BindingTable.into(),
        columns,
        ordered: true,
    })
}

fn omitted_header() -> ResultFrame {
    ResultFrame::Header(proto::ResultHeader {
        result_type: proto::ResultType::Omitted.into(),
        columns: Vec::new(),
        ordered: false,
    })
}

fn batch(rows: Vec<proto::Row>) -> ResultFrame {
    ResultFrame::Batch(proto::RowBatch { rows })
}

fn int_row(n: i64) -> proto::Row {
    proto::Row {
        values: vec![proto::Value::from(Value::Integer(n))],
    }
}

fn summary(
    status: proto::GqlStatus,
    rows_affected: i64,
    counters: HashMap<String, i64>,
) -> ResultFrame {
    ResultFrame::Summary(proto::ResultSummary {
        status: Some(status),
        warnings: Vec::new(),
        rows_affected,
        counters,
    })
}

/// One row holding the parameters, as columns sorted by name.
fn echo_result(parameters: &HashMap<String, Value>) -> Pin<Box<dyn ResultStream>> {
    let sorted: BTreeMap<&String, &Value> = parameters.iter().collect();
    let columns = sorted.keys().map(|name| column(name)).collect();
    let row = proto::Row {
        values: sorted
            .values()
            .map(|v| proto::Value::from((*v).clone()))
            .collect(),
    };
    FrameStream::boxed(vec![
        Ok(header(columns)),
        Ok(batch(vec![row])),
        Ok(summary(status::success(), 1, HashMap::new())),
    ])
}

/// `count` integer rows in batches of 1000.
fn rows_result(count: i64) -> Pin<Box<dyn ResultStream>> {
    let mut frames = vec![Ok(header(vec![column("n")]))];
    let mut next = 0;
    while next < count {
        let end = (next + 1000).min(count);
        frames.push(Ok(batch((next..end).map(int_row).collect())));
        next = end;
    }
    frames.push(Ok(summary(status::success(), count, HashMap::new())));
    FrameStream::boxed(frames)
}

/// A statement with an omitted result.
fn omitted_result() -> Pin<Box<dyn ResultStream>> {
    FrameStream::boxed(vec![
        Ok(omitted_header()),
        Ok(summary(status::omitted(), 0, HashMap::new())),
    ])
}

/// A statement that ran and failed: no header, an error summary.
fn error_result(code: &str, message: &str) -> Pin<Box<dyn ResultStream>> {
    FrameStream::boxed(vec![Ok(summary(
        status::error(code, message),
        0,
        HashMap::new(),
    ))])
}

// ============================================================================
// Servers and clients
// ============================================================================

/// A server wired by hand, so tests can inspect its session and
/// transaction managers.
pub struct TestServer {
    pub addr: SocketAddr,
    pub backend: TestBackend,
    pub sessions: SessionManager,
    pub transactions: TransactionManager,
}

impl TestServer {
    pub fn endpoint(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub async fn channel(&self) -> Channel {
        Channel::from_shared(self.endpoint())
            .unwrap()
            .connect()
            .await
            .unwrap()
    }

    pub async fn session_client(&self) -> SessionServiceClient<Channel> {
        SessionServiceClient::new(self.channel().await)
    }

    pub async fn gql_client(&self) -> GqlServiceClient<Channel> {
        GqlServiceClient::new(self.channel().await)
    }

    /// Handshake and return the session id.
    pub async fn handshake(&self) -> String {
        handshake(&mut self.session_client().await).await
    }
}

/// Start a server with every service on a fresh `TestBackend`.
pub async fn start() -> TestServer {
    start_with(TestBackend::new()).await
}

/// Start a server with every service on the given backend.
pub async fn start_with(backend: TestBackend) -> TestServer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let shared = Arc::new(backend.clone());
    let sessions = SessionManager::new();
    let transactions = TransactionManager::new();

    let session_service = SessionServiceImpl::new(
        Arc::clone(&shared),
        sessions.clone(),
        transactions.clone(),
        None,
    );
    let gql_service =
        GqlServiceImpl::new(Arc::clone(&shared), sessions.clone(), transactions.clone());
    let catalog_service = CatalogServiceImpl::new(Arc::clone(&shared));
    let admin_service = AdminServiceImpl::new(Arc::clone(&shared));
    let search_service = SearchServiceImpl::new(shared);

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(proto::session_service_server::SessionServiceServer::new(
                session_service,
            ))
            .add_service(proto::gql_service_server::GqlServiceServer::new(
                gql_service,
            ))
            .add_service(proto::catalog_service_server::CatalogServiceServer::new(
                catalog_service,
            ))
            .add_service(proto::admin_service_server::AdminServiceServer::new(
                admin_service,
            ))
            .add_service(proto::search_service_server::SearchServiceServer::new(
                search_service,
            ))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    TestServer {
        addr,
        backend,
        sessions,
        transactions,
    }
}

/// Handshake on a raw session client and return the session id.
pub async fn handshake(client: &mut SessionServiceClient<Channel>) -> String {
    client
        .handshake(proto::HandshakeRequest {
            protocol_version: 1,
            credentials: None,
            client_info: HashMap::new(),
        })
        .await
        .unwrap()
        .into_inner()
        .session_id
}

/// Execute on a raw client and collect every frame of the response.
pub async fn execute_frames(
    client: &mut GqlServiceClient<Channel>,
    session_id: &str,
    statement: &str,
    transaction_id: Option<&str>,
) -> Result<Vec<proto::execute_response::Frame>, tonic::Status> {
    let mut stream = client
        .execute(proto::ExecuteRequest {
            session_id: session_id.to_owned(),
            statement: statement.to_owned(),
            parameters: HashMap::new(),
            transaction_id: transaction_id.map(str::to_owned),
        })
        .await?
        .into_inner();
    let mut frames = Vec::new();
    while let Some(response) = stream.message().await? {
        frames.extend(response.frame);
    }
    Ok(frames)
}

/// The status of the summary frame, which must be the last frame.
pub fn final_status(frames: &[proto::execute_response::Frame]) -> proto::GqlStatus {
    match frames.last() {
        Some(proto::execute_response::Frame::Summary(s)) => s.status.clone().unwrap(),
        other => panic!("last frame is not a summary: {other:?}"),
    }
}

/// Begin a transaction on a raw client and return its id.
pub async fn begin(client: &mut GqlServiceClient<Channel>, session_id: &str) -> String {
    let response = client
        .begin_transaction(proto::BeginRequest {
            session_id: session_id.to_owned(),
            mode: proto::TransactionMode::ReadWrite.into(),
        })
        .await
        .unwrap()
        .into_inner();
    let code = response.status.unwrap().code;
    assert_eq!(code, status::SUCCESS, "begin failed");
    response.transaction_id
}

/// Commit on a raw client and return the GQLSTATUS code.
pub async fn commit(
    client: &mut GqlServiceClient<Channel>,
    session_id: &str,
    transaction_id: &str,
) -> String {
    client
        .commit(proto::CommitRequest {
            session_id: session_id.to_owned(),
            transaction_id: transaction_id.to_owned(),
        })
        .await
        .unwrap()
        .into_inner()
        .status
        .unwrap()
        .code
}

/// Roll back on a raw client and return the GQLSTATUS code.
pub async fn rollback(
    client: &mut GqlServiceClient<Channel>,
    session_id: &str,
    transaction_id: &str,
) -> String {
    client
        .rollback(proto::RollbackRequest {
            session_id: session_id.to_owned(),
            transaction_id: transaction_id.to_owned(),
        })
        .await
        .unwrap()
        .into_inner()
        .status
        .unwrap()
        .code
}

/// Poll `condition` until it holds, failing the test after 10 seconds.
pub async fn eventually(what: &str, mut condition: impl FnMut() -> bool) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while !condition() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for: {what}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
