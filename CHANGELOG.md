# Changelog

All notable changes to gwp, a pure Rust gRPC wire protocol for GQL (ISO/IEC 39075) with clients in Rust, Python, JavaScript, Go and Java.

## [0.4.0] - 2026-11-07

Sessions release: catalog, admin and search requests name the session they run in, a server with authentication requires one, and every binding can name its session on catalog calls. Breaking for code that wires the services by hand (the constructors take the `SessionManager`) or builds admin and search requests as struct literals, and for builds below Rust 1.99; servers built with `GqlServer` need no change. Against a server with authentication, the Rust client uses `session.catalog()`, `session.admin()` and `session.search()`; the Python, JavaScript, Go and Java clients send no handshake credentials yet, so they cannot authenticate.

### Added

- **Admin and search calls name their session**: every `AdminService` and `SearchService` request has a `session_id` (field 15, as on the catalog), and the Rust `GqlSession::admin()` and `GqlSession::search()` return clients that send it; `AdminClient::session_id()` and `SearchClient::session_id()` tell which. `GqlBackend` gains a context twin for each admin and search method (`get_graph_stats_in` to `hybrid_search_in`) that gets the session, and the defaults call the existing methods.
- **Session-bound catalog in every binding**: `session.catalog()` in Python, JavaScript and Java and `session.Catalog()` in Go return a catalog client whose requests name the session; `session_id` (Python), `sessionId` (JavaScript), `SessionID()` (Go) and `sessionId()` (Java) tell which. These clients send no handshake credentials yet, so they cannot open a session on a server with authentication.
- **`SessionManager::require_sessions()`** makes the catalog, admin and search services refuse requests without a session, for a server that authenticates callers another way; `sessions_required()` tells whether it is set.
- **`proto::FILE_DESCRIPTOR_SET`**: the encoded descriptors of gwp's protocol, for gRPC server reflection or tools that walk its services.

### Changed

Code and requests that worked with 0.3.2 and now fail.

- **Rust 1.99 is the minimum** (was 1.88), as for grafeo and grafeo-server; CI checks the crate on Rust 1.99.
- **Go 1.26 is the minimum for the Go module** (was 1.25), as the updated grpc and `golang.org/x` modules need; `go.mod` names `toolchain go1.26.10`, whose standard library has the current security fixes.
- **Service constructors take the session manager**: `CatalogServiceImpl::new(backend, sessions)`, `AdminServiceImpl::new(backend, sessions)` and `SearchServiceImpl::new(backend, sessions)`. Pass the `SessionManager` the `SessionServiceImpl` uses; `GqlServer` does this already.
- **Admin and search request messages** have the new `session_id` field: Rust code that builds them as struct literals needs it, for example with `..Default::default()`.
- **An admin or search call that names an unknown session** gets `NOT_FOUND`, as on the catalog and `GqlService`, before its arguments are checked; the session was ignored.
- **The Python, JavaScript and Go stubs are regenerated**: they now carry the catalog `session_id` from 0.3.2 too. Python `GqlSession.__init__` takes the channel as a fourth argument (`GqlSession.create()` is unchanged).

### Deprecated

- **`CatalogContext`**: use `RequestContext`, which the catalog, admin and search methods share; `CatalogContext` is an alias for it.

### Removed

- **`CatalogServiceImpl::with_sessions()`**: the session manager is a constructor argument.

### Security

- **With authentication, catalog, admin and search calls need a session**: on a server with an `AuthValidator` (`GqlServer::builder(...).auth(...)`), a `CatalogService`, `AdminService` or `SearchService` request that names no session is refused with `UNAUTHENTICATED`, and one naming an unknown session with `NOT_FOUND`, before the backend sees it.
- **pulldown-cmark 0.13.6** (from 0.13.4), a build dependency through `prost-build`: [RUSTSEC-2026-0414](https://rustsec.org/advisories/RUSTSEC-2026-0414.html) and [RUSTSEC-2026-0433](https://rustsec.org/advisories/RUSTSEC-2026-0433.html), exponential and quadratic parsing with `ENABLE_WIKILINKS`. gwp does not parse Markdown at run time.
- **Go: `golang.org/x/net` 0.62.0** (from 0.58.0): GO-2026-6617, GO-2026-6612, GO-2026-6611 and GO-2026-6603 (HTTP/2 crash, flow control, CPU and memory exhaustion), GO-2026-6610, GO-2026-6866 and GO-2026-6860.
- **JavaScript: `@grpc/grpc-js` `^1.14.7`** (was `^1.12.0`, which allowed affected versions): GHSA-5375-pq7m-f5r2 and GHSA-99f4-grh7-6pcq (a malformed message crashes a client or server), GHSA-m9gg-hp2v-232j (`getAuthContext` reports unauthorized certificates as authorized) and GHSA-f596-whhp-79r4.
- **Java: grpc-java 1.85.1** (from 1.68.1): its bundled netty 4.2.17 clears GHSA-prj3-ccx8-p6x4 (the MadeYouReset HTTP/2 denial of service) and 36 advisories against the netty 4.1.110 it bundled. Guava is pinned to 33.8.0-android, above the 33.6.0 grpc brings, for GHSA-xxph-c9ww-hj94.
- **Python: `protobuf>=6.33.5`** (was `>=6.31.1`): GHSA-7gcm-g887-7qv7, JSON parsing past the recursion depth limit.

### Dependencies

- **tonic 0.14.9** (requires `>=0.14.9`, with `tonic-health`, `tonic-prost` and `tonic-prost-build`): an HTTP/1 connection's header read timeout now applies.
- **Go: `google.golang.org/grpc` 1.85.0** (from 1.84.0), with `golang.org/x/sys` 0.49.0 and `golang.org/x/text` 0.43.0.
- **Java: protobuf-java and protoc 4.36.2** (from 4.28.3).
- **Python: `grpcio>=1.84.0` on Python 3.15**, the first release with wheels for it; other versions keep `grpcio>=1.78.0`.

## [0.3.2] - 2026-10-24

Catalog sessions release: a catalog request can name the session it runs in, so a server with one catalog per session shows that session's schemas and graphs. Only the Rust client sends the session.

### Added

- The `CatalogService` can work in the calling session's catalog (ISO/IEC 39075 sec 4.3.5: a session works in one GQL-catalog). Every catalog request has an optional `session_id` (field 15; empty means no session). The server checks that a named session exists (an unknown one gets `NOT_FOUND`, as on `GqlService`), counts the request as activity on it and passes it to the backend.
- `GqlBackend` gains a context twin for each catalog method (`list_schemas_in`, `create_schema_in`, `drop_schema_in`, `list_graphs_in`, `create_graph_in`, `drop_graph_in`, `get_graph_info_in`, `list_graph_types_in`, `create_graph_type_in`, `drop_graph_type_in`), each taking a `CatalogContext` with the calling session. The defaults ignore the context and call the existing methods, so existing backends keep working unchanged.
- `CatalogServiceImpl::with_sessions()` attaches the session manager (`GqlServer` does this); without one, a request that names a session is rejected with `NOT_FOUND`, since it cannot be checked.
- `GqlSession::catalog()` returns a `CatalogClient` whose requests name that session, and `CatalogClient::session_id()` tells which; `GqlConnection::create_catalog_client()` still names none. The Python, JS, Go and Java clients do not send the session yet.

### Changed

- Rust code that builds the ten catalog request messages as struct literals needs the new field, for example with `..Default::default()`.

## [0.3.1] - 2026-10-24

Republish release: the 0.3.0 code reached crates.io and PyPI but not npm or Maven Central; 0.3.1 publishes the same code everywhere. Nothing changes for Rust or Python users.

### Fixed

- npm publishes through trusted publishing (OIDC) instead of the `NPM_TOKEN` secret, which npm no longer accepts; the job runs on Node 24 with the latest npm (trusted publishing needs npm 11.5.1 or newer). `package.json` gains a `repository` field (`GrafeoDB/gwp`), which npm provenance checks.
- Java publishes with `central-publishing-maven-plugin` 0.12.0 (was 0.6.0); Maven Central rejected the 0.3.0 bundle with "Bundle has content that does NOT have a .pom file".

## [0.3.0] - 2026-10-24

Breaking release: GQLSTATUS constants and operation codes follow ISO/IEC 39075 Tables 8 and 9, gRPC errors carry a code from their GQLSTATUS class, `GqlError::Status` is boxed, and the minimum versions are Rust 1.88, Python 3.12 and Node 22. Also many transaction and session fixes, and typed write counters in every binding.

### Added

- `GqlServer::server_info(name, version)` and `SessionServiceImpl::with_server_info()` set the server name and version the handshake reports; the default stays `gql-wire-protocol` with gwp's version.
- `types::Counters` and `ResultCursor::counters()`: typed write counters from `ResultSummary.counters`, whose key names are now documented in the proto.
- Typed write counters in every binding, next to the raw map on the summary: Go `Counters`, `ResultCursor.Counters()` and `ResultSummary.WriteCounters()`; JS `Counters`, `cursor.counters()` and `summary.writeCounters`; Python `Counters`, `await cursor.counters()` and `summary.write_counters`; Java `Counters`, `cursor.counters()` and `summary.writeCounters()`.
- `MockBackend` (and so `gwp-test-server`) reports write counters for `INSERT`, `DELETE` and `SET`, plus an `execution_time_ms` entry.
- `GqlServer::serve_with_listener()` serves on an already bound `TcpListener`.
- `GqlConnection::create_session_with_credentials()` for servers with an `AuthValidator`.
- `GqlSession::set_parameter()` sets a named session parameter.
- `TransactionManager::begin_termination()` and `has_transaction()`; `register()` rejects a transaction id already in use.
- Protocol tests over real sockets against a stateful backend: value round-trips for every GQL type and their boundaries, the catalog flow with `IF NOT EXISTS` / `IF EXISTS`, session properties, transaction failures and races, untrusted input, disconnects and backpressure.

### Changed

- `GqlError::Status` holds its status boxed (`status: Box<proto::GqlStatus>`), which keeps `GqlError` and every `Result` carrying it small (clippy `result_large_err` on Rust 1.98). Code that builds the variant wraps the status in `Box::new(...)`; code that matches it and reads fields is unchanged through auto-deref.
- The GQLSTATUS constants follow ISO/IEC 39075 Table 8. Codes that ISO gives to another condition move: `NEGATIVE_LIMIT` is `22G02` (was `22G05`) and `LIST_DATA_RIGHT_TRUNCATION` is `22G0B` (was `22G09`). Conditions ISO does not define keep their names and move to implementation-defined subclasses (ISO 23.1): `22I01` to `22I09` (`INVALID_ELEMENT_ID` to `INCOMPATIBLE_RECORD_FIELD_TYPES`, were `22G06` to `22G0F`), `NO_ACTIVE_TRANSACTION` `25I01` (was `25G02`), `TRANSACTION_FAILED_STATE` `25I02` (was `25G04`), `42I01` to `42I12` (`DUPLICATE_DEFINITION` to `PROCEDURE_NOT_FOUND`, were `42004` to `42015`) and `G1I01` to `G1I03` (`GRAPH_DEPENDS_ON_SCHEMA` to `GRAPH_DEPENDS_ON_GRAPH_TYPE`, were `G1001` to `G1003`). The rest of Table 8 is new, for example `CATALOG_AND_DATA_STATEMENT_MIXING` (`25G02`), `MULTIPLE_GRAPHS_NOT_SUPPORTED` (`25G04`), `VISUALLY_CONFUSABLE_IDENTIFIERS` (`42004`), the graph type limits `42006` to `42015` and `EDGES_STILL_EXIST` (`G1001`). Python `gwp_py.status` and Java `GqlStatus` have the same constants; Go and JS keep their few constants, which already matched.
- The `OP_*` operation codes follow ISO/IEC 39075 Table 9: `OP_SESSION_RESET` is 7, `OP_SESSION_CLOSE` 8, `OP_START_TRANSACTION` 50, `OP_ROLLBACK` 51, `OP_COMMIT` 52, schema statements 100 and 101, graph statements 200 and 201, graph type statements 300 and 301, `OP_FILTER_STATEMENT` to `OP_RETURN_STATEMENT` 601 to 605. `OP_SESSION_SET_PARAMETER` is split into `OP_SESSION_SET_GRAPH_PARAMETER` (4), `OP_SESSION_SET_BINDING_TABLE_PARAMETER` (5) and `OP_SESSION_SET_VALUE_PARAMETER` (6); `OP_OPTIONAL_MATCH` is removed (ISO has no such operation); `OP_SELECT_STATEMENT` (606) and `OP_UNRECOGNIZED` (0) are new.
- A `GqlError::Status` sent as a gRPC error (from `SessionService`, `CatalogService`, `AdminService` or `SearchService`) gets its gRPC code from the GQLSTATUS class instead of always `INTERNAL`: `08` is `UNAVAILABLE`, `22` and `42` `INVALID_ARGUMENT`, `25`, `2D`, `G1` and `G2` `FAILED_PRECONDITION`, `40` `ABORTED`, any other class `INTERNAL`. The error also carries its code in a `gql-status` metadata entry (`error::GQL_STATUS_METADATA_KEY`), and the Rust client turns such an error back into `GqlError::Status` instead of `GqlError::Grpc`, so a backend that refuses a `Configure` or `Reset` with `25G01` reaches the caller as that status.
- The Rust MSRV is 1.88 (was 1.85), so `tonic` can move to 0.14.6; CI gains a job that checks the crate on Rust 1.88. Code uses let chains (stable since 1.88) where clippy now asks for collapsed `if`s.
- The Python client requires Python 3.12 or newer (was 3.10; Python 3.10 reaches end of life in October 2026). Python 3.15 is supported: the package has classifiers for 3.12 to 3.15 and CI tests 3.12, 3.13, 3.14 and 3.15.
- The JS client requires Node 22 or newer (`engines.node` was `>=18`; Node 18 and 20 are end of life); CI tests Node 22 and 24.
- `GqlBackend` documents what the server does after a failed commit or rollback, and that a `ResultStream` is dropped when the client cancels or disconnects.
- README documents write counters; Python installs with `uv add gwp-py`.
- Go and JS stubs regenerated from the current proto (only the `ResultSummary.counters` comment changed; Python stubs carry no comments and are unchanged); `js/scripts/generate-proto.sh` no longer calls `npm bin`, which npm 9 removed.
- GitHub Actions updated (`actions/checkout` v7, `setup-go`, `setup-node` and `setup-python` v7, `setup-java` v6, `upload-artifact` v7, `download-artifact` v8); Rust test assertions use `assert_eq!` / `assert_ne!` for the clippy `assert_is_empty` lint (Rust 1.99).

### Fixed

- A failed `Commit` now ends the transaction (ISO/IEC 39075 sec 8.4) and asks the backend to roll it back. Before, the transaction stayed registered and every later `BeginTransaction` on the session failed with `25G01`. A failed `Rollback` also ends the transaction.
- An `Execute` whose statement has a multi-byte character across byte 100 no longer panics the handler (the statement was cut at a fixed byte offset for the tracing span).
- `BeginTransaction` with an unknown `TransactionMode` is rejected with `INVALID_ARGUMENT` instead of starting a READ WRITE transaction.
- A second `BeginTransaction` on a session that has a transaction is rejected with `25G01` before the backend is called (the backend used to start and then roll back a second transaction).
- Commit, rollback and statements on one transaction no longer race: a commit or rollback claims the transaction, so a concurrent commit, rollback or statement on it gets `25000` (`FAILED_PRECONDITION` for `Execute`) and a concurrent begin gets `25G01`.
- `CloseSession` unregisters the session before cleaning up, so concurrent closes close the backend session once, and a transaction begun while its session closes is rolled back instead of staying registered.
- The idle session reaper rolls back the active transaction before closing the backend session, like `CloseSession`; an idle timeout of zero, or one large enough to overflow the reaper's next tick (such as `Duration::MAX`), no longer panics the reaper task, which left idle sessions unreaped. The reaper now runs at least once an hour.
- Nothing is streamed after the summary: after a summary, or a backend error mid-stream (sent as an error summary), the backend stream is dropped and the response ends.
- `Reset` with `RESET_ALL` no longer drops the active transaction from the server's session state.
- `status::class()` and the `is_*()` helpers no longer panic on a code whose first two bytes are not whole characters.
- Dropping a client `Transaction` outside a tokio runtime no longer panics.
- Python `gwp_py.__version__` matches the package version.

### Dependencies

- Rust: minimums `tokio` 1.44.2 (RUSTSEC-2025-0023) and `tracing` 0.1.40 (RUSTSEC-2023-0078); lockfile refreshed (`tokio` 1.54.1, since 1.54.0 is yanked for RUSTSEC-2026-0384; `h2` 0.4.20, `rustls` 0.23.46, `rustls-webpki` 0.103.16 clear RUSTSEC-2026-0258 and the 2026 rustls and webpki advisories); `tonic`, `tonic-build`, `tonic-health`, `tonic-prost` and `tonic-prost-build` 0.14.6 (needs Rust 1.88, see the MSRV raise above). Go: `google.golang.org/grpc` 1.84.0 (fixes GO-2026-6061 and GO-2026-6348) and `golang.org/x/net` 0.58.0, which raises the `go` directive from 1.24.0 to 1.25.0. Python: minimums `grpcio>=1.78.0` and `protobuf>=6.31.1` (`grpcio-tools>=1.78.0` for development), the versions the generated stubs check on import; with older versions `import gwp_py` failed. JS and Java: no range or version changes.

## [0.2.1] - 2026-04-11

Rename release: the `Close` RPC is `CloseSession`, so `grpc-js` clients no longer clash with `Client.close()`. Code that calls the RPC by its old name needs the new one.

### Changed

- Proto RPC `Close` renamed to `CloseSession` (`CloseRequest`/`CloseResponse` to `CloseSessionRequest`/`CloseSessionResponse`) to avoid `grpc-js` `Client.close()` conflict.

## [0.2.0] - 2026-04-11

Authentication release: the validator returns the caller's identity, and the backend gets it when a session is created. `AuthValidator` implementations need the new return type.

### Added

- `AuthInfo` struct with `principal: String` field, captured during handshake.
- `SessionConfig` gains `auth_info: Option<AuthInfo>` field, passing validated identity through to `create_session`.

### Changed

- `AuthValidator::validate()` return type changed from `Result<(), GqlError>` to `Result<AuthInfo, GqlError>`.

## [0.1.6] - 2026-02-28

Catalog release: the `CatalogService` (catalog, schema, graph and graph type, ISO/IEC 39075 sec 12) replaces the `DatabaseService`, and the type system, GQLSTATUS codes and operation codes grow toward the standard. Code using `DatabaseService`, `DatabaseClient` or the database backend methods needs the catalog ones.

### Added

- `AdminClient` wrapper for stats, WAL, validation, and index operations.
- `SearchClient` wrapper for vector, text, and hybrid search.
- `Value` ergonomics: `TryFrom<Value>` for 11 types, `as_*()` accessors, `is_null()`, `type_name()`.
- `From<f32>` for `Value` (lossless promotion to f64).
- `execute_simple()` convenience on `GqlSession` and `Transaction`.
- `TypeDescriptor` extended with precision, scale, min/max length, max cardinality, group/open flags, duration qualifier, component types.
- New `GqlType` variants: `TYPE_EMPTY`, `TYPE_YEAR_MONTH_DURATION`, `TYPE_DAY_TIME_DURATION`, `TYPE_NODE_REFERENCE`, `TYPE_EDGE_REFERENCE`, `TYPE_GRAPH_REFERENCE`, `TYPE_BINDING_TABLE_REFERENCE`.
- `DurationQualifier` enum for year-to-month vs day-to-second duration distinction.
- `ResultHeader.ordered` field for semantically meaningful row ordering.
- `DiagnosticRecord` extended with `invalid_reference` field, `current_schema` now optional.
- ~30 new GQLSTATUS code constants (warnings, informational, data exceptions, transaction state, syntax, dependent objects).
- `warning()` and `informational()` GQLSTATUS constructors.
- 25+ operation code constants (Table 9 from GQL spec).

### Changed

- `DatabaseService` replaced by `CatalogService` (catalog > schema > graph hierarchy per GQL spec sec 12.2-12.7).
- `DatabaseClient` replaced by `CatalogClient` with schema, graph, and graph type operations.
- Admin/Search request messages renamed `database` field to `graph`.
- `GqlBackend` trait: removed `list_databases`, `create_database`, `delete_database`, `get_database_info`; added `list_schemas`, `create_schema`, `drop_schema`, `list_graphs`, `create_graph`, `drop_graph`, `get_graph_info`, `list_graph_types`, `create_graph_type`, `drop_graph_type`.

## [0.1.5] - 2026-02-19

Admin and search release: optional `AdminService` and `SearchService` gRPC services for graph maintenance, indexes and vector, text and hybrid search.

### Added

- `AdminService` gRPC service (database stats, WAL status/checkpoint, integrity validation, index create/drop).
- `SearchService` gRPC service (vector similarity, full-text, hybrid search).
- Three index types: property (hash), vector (HNSW), full-text (BM25).
- `GqlBackend` trait extended with optional admin and search methods.

## [0.1.4] - 2026-02-18

Operations release: structured tracing, graceful shutdown and a gRPC health service.

### Added

- Structured tracing via `tracing` crate on all gRPC methods.
- Graceful shutdown with `.shutdown(signal)` builder method.
- gRPC health check service (`grpc.health.v1.Health`).

## [0.1.3] - 2026-02-17

Server release: the `GqlServer` builder with TLS, authentication, idle session timeouts and a session limit.

### Added

- `GqlServer` builder pattern with `.tls()`, `.auth()`, `.idle_timeout()`, `.max_sessions()`.
- Optional TLS via `tls` feature flag (rustls).
- `AuthValidator` trait for pluggable handshake credential checks.
- Idle session reaper with configurable timeout.
- Configurable max concurrent sessions (`RESOURCE_EXHAUSTED` on limit).
- `GqlConnection::connect_tls()` on the client.
- `publish.yml` workflow for crates.io (trusted publishing), npm, and Maven Central.

## [0.1.2] - 2026-02-15

Database client release: a `DatabaseClient` in every binding, extended numeric types that keep their value, and an optional transaction id on `ExecuteRequest`. Breaking for Rust code that sets `ExecuteRequest.transaction_id`, now an `Option<String>`.

### Added

- `Display` impl for `Value` type.
- `DatabaseClient` wrapper in Rust client.
- `DatabaseClient` wrapper in all 4 bindings (Python, JS, Go, Java).
- GitHub Actions CI + PyPI trusted publishing + prek pre-commit hooks.

### Changed

- `ExecuteRequest.transaction_id` changed from `string` to `optional string`.
- ResultCursor uses VecDeque instead of Vec for O(1) row consumption.
- Regenerated proto stubs for all bindings.

### Fixed

- Extended numeric types (Decimal, BigInteger, BigFloat) no longer silently convert to Null.

## [0.1.1] - 2026-02-14

Bindings release: the Python, JavaScript, Go and Java bindings are published.

### Added

- Python binding (gwp-py) published to PyPI.
- JavaScript/TypeScript binding (gwp-js) published to npm.
- Go binding published to Go proxy.
- Java binding (dev.grafeo:gwp) published to Maven Central.
- DatabaseService added to proto and server.

## [0.1.0] - 2026-02-12

Foundation release: the GQL type system in protobuf, the session and GQL services, a Rust server with a pluggable backend and a Rust client.

### Added

- Foundation release.
- Full GQL type system in protobuf (all ISO/IEC 39075 value types).
- SessionService and GqlService gRPC definitions.
- Rust server with pluggable GqlBackend trait.
- Rust client library (GqlConnection, GqlSession, Transaction, ResultCursor).
- MockBackend for testing.
- GQLSTATUS code constants and helpers.

---

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
