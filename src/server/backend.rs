//! The pluggable backend trait for GQL database engines.
//!
//! Any GQL-compatible database implements `GqlBackend` to plug into
//! the wire protocol server. The trait covers session lifecycle,
//! statement execution, and transaction management.

use std::collections::HashMap;
use std::pin::Pin;

use crate::error::GqlError;
use crate::proto;
use crate::types::Value;

/// Opaque session identifier issued at handshake.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionHandle(pub String);

/// Opaque transaction identifier issued at begin.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionHandle(pub String);

/// Configuration for a new session, derived from the handshake request.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Protocol version requested by the client.
    pub protocol_version: u32,
    /// Client metadata (driver name, version, platform).
    pub client_info: HashMap<String, String>,
}

/// A session property to configure.
#[derive(Debug, Clone)]
pub enum SessionProperty {
    /// Set the current schema.
    Schema(String),
    /// Set the current graph.
    Graph(String),
    /// Set the session timezone (UTC offset in minutes).
    TimeZone(i32),
    /// Set a named session parameter.
    Parameter {
        /// Parameter name.
        name: String,
        /// Parameter value.
        value: Value,
    },
}

/// What to reset on a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetTarget {
    /// Reset all session state to defaults.
    All,
    /// Reset current schema.
    Schema,
    /// Reset current graph.
    Graph,
    /// Reset timezone.
    TimeZone,
    /// Reset all parameters.
    Parameters,
}

/// A single frame in the result stream from executing a GQL statement.
#[derive(Debug, Clone)]
pub enum ResultFrame {
    /// Column metadata and result type. Always the first frame.
    Header(proto::ResultHeader),
    /// A batch of rows.
    Batch(proto::RowBatch),
    /// Completion status and statistics. Always the last frame.
    Summary(proto::ResultSummary),
}

/// Stream of result frames produced by statement execution.
///
/// Backends return a `ResultStream` from `execute()`. The server
/// converts each frame into a gRPC `ExecuteResponse` message.
pub trait ResultStream: Send + 'static {
    /// Get the next result frame.
    ///
    /// Returns `Ok(None)` when all frames have been delivered.
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<ResultFrame, GqlError>>>;
}

/// Configuration for creating a new database.
#[derive(Debug, Clone)]
pub struct CreateDatabaseConfig {
    /// Database name.
    pub name: String,
    /// Database type (e.g. "Lpg", "Rdf").
    pub database_type: String,
    /// Storage mode (e.g. `InMemory`, `Persistent`).
    pub storage_mode: String,
    /// Optional memory limit in bytes.
    pub memory_limit_bytes: Option<u64>,
    /// Whether to maintain backward edges.
    pub backward_edges: Option<bool>,
    /// Number of worker threads.
    pub threads: Option<u32>,
    /// Whether write-ahead logging is enabled.
    pub wal_enabled: Option<bool>,
    /// WAL durability mode.
    pub wal_durability: Option<String>,
}

/// Summary information about a database.
#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    /// Database name.
    pub name: String,
    /// Number of nodes in the database.
    pub node_count: u64,
    /// Number of edges in the database.
    pub edge_count: u64,
    /// Whether the database is persistent.
    pub persistent: bool,
    /// Database type (e.g. "Lpg", "Rdf").
    pub database_type: String,
    /// Storage mode (e.g. `InMemory`, `Persistent`).
    pub storage_mode: String,
    /// Memory limit in bytes, if configured.
    pub memory_limit_bytes: Option<u64>,
    /// Whether backward edges are maintained.
    pub backward_edges: Option<bool>,
    /// Number of worker threads.
    pub threads: Option<u32>,
}

/// The pluggable backend trait for GQL database engines.
///
/// Implement this trait to connect any GQL-compatible database to the
/// wire protocol server. The server handles gRPC transport, session
/// management, and protocol details - the backend focuses on executing
/// GQL statements and managing data.
#[tonic::async_trait]
pub trait GqlBackend: Send + Sync + 'static {
    /// Create a new session with the given configuration.
    ///
    /// Called during handshake. The backend should allocate any per-session
    /// resources and return a handle for subsequent calls.
    async fn create_session(&self, config: &SessionConfig) -> Result<SessionHandle, GqlError>;

    /// Close a session and release its resources.
    ///
    /// Called when the client explicitly closes the session or when
    /// the connection drops. Any active transaction should be rolled back.
    async fn close_session(&self, session: &SessionHandle) -> Result<(), GqlError>;

    /// Set a session property (schema, graph, timezone, or parameter).
    async fn configure_session(
        &self,
        session: &SessionHandle,
        property: SessionProperty,
    ) -> Result<(), GqlError>;

    /// Reset session state to defaults.
    async fn reset_session(
        &self,
        session: &SessionHandle,
        target: ResetTarget,
    ) -> Result<(), GqlError>;

    /// Execute a GQL statement and return a stream of result frames.
    ///
    /// The stream should emit frames in order: Header, then zero or more
    /// Batch frames, then Summary. The server converts these into
    /// streaming gRPC `ExecuteResponse` messages.
    async fn execute(
        &self,
        session: &SessionHandle,
        statement: &str,
        parameters: &HashMap<String, Value>,
        transaction: Option<&TransactionHandle>,
    ) -> Result<Pin<Box<dyn ResultStream>>, GqlError>;

    /// Begin an explicit transaction.
    ///
    /// Returns a transaction handle for use in subsequent `execute`,
    /// `commit`, and `rollback` calls.
    async fn begin_transaction(
        &self,
        session: &SessionHandle,
        mode: proto::TransactionMode,
    ) -> Result<TransactionHandle, GqlError>;

    /// Commit the transaction.
    async fn commit(
        &self,
        session: &SessionHandle,
        transaction: &TransactionHandle,
    ) -> Result<(), GqlError>;

    /// Roll back the transaction.
    async fn rollback(
        &self,
        session: &SessionHandle,
        transaction: &TransactionHandle,
    ) -> Result<(), GqlError>;

    /// List all databases.
    ///
    /// Default implementation returns `Unimplemented` for backends that
    /// don't support database management.
    async fn list_databases(&self) -> Result<Vec<DatabaseInfo>, GqlError> {
        Err(GqlError::Protocol(
            "database management not supported".into(),
        ))
    }

    /// Create a new database.
    ///
    /// Default implementation returns `Unimplemented` for backends that
    /// don't support database management.
    async fn create_database(
        &self,
        _config: CreateDatabaseConfig,
    ) -> Result<DatabaseInfo, GqlError> {
        Err(GqlError::Protocol(
            "database management not supported".into(),
        ))
    }

    /// Delete a database by name.
    ///
    /// Returns the name of the deleted database on success.
    /// Default implementation returns `Unimplemented` for backends that
    /// don't support database management.
    async fn delete_database(&self, _name: &str) -> Result<String, GqlError> {
        Err(GqlError::Protocol(
            "database management not supported".into(),
        ))
    }

    /// Get info for a specific database.
    ///
    /// Default implementation returns `Unimplemented` for backends that
    /// don't support database management.
    async fn get_database_info(&self, _name: &str) -> Result<DatabaseInfo, GqlError> {
        Err(GqlError::Protocol(
            "database management not supported".into(),
        ))
    }
}
