//! Mock backend for testing the wire protocol server.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

use crate::error::GqlError;
use crate::proto;
use crate::types::Value;

use super::backend::{
    CreateDatabaseConfig, DatabaseInfo, GqlBackend, ResetTarget, ResultFrame, ResultStream,
    SessionConfig, SessionHandle, SessionProperty, TransactionHandle,
};

/// A simple in-memory backend for testing.
///
/// Tracks sessions and transactions. For `execute()`, returns canned
/// results based on the statement text.
pub struct MockBackend {
    session_counter: AtomicU64,
    transaction_counter: AtomicU64,
}

impl MockBackend {
    /// Create a new mock backend.
    #[must_use]
    pub fn new() -> Self {
        Self {
            session_counter: AtomicU64::new(1),
            transaction_counter: AtomicU64::new(1),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl GqlBackend for MockBackend {
    async fn create_session(&self, _config: &SessionConfig) -> Result<SessionHandle, GqlError> {
        let id = self.session_counter.fetch_add(1, Ordering::Relaxed);
        Ok(SessionHandle(format!("mock-session-{id}")))
    }

    async fn close_session(&self, _session: &SessionHandle) -> Result<(), GqlError> {
        Ok(())
    }

    async fn configure_session(
        &self,
        _session: &SessionHandle,
        _property: SessionProperty,
    ) -> Result<(), GqlError> {
        Ok(())
    }

    async fn reset_session(
        &self,
        _session: &SessionHandle,
        _target: ResetTarget,
    ) -> Result<(), GqlError> {
        Ok(())
    }

    async fn execute(
        &self,
        _session: &SessionHandle,
        statement: &str,
        _parameters: &HashMap<String, Value>,
        _transaction: Option<&TransactionHandle>,
    ) -> Result<Pin<Box<dyn ResultStream>>, GqlError> {
        // Parse statement to determine response
        let trimmed = statement.trim().to_uppercase();

        if trimmed.starts_with("MATCH") || trimmed.starts_with("RETURN") {
            // Simulate a binding table result with some rows
            Ok(Box::pin(MockResultStream::binding_table()))
        } else if trimmed.starts_with("INSERT")
            || trimmed.starts_with("DELETE")
            || trimmed.starts_with("SET")
        {
            // Simulate a DML operation
            Ok(Box::pin(MockResultStream::dml(3)))
        } else if trimmed.starts_with("CREATE") || trimmed.starts_with("DROP") {
            // Simulate a DDL operation
            Ok(Box::pin(MockResultStream::ddl()))
        } else if trimmed.starts_with("ERROR") {
            // Simulate an error for testing
            Err(GqlError::status(
                crate::status::INVALID_SYNTAX,
                "mock syntax error",
            ))
        } else {
            Ok(Box::pin(MockResultStream::ddl()))
        }
    }

    async fn begin_transaction(
        &self,
        _session: &SessionHandle,
        _mode: proto::TransactionMode,
    ) -> Result<TransactionHandle, GqlError> {
        let id = self.transaction_counter.fetch_add(1, Ordering::Relaxed);
        Ok(TransactionHandle(format!("mock-tx-{id}")))
    }

    async fn commit(
        &self,
        _session: &SessionHandle,
        _transaction: &TransactionHandle,
    ) -> Result<(), GqlError> {
        Ok(())
    }

    async fn rollback(
        &self,
        _session: &SessionHandle,
        _transaction: &TransactionHandle,
    ) -> Result<(), GqlError> {
        Ok(())
    }

    async fn list_databases(&self) -> Result<Vec<DatabaseInfo>, GqlError> {
        Ok(vec![
            DatabaseInfo {
                name: "default".to_owned(),
                node_count: 100,
                edge_count: 50,
                persistent: false,
                database_type: "Lpg".to_owned(),
                storage_mode: "InMemory".to_owned(),
                memory_limit_bytes: None,
                backward_edges: Some(false),
                threads: None,
            },
            DatabaseInfo {
                name: "test".to_owned(),
                node_count: 10,
                edge_count: 5,
                persistent: false,
                database_type: "Lpg".to_owned(),
                storage_mode: "InMemory".to_owned(),
                memory_limit_bytes: None,
                backward_edges: None,
                threads: None,
            },
        ])
    }

    async fn create_database(
        &self,
        config: CreateDatabaseConfig,
    ) -> Result<DatabaseInfo, GqlError> {
        if config.name == "default" {
            return Err(GqlError::Session(
                "database 'default' already exists".to_owned(),
            ));
        }
        Ok(DatabaseInfo {
            name: config.name,
            node_count: 0,
            edge_count: 0,
            persistent: config.storage_mode == "Persistent",
            database_type: config.database_type,
            storage_mode: config.storage_mode,
            memory_limit_bytes: config.memory_limit_bytes,
            backward_edges: config.backward_edges,
            threads: config.threads,
        })
    }

    async fn delete_database(&self, name: &str) -> Result<String, GqlError> {
        if name == "default" {
            return Err(GqlError::Session(
                "cannot delete the default database".to_owned(),
            ));
        }
        Ok(name.to_owned())
    }

    async fn get_database_info(&self, name: &str) -> Result<DatabaseInfo, GqlError> {
        match name {
            "default" => Ok(DatabaseInfo {
                name: "default".to_owned(),
                node_count: 100,
                edge_count: 50,
                persistent: false,
                database_type: "Lpg".to_owned(),
                storage_mode: "InMemory".to_owned(),
                memory_limit_bytes: None,
                backward_edges: Some(false),
                threads: None,
            }),
            "test" => Ok(DatabaseInfo {
                name: "test".to_owned(),
                node_count: 10,
                edge_count: 5,
                persistent: false,
                database_type: "Lpg".to_owned(),
                storage_mode: "InMemory".to_owned(),
                memory_limit_bytes: None,
                backward_edges: None,
                threads: None,
            }),
            _ => Err(GqlError::Session(format!(
                "database '{name}' not found"
            ))),
        }
    }
}

/// Mock result stream that yields pre-configured frames.
struct MockResultStream {
    frames: Vec<ResultFrame>,
    index: usize,
}

impl MockResultStream {
    fn binding_table() -> Self {
        let header = ResultFrame::Header(proto::ResultHeader {
            result_type: proto::ResultType::BindingTable.into(),
            columns: vec![
                proto::ColumnDescriptor {
                    name: "name".to_owned(),
                    r#type: Some(proto::TypeDescriptor {
                        r#type: proto::GqlType::TypeString.into(),
                        nullable: false,
                        element_type: None,
                        fields: Vec::new(),
                    }),
                },
                proto::ColumnDescriptor {
                    name: "age".to_owned(),
                    r#type: Some(proto::TypeDescriptor {
                        r#type: proto::GqlType::TypeInt64.into(),
                        nullable: false,
                        element_type: None,
                        fields: Vec::new(),
                    }),
                },
            ],
        });

        let batch = ResultFrame::Batch(proto::RowBatch {
            rows: vec![
                proto::Row {
                    values: vec![
                        proto::Value::from(Value::from("Alice")),
                        proto::Value::from(Value::from(30_i64)),
                    ],
                },
                proto::Row {
                    values: vec![
                        proto::Value::from(Value::from("Bob")),
                        proto::Value::from(Value::from(25_i64)),
                    ],
                },
            ],
        });

        let summary = ResultFrame::Summary(proto::ResultSummary {
            status: Some(crate::status::success()),
            warnings: Vec::new(),
            rows_affected: 2,
            counters: HashMap::new(),
        });

        Self {
            frames: vec![header, batch, summary],
            index: 0,
        }
    }

    fn dml(rows_affected: i64) -> Self {
        let header = ResultFrame::Header(proto::ResultHeader {
            result_type: proto::ResultType::Omitted.into(),
            columns: Vec::new(),
        });

        let summary = ResultFrame::Summary(proto::ResultSummary {
            status: Some(crate::status::success()),
            warnings: Vec::new(),
            rows_affected,
            counters: HashMap::new(),
        });

        Self {
            frames: vec![header, summary],
            index: 0,
        }
    }

    fn ddl() -> Self {
        let header = ResultFrame::Header(proto::ResultHeader {
            result_type: proto::ResultType::Omitted.into(),
            columns: Vec::new(),
        });

        let summary = ResultFrame::Summary(proto::ResultSummary {
            status: Some(crate::status::omitted()),
            warnings: Vec::new(),
            rows_affected: 0,
            counters: HashMap::new(),
        });

        Self {
            frames: vec![header, summary],
            index: 0,
        }
    }
}

impl ResultStream for MockResultStream {
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<ResultFrame, GqlError>>> {
        if self.index < self.frames.len() {
            let frame = self.frames[self.index].clone();
            self.index += 1;
            Poll::Ready(Some(Ok(frame)))
        } else {
            Poll::Ready(None)
        }
    }
}
