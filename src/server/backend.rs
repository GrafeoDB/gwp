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

// ============================================================================
// Catalog types
// ============================================================================

/// Summary information about a schema.
#[derive(Debug, Clone)]
pub struct SchemaInfo {
    /// Schema name.
    pub name: String,
    /// Number of graphs in the schema.
    pub graph_count: u32,
    /// Number of graph types in the schema.
    pub graph_type_count: u32,
}

/// Summary information about a graph.
#[derive(Debug, Clone)]
pub struct GraphInfo {
    /// Schema this graph belongs to.
    pub schema: String,
    /// Graph name.
    pub name: String,
    /// Number of nodes in the graph.
    pub node_count: u64,
    /// Number of edges in the graph.
    pub edge_count: u64,
    /// Graph type name (empty if open).
    pub graph_type: String,
    /// Storage mode (e.g. `InMemory`, `Persistent`).
    pub storage_mode: String,
    /// Memory limit in bytes, if configured.
    pub memory_limit_bytes: Option<u64>,
    /// Whether backward edges are maintained.
    pub backward_edges: Option<bool>,
    /// Number of worker threads.
    pub threads: Option<u32>,
}

/// Graph type specification for creating a graph.
#[derive(Debug, Clone)]
pub enum GraphTypeSpec {
    /// Open graph type (ANY GRAPH).
    Open,
    /// Reference to a named graph type.
    Named(String),
}

/// Configuration for creating a new graph.
#[derive(Debug, Clone)]
pub struct CreateGraphConfig {
    /// Schema to create the graph in.
    pub schema: String,
    /// Graph name.
    pub name: String,
    /// Whether to skip if the graph already exists.
    pub if_not_exists: bool,
    /// Whether to replace an existing graph.
    pub or_replace: bool,
    /// Graph type specification.
    pub type_spec: Option<GraphTypeSpec>,
    /// Optional source graph to copy from (`schema.graph` qualified name).
    pub copy_of: Option<String>,
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

/// Summary information about a graph type.
#[derive(Debug, Clone)]
pub struct GraphTypeInfo {
    /// Schema this graph type belongs to.
    pub schema: String,
    /// Graph type name.
    pub name: String,
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

    // =========================================================================
    // Catalog operations (optional - sec 12)
    // =========================================================================

    /// List all schemas.
    async fn list_schemas(&self) -> Result<Vec<SchemaInfo>, GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    /// Create a schema.
    async fn create_schema(
        &self,
        _name: &str,
        _if_not_exists: bool,
    ) -> Result<(), GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    /// Drop a schema. Returns whether it existed.
    async fn drop_schema(&self, _name: &str, _if_exists: bool) -> Result<bool, GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    /// List all graphs in a schema.
    async fn list_graphs(&self, _schema: &str) -> Result<Vec<GraphInfo>, GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    /// Create a graph. Returns the new graph info.
    async fn create_graph(&self, _config: CreateGraphConfig) -> Result<GraphInfo, GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    /// Drop a graph. Returns whether it existed.
    async fn drop_graph(
        &self,
        _schema: &str,
        _name: &str,
        _if_exists: bool,
    ) -> Result<bool, GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    /// Get detailed information about a graph.
    async fn get_graph_info(&self, _schema: &str, _name: &str) -> Result<GraphInfo, GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    /// List graph types in a schema.
    async fn list_graph_types(&self, _schema: &str) -> Result<Vec<GraphTypeInfo>, GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    /// Create a graph type.
    async fn create_graph_type(
        &self,
        _schema: &str,
        _name: &str,
        _if_not_exists: bool,
        _or_replace: bool,
    ) -> Result<(), GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    /// Drop a graph type. Returns whether it existed.
    async fn drop_graph_type(
        &self,
        _schema: &str,
        _name: &str,
        _if_exists: bool,
    ) -> Result<bool, GqlError> {
        Err(GqlError::Protocol("catalog not supported".into()))
    }

    // =========================================================================
    // Admin operations (optional)
    // =========================================================================

    /// Get detailed graph statistics.
    async fn get_graph_stats(&self, _graph: &str) -> Result<AdminStats, GqlError> {
        Err(GqlError::Protocol("admin not supported".into()))
    }

    /// Get WAL status for a graph.
    async fn wal_status(&self, _graph: &str) -> Result<AdminWalStatus, GqlError> {
        Err(GqlError::Protocol("admin not supported".into()))
    }

    /// Force a WAL checkpoint on a graph.
    async fn wal_checkpoint(&self, _graph: &str) -> Result<(), GqlError> {
        Err(GqlError::Protocol("admin not supported".into()))
    }

    /// Validate graph integrity.
    async fn validate(&self, _graph: &str) -> Result<AdminValidationResult, GqlError> {
        Err(GqlError::Protocol("admin not supported".into()))
    }

    /// Create an index on a graph.
    async fn create_index(&self, _graph: &str, _index: IndexDefinition) -> Result<(), GqlError> {
        Err(GqlError::Protocol("admin not supported".into()))
    }

    /// Drop an index from a graph.
    async fn drop_index(&self, _graph: &str, _index: IndexDefinition) -> Result<bool, GqlError> {
        Err(GqlError::Protocol("admin not supported".into()))
    }

    // =========================================================================
    // Search operations (optional)
    // =========================================================================

    /// Vector similarity search (KNN).
    async fn vector_search(&self, _req: VectorSearchParams) -> Result<Vec<SearchHit>, GqlError> {
        Err(GqlError::Protocol("search not supported".into()))
    }

    /// Full-text search (BM25).
    async fn text_search(&self, _req: TextSearchParams) -> Result<Vec<SearchHit>, GqlError> {
        Err(GqlError::Protocol("search not supported".into()))
    }

    /// Hybrid search (vector + text with rank fusion).
    async fn hybrid_search(&self, _req: HybridSearchParams) -> Result<Vec<SearchHit>, GqlError> {
        Err(GqlError::Protocol("search not supported".into()))
    }
}

// ============================================================================
// Admin types
// ============================================================================

/// Detailed graph statistics.
#[derive(Debug, Clone)]
pub struct AdminStats {
    /// Number of nodes.
    pub node_count: u64,
    /// Number of edges.
    pub edge_count: u64,
    /// Number of distinct labels.
    pub label_count: u64,
    /// Number of distinct edge types.
    pub edge_type_count: u64,
    /// Number of distinct property keys.
    pub property_key_count: u64,
    /// Number of indexes.
    pub index_count: u64,
    /// Memory usage in bytes.
    pub memory_bytes: u64,
    /// Disk usage in bytes (if persistent).
    pub disk_bytes: Option<u64>,
}

/// WAL status information.
#[derive(Debug, Clone)]
pub struct AdminWalStatus {
    /// Whether WAL is enabled.
    pub enabled: bool,
    /// WAL file path.
    pub path: Option<String>,
    /// WAL size in bytes.
    pub size_bytes: u64,
    /// Number of WAL records.
    pub record_count: u64,
    /// Last checkpoint timestamp.
    pub last_checkpoint: Option<u64>,
    /// Current epoch.
    pub current_epoch: u64,
}

/// Validation result.
#[derive(Debug, Clone)]
pub struct AdminValidationResult {
    /// Whether validation passed (no errors).
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<ValidationDiagnostic>,
    /// Validation warnings.
    pub warnings: Vec<ValidationDiagnostic>,
}

/// A single validation diagnostic (error or warning).
#[derive(Debug, Clone)]
pub struct ValidationDiagnostic {
    /// Diagnostic code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Optional context (e.g. the affected element or property).
    pub context: Option<String>,
}

/// Index definition for create/drop operations.
#[derive(Debug, Clone)]
pub enum IndexDefinition {
    /// Property hash index.
    Property {
        /// Property name.
        property: String,
    },
    /// Vector similarity index (HNSW).
    Vector {
        /// Node label.
        label: String,
        /// Property name.
        property: String,
        /// Expected dimensions.
        dimensions: Option<u32>,
        /// Distance metric.
        metric: Option<String>,
        /// HNSW links per node.
        m: Option<u32>,
        /// Construction beam width.
        ef_construction: Option<u32>,
    },
    /// Full-text index (BM25).
    Text {
        /// Node label.
        label: String,
        /// Property name.
        property: String,
    },
}

// ============================================================================
// Search types
// ============================================================================

/// Vector search parameters.
#[derive(Debug, Clone)]
pub struct VectorSearchParams {
    /// Graph name.
    pub graph: String,
    /// Node label.
    pub label: String,
    /// Property name.
    pub property: String,
    /// Query vector.
    pub query_vector: Vec<f32>,
    /// Number of results.
    pub k: u32,
    /// Search beam width.
    pub ef: Option<u32>,
    /// Property filters.
    pub filters: std::collections::HashMap<String, Value>,
}

/// Text search parameters.
#[derive(Debug, Clone)]
pub struct TextSearchParams {
    /// Graph name.
    pub graph: String,
    /// Node label.
    pub label: String,
    /// Property name.
    pub property: String,
    /// Search query text.
    pub query: String,
    /// Number of results.
    pub k: u32,
}

/// Hybrid search parameters.
#[derive(Debug, Clone)]
pub struct HybridSearchParams {
    /// Graph name.
    pub graph: String,
    /// Node label.
    pub label: String,
    /// Text property name.
    pub text_property: String,
    /// Vector property name.
    pub vector_property: String,
    /// Text query.
    pub query_text: String,
    /// Optional vector query.
    pub query_vector: Vec<f32>,
    /// Number of results.
    pub k: u32,
}

/// A single search result hit.
///
/// Search results use a numeric `node_id` (uint64) rather than the opaque
/// `bytes` element ID from the GQL type system. This is an internal
/// identifier suitable for fast lookups; it is not the same as `Node.id`.
#[derive(Debug, Clone)]
pub struct SearchHit {
    /// Internal numeric node identifier (not the opaque GQL element ID).
    pub node_id: u64,
    /// Relevance score (distance for vector, BM25 for text).
    pub score: f64,
    /// Node properties.
    pub properties: std::collections::HashMap<String, Value>,
}
