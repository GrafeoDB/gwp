# GWP

A standalone, pure Rust gRPC wire protocol for [GQL (ISO/IEC 39075)](https://www.iso.org/standard/76120.html) - the international standard query language for property graphs.

Any GQL-compatible database engine can plug in via the `GqlBackend` trait. GWP handles gRPC transport, session management, transactions, and the full GQL type system over the wire.

## Features

- **Spec-faithful:** Full GQL type system, GQLSTATUS codes, session/transaction semantics
- **Pure Rust:** No C/C++ dependencies, `#![forbid(unsafe_code)]`
- **Lightweight:** Minimal deps: tonic, prost, tokio
- **Fast:** Streaming results via server-side gRPC streaming
- **Embeddable:** Library-first design, usable by any Rust project
- **TLS:** Optional TLS via `tls` feature flag (rustls)
- **Auth:** Pluggable authentication via `AuthValidator` trait
- **Health checks:** Standard `grpc.health.v1.Health` service
- **Observability:** Structured tracing on all gRPC methods via `tracing` crate
- **Graceful shutdown:** Drain connections on signal with `.shutdown()`

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
gwp = "0.1"
```

### Implementing a Backend

Implement the `GqlBackend` trait to connect your database:

```rust
use gwp::server::{GqlBackend, SessionHandle, TransactionHandle, SessionConfig};
use gwp::error::GqlError;

struct MyDatabase { /* ... */ }

#[tonic::async_trait]
impl GqlBackend for MyDatabase {
    async fn create_session(&self, config: &SessionConfig) -> Result<SessionHandle, GqlError> {
        // Create a session in your database
        Ok(SessionHandle("session-1".to_owned()))
    }

    async fn execute(
        &self,
        session: &SessionHandle,
        statement: &str,
        parameters: &std::collections::HashMap<String, gwp::types::Value>,
        transaction: Option<&TransactionHandle>,
    ) -> Result<std::pin::Pin<Box<dyn gwp::server::ResultStream>>, GqlError> {
        // Execute GQL and return a result stream
        todo!()
    }

    // ... other trait methods
}
```

### Starting the Server

```rust
use gwp::server::GqlServer;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = MyDatabase::new();
    let addr = "127.0.0.1:50051".parse()?;

    // Quick start with ctrl-c shutdown
    GqlServer::start(backend, addr).await?;

    Ok(())
}
```

Or use the builder for full control:

```rust
GqlServer::builder(backend)
    .idle_timeout(Duration::from_secs(300))
    .max_sessions(1000)
    .shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
    .serve(addr)
    .await?;
```

### Using the Client

```rust
use gwp::client::GqlConnection;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = GqlConnection::connect("http://127.0.0.1:50051").await?;
    let mut session = conn.create_session().await?;

    let mut cursor = session.execute("MATCH (n:Person) RETURN n.name", HashMap::new()).await?;

    while let Some(row) = cursor.next_row().await? {
        println!("{row:?}");
    }

    session.close().await?;
    Ok(())
}
```

## Architecture

```
Application (GQL statements, parameters, results)
       |
       v
  gRPC Services
  - SessionService:  handshake, configure, reset, close, ping
  - GqlService:      execute, begin_transaction, commit, rollback
  - DatabaseService: list, create, delete, get_info
  - HealthService:   grpc.health.v1.Health (check, watch)
       |
       v
  Protocol Buffers (prost)
  - Full GQL type system: Value, Node, Edge, Path, Record
  - GQLSTATUS codes for structured error reporting
       |
       v
  GqlBackend trait (your database plugs in here)
```

## GQL Type Support

| GQL Type | Wire Representation |
|----------|-------------------|
| `NULL`, `BOOLEAN`, `INTEGER`, `FLOAT`, `STRING`, `BYTES` | Native protobuf types |
| `DATE`, `TIME`, `DATETIME`, `DURATION` | Custom messages |
| `LIST`, `MAP` | Recursive `Value` containers |
| `NODE` | ID + labels + properties |
| `EDGE` | ID + type + endpoints + properties |
| `PATH` | Alternating nodes and edges |
| `BIGINTEGER`, `BIGFLOAT`, `DECIMAL` | String-encoded precision types |

## Modules

| Module | Description |
|--------|-------------|
| `proto` | Generated protobuf types and gRPC stubs |
| `types` | Ergonomic Rust wrappers over proto types |
| `server` | `GqlBackend` trait, session/transaction management, gRPC server |
| `client` | `GqlConnection`, `GqlSession`, `ResultCursor`, `Transaction`, `DatabaseClient` |
| `error` | `GqlError` enum |
| `status` | GQLSTATUS code constants and helpers |

## Client Bindings

| Language | Package | Install |
| ---------- | ------- | ------- |
| Python | [gwp-py](https://pypi.org/project/gwp-py/) | `pip install gwp-py` |
| JavaScript/TypeScript | [gwp-js](https://www.npmjs.com/package/gwp-js) | `npm install gwp-js` |
| Go | `github.com/GrafeoDB/gwp/go` | `go get github.com/GrafeoDB/gwp/go` |
| Java | `dev.grafeo:gwp` | Maven Central |

All bindings include `GqlConnection`, `GqlSession`, `Transaction`, `ResultCursor`, and `DatabaseClient`.

## Requirements

- Rust 1.85.0+ (edition 2024)
- `protoc` (Protocol Buffers compiler) - required at build time

## License

Licensed under either of [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0) or [MIT license](http://opensource.org/licenses/MIT) at your option.
