# Changelog

## 0.1.2

- **Breaking (proto)**: `ExecuteRequest.transaction_id` changed from `string` to `optional string`
- **Bug fix**: Extended numeric types (Decimal, BigInteger, BigFloat) no longer silently convert to Null
- **Perf**: ResultCursor uses VecDeque instead of Vec for O(1) row consumption
- **Ergonomics**: `Display` impl for `Value` type
- **Feature**: `DatabaseClient` wrapper in Rust client
- **Feature**: `DatabaseClient` wrapper in all 4 bindings (Python, JS, Go, Java)
- Regenerated proto stubs for all bindings
- **Infra**: GitHub Actions CI + PyPI trusted publishing + prek pre-commit hooks

## 0.1.1

- Python binding (gwp-py) published to PyPI
- JavaScript/TypeScript binding (gwp-js) published to npm
- Go binding published to Go proxy
- Java binding (dev.grafeo:gwp) published to Maven Central
- DatabaseService added to proto and server

## 0.1.0

- Foundation release
- Full GQL type system in protobuf (all ISO/IEC 39075 value types)
- SessionService and GqlService gRPC definitions
- Rust server with pluggable GqlBackend trait
- Rust client library (GqlConnection, GqlSession, Transaction, ResultCursor)
- MockBackend for testing
- GQLSTATUS code constants and helpers
