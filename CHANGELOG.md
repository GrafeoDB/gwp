# Changelog

## 0.3.0 Unreleased

- **Breaking**: `GqlError::Status` holds its status boxed (`status: Box<proto::GqlStatus>`), which keeps `GqlError` and every `Result` carrying it small (clippy `result_large_err` on Rust 1.98). Code that builds the variant wraps the status in `Box::new(...)`; code that matches it and reads fields is unchanged through auto-deref
- **Bug fix**: A failed `Commit` now ends the transaction (ISO/IEC 39075 sec 8.4) and asks the backend to roll it back. Before, the transaction stayed registered and every later `BeginTransaction` on the session failed with `25G01`. A failed `Rollback` also ends the transaction
- **Bug fix**: An `Execute` whose statement has a multi-byte character across byte 100 no longer panics the handler (the statement was cut at a fixed byte offset for the tracing span)
- **Bug fix**: `BeginTransaction` with an unknown `TransactionMode` is rejected with `INVALID_ARGUMENT` instead of starting a READ WRITE transaction
- **Bug fix**: A second `BeginTransaction` on a session that has a transaction is rejected with `25G01` before the backend is called (the backend used to start and then roll back a second transaction)
- **Bug fix**: Commit, rollback and statements on one transaction no longer race: a commit or rollback claims the transaction, so a concurrent commit, rollback or statement on it gets `25000` (`FAILED_PRECONDITION` for `Execute`) and a concurrent begin gets `25G01`
- **Bug fix**: `CloseSession` unregisters the session before cleaning up, so concurrent closes close the backend session once, and a transaction begun while its session closes is rolled back instead of staying registered
- **Bug fix**: The idle session reaper rolls back the active transaction before closing the backend session, like `CloseSession`; an idle timeout of zero, or one large enough to overflow the reaper's next tick (such as `Duration::MAX`), no longer panics the reaper task, which left idle sessions unreaped. The reaper now runs at least once an hour
- **Bug fix**: Nothing is streamed after the summary: after a summary, or a backend error mid-stream (sent as an error summary), the backend stream is dropped and the response ends
- **Bug fix**: `Reset` with `RESET_ALL` no longer drops the active transaction from the server's session state
- **Bug fix**: `status::class()` and the `is_*()` helpers no longer panic on a code whose first two bytes are not whole characters
- **Bug fix**: Dropping a client `Transaction` outside a tokio runtime no longer panics
- **Feature**: `types::Counters` and `ResultCursor::counters()`: typed write counters from `ResultSummary.counters`, whose key names are now documented in the proto
- **Feature**: Typed write counters in every binding, next to the raw map on the summary: Go `Counters`, `ResultCursor.Counters()` and `ResultSummary.WriteCounters()`; JS `Counters`, `cursor.counters()` and `summary.writeCounters`; Python `Counters`, `await cursor.counters()` and `summary.write_counters`; Java `Counters`, `cursor.counters()` and `summary.writeCounters()`
- **Feature**: `MockBackend` (and so `gwp-test-server`) reports write counters for `INSERT`, `DELETE` and `SET`, plus an `execution_time_ms` entry
- **Feature**: `GqlServer::serve_with_listener()` serves on an already bound `TcpListener`
- **Feature**: `GqlConnection::create_session_with_credentials()` for servers with an `AuthValidator`
- **Feature**: `GqlSession::set_parameter()` sets a named session parameter
- **Feature**: `TransactionManager::begin_termination()` and `has_transaction()`; `register()` rejects a transaction id already in use
- **Docs**: `GqlBackend` documents what the server does after a failed commit or rollback, and that a `ResultStream` is dropped when the client cancels or disconnects
- **Docs**: README documents write counters; Python installs with `uv add gwp-py`
- **Infra**: Go and JS stubs regenerated from the current proto (only the `ResultSummary.counters` comment changed; Python stubs carry no comments and are unchanged); `js/scripts/generate-proto.sh` no longer calls `npm bin`, which npm 9 removed
- **Bug fix**: Python `gwp_py.__version__` matches the package version
- **Tests**: Protocol tests over real sockets against a stateful backend: value round-trips for every GQL type and their boundaries, the catalog flow with `IF NOT EXISTS` / `IF EXISTS`, session properties, transaction failures and races, untrusted input, disconnects and backpressure

## 0.2.1 2026-04-11

- **Breaking**: Proto RPC `Close` renamed to `CloseSession` (`CloseRequest`/`CloseResponse` to `CloseSessionRequest`/`CloseSessionResponse`) to avoid `grpc-js` `Client.close()` conflict

## 0.2.0 2026-04-11

- **Breaking**: `AuthValidator::validate()` return type changed from `Result<(), GqlError>` to `Result<AuthInfo, GqlError>`
- **Feature**: `AuthInfo` struct with `principal: String` field, captured during handshake
- **Feature**: `SessionConfig` gains `auth_info: Option<AuthInfo>` field, passing validated identity through to `create_session`

## 0.1.6 2026-02-28

- **Breaking**: `DatabaseService` replaced by `CatalogService` (catalog > schema > graph hierarchy per GQL spec sec 12.2-12.7)
- **Breaking**: `DatabaseClient` replaced by `CatalogClient` with schema, graph, and graph type operations
- **Breaking**: Admin/Search request messages renamed `database` field to `graph`
- **Breaking**: `GqlBackend` trait: removed `list_databases`, `create_database`, `delete_database`, `get_database_info`; added `list_schemas`, `create_schema`, `drop_schema`, `list_graphs`, `create_graph`, `drop_graph`, `get_graph_info`, `list_graph_types`, `create_graph_type`, `drop_graph_type`
- **Feature**: `AdminClient` wrapper for stats, WAL, validation, and index operations
- **Feature**: `SearchClient` wrapper for vector, text, and hybrid search
- **Feature**: `Value` ergonomics: `TryFrom<Value>` for 11 types, `as_*()` accessors, `is_null()`, `type_name()`
- **Feature**: `From<f32>` for `Value` (lossless promotion to f64)
- **Feature**: `execute_simple()` convenience on `GqlSession` and `Transaction`
- **Feature**: `TypeDescriptor` extended with precision, scale, min/max length, max cardinality, group/open flags, duration qualifier, component types
- **Feature**: New `GqlType` variants: `TYPE_EMPTY`, `TYPE_YEAR_MONTH_DURATION`, `TYPE_DAY_TIME_DURATION`, `TYPE_NODE_REFERENCE`, `TYPE_EDGE_REFERENCE`, `TYPE_GRAPH_REFERENCE`, `TYPE_BINDING_TABLE_REFERENCE`
- **Feature**: `DurationQualifier` enum for year-to-month vs day-to-second duration distinction
- **Feature**: `ResultHeader.ordered` field for semantically meaningful row ordering
- **Feature**: `DiagnosticRecord` extended with `invalid_reference` field, `current_schema` now optional
- **Feature**: ~30 new GQLSTATUS code constants (warnings, informational, data exceptions, transaction state, syntax, dependent objects)
- **Feature**: `warning()` and `informational()` GQLSTATUS constructors
- **Feature**: 25+ operation code constants (Table 9 from GQL spec)

## 0.1.5 2026-02-19

- **Feature**: `AdminService` gRPC service (database stats, WAL status/checkpoint, integrity validation, index create/drop)
- **Feature**: `SearchService` gRPC service (vector similarity, full-text, hybrid search)
- **Feature**: Three index types: property (hash), vector (HNSW), full-text (BM25)
- **Feature**: `GqlBackend` trait extended with optional admin and search methods

## 0.1.4 2026-02-18

- **Feature**: Structured tracing via `tracing` crate on all gRPC methods
- **Feature**: Graceful shutdown with `.shutdown(signal)` builder method
- **Feature**: gRPC health check service (`grpc.health.v1.Health`)

## 0.1.3 2026-02-17

- **Feature**: `GqlServer` builder pattern with `.tls()`, `.auth()`, `.idle_timeout()`, `.max_sessions()`
- **Feature**: Optional TLS via `tls` feature flag (rustls)
- **Feature**: `AuthValidator` trait for pluggable handshake credential checks
- **Feature**: Idle session reaper with configurable timeout
- **Feature**: Configurable max concurrent sessions (`RESOURCE_EXHAUSTED` on limit)
- **Feature**: `GqlConnection::connect_tls()` on the client
- **Infra**: `publish.yml` workflow for crates.io (trusted publishing), npm, and Maven Central

## 0.1.2 2026-02-15

- **Breaking (proto)**: `ExecuteRequest.transaction_id` changed from `string` to `optional string`
- **Bug fix**: Extended numeric types (Decimal, BigInteger, BigFloat) no longer silently convert to Null
- **Perf**: ResultCursor uses VecDeque instead of Vec for O(1) row consumption
- **Ergonomics**: `Display` impl for `Value` type
- **Feature**: `DatabaseClient` wrapper in Rust client
- **Feature**: `DatabaseClient` wrapper in all 4 bindings (Python, JS, Go, Java)
- Regenerated proto stubs for all bindings
- **Infra**: GitHub Actions CI + PyPI trusted publishing + prek pre-commit hooks

## 0.1.1 2026-02-14

- Python binding (gwp-py) published to PyPI
- JavaScript/TypeScript binding (gwp-js) published to npm
- Go binding published to Go proxy
- Java binding (dev.grafeo:gwp) published to Maven Central
- DatabaseService added to proto and server

## 0.1.0 2026-02-12

- Foundation release
- Full GQL type system in protobuf (all ISO/IEC 39075 value types)
- SessionService and GqlService gRPC definitions
- Rust server with pluggable GqlBackend trait
- Rust client library (GqlConnection, GqlSession, Transaction, ResultCursor)
- MockBackend for testing
- GQLSTATUS code constants and helpers
