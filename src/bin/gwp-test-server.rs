//! Standalone test server for GWP integration testing.
//!
//! Starts a gRPC server with `MockBackend` on the specified port.
//! Used by all language bindings for integration tests.
//!
//! Usage: `gwp-test-server [PORT]` (default: 50051)

use std::net::SocketAddr;

use gwp::server::mock_backend::MockBackend;
use gwp::server::GqlServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50051);

    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    let backend = MockBackend::new();

    eprintln!("GWP test server listening on {addr}");
    GqlServer::serve(backend, addr).await?;

    Ok(())
}
