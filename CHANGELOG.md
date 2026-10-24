# Changelog

## 0.3.0 2026-10-24

- **Breaking**: `GqlError::Status` holds its status boxed (`status: Box<proto::GqlStatus>`), which keeps `GqlError` and every `Result` carrying it small (clippy `result_large_err` on Rust 1.98). Code that builds the variant wraps the status in `Box::new(...)`; code that matches it and reads fields is unchanged through auto-deref
- **Breaking**: The GQLSTATUS constants follow ISO/IEC 39075 Table 8. Codes that ISO gives to another condition move: `NEGATIVE_LIMIT` is `22G02` (was `22G05`) and `LIST_DATA_RIGHT_TRUNCATION` is `22G0B` (was `22G09`). Conditions ISO does not define keep their names and move to implementation-defined subclasses (ISO 23.1): `22I01` to `22I09` (`INVALID_ELEMENT_ID` to `INCOMPATIBLE_RECORD_FIELD_TYPES`, were `22G06` to `22G0F`), `NO_ACTIVE_TRANSACTION` `25I01` (was `25G02`), `TRANSACTION_FAILED_STATE` `25I02` (was `25G04`), `42I01` to `42I12` (`DUPLICATE_DEFINITION` to `PROCEDURE_NOT_FOUND`, were `42004` to `42015`) and `G1I01` to `G1I03` (`GRAPH_DEPENDS_ON_SCHEMA` to `GRAPH_DEPENDS_ON_GRAPH_TYPE`, were `G1001` to `G1003`). The rest of Table 8 is new, for example `CATALOG_AND_DATA_STATEMENT_MIXING` (`25G02`), `MULTIPLE_GRAPHS_NOT_SUPPORTED` (`25G04`), `VISUALLY_CONFUSABLE_IDENTIFIERS` (`42004`), the graph type limits `42006` to `42015` and `EDGES_STILL_EXIST` (`G1001`). Python `gwp_py.status` and Java `GqlStatus` have the same constants; Go and JS keep their few constants, which already matched
- **Breaking**: The `OP_*` operation codes follow ISO/IEC 39075 Table 9: `OP_SESSION_RESET` is 7, `OP_SESSION_CLOSE` 8, `OP_START_TRANSACTION` 50, `OP_ROLLBACK` 51, `OP_COMMIT` 52, schema statements 100 and 101, graph statements 200 and 201, graph type statements 300 and 301, `OP_FILTER_STATEMENT` to `OP_RETURN_STATEMENT` 601 to 605. `OP_SESSION_SET_PARAMETER` is split into `OP_SESSION_SET_GRAPH_PARAMETER` (4), `OP_SESSION_SET_BINDING_TABLE_PARAMETER` (5) and `OP_SESSION_SET_VALUE_PARAMETER` (6); `OP_OPTIONAL_MATCH` is removed (ISO has no such operation); `OP_SELECT_STATEMENT` (606) and `OP_UNRECOGNIZED` (0) are new
- **Breaking**: A `GqlError::Status` sent as a gRPC error (from `SessionService`, `CatalogService`, `AdminService` or `SearchService`) gets its gRPC code from the GQLSTATUS class instead of always `INTERNAL`: `08` is `UNAVAILABLE`, `22` and `42` `INVALID_ARGUMENT`, `25`, `2D`, `G1` and `G2` `FAILED_PRECONDITION`, `40` `ABORTED`, any other class `INTERNAL`. The error also carries its code in a `gql-status` metadata entry (`error::GQL_STATUS_METADATA_KEY`), and the Rust client turns such an error back into `GqlError::Status` instead of `GqlError::Grpc`, so a backend that refuses a `Configure` or `Reset` with `25G01` reaches the caller as that status
- **Feature**: `GqlServer::server_info(name, version)` and `SessionServiceImpl::with_server_info()` set the server name and version the handshake reports; the default stays `gql-wire-protocol` with gwp's version
- **Breaking**: The Rust MSRV is 1.88 (was 1.85), so `tonic` can move to 0.14.6; CI gains a job that checks the crate on Rust 1.88. Code uses let chains (stable since 1.88) where clippy now asks for collapsed `if`s
- **Breaking**: The Python client requires Python 3.12 or newer (was 3.10; Python 3.10 reaches end of life in October 2026). Python 3.15 is supported: the package has classifiers for 3.12 to 3.15 and CI tests 3.12, 3.13, 3.14 and 3.15
- **Breaking**: The JS client requires Node 22 or newer (`engines.node` was `>=18`; Node 18 and 20 are end of life); CI tests Node 22 and 24
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
- **Dependencies**: Rust: minimums `tokio` 1.44.2 (RUSTSEC-2025-0023) and `tracing` 0.1.40 (RUSTSEC-2023-0078); lockfile refreshed (`tokio` 1.54.1, since 1.54.0 is yanked for RUSTSEC-2026-0384; `h2` 0.4.20, `rustls` 0.23.46, `rustls-webpki` 0.103.16 clear RUSTSEC-2026-0258 and the 2026 rustls and webpki advisories); `tonic`, `tonic-build`, `tonic-health`, `tonic-prost` and `tonic-prost-build` 0.14.6 (needs Rust 1.88, see the MSRV raise above). Go: `google.golang.org/grpc` 1.84.0 (fixes GO-2026-6061 and GO-2026-6348) and `golang.org/x/net` 0.58.0, which raises the `go` directive from 1.24.0 to 1.25.0. Python: minimums `grpcio>=1.78.0` and `protobuf>=6.31.1` (`grpcio-tools>=1.78.0` for development), the versions the generated stubs check on import; with older versions `import gwp_py` failed. JS and Java: no range or version changes
- **Infra**: GitHub Actions updated (`actions/checkout` v7, `setup-go`, `setup-node` and `setup-python` v7, `setup-java` v6, `upload-artifact` v7, `download-artifact` v8); Rust test assertions use `assert_eq!` / `assert_ne!` for the clippy `assert_is_empty` lint (Rust 1.99)

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
