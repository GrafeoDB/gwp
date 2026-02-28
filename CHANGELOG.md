# Changelog

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
