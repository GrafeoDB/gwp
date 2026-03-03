//! GWP: a standalone, pure Rust gRPC wire protocol for GQL (ISO/IEC 39075).
//!
//! This crate provides the protobuf type definitions, gRPC service
//! implementations, and client library for communicating GQL queries
//! and results over the wire.
//!
//! # Quick start (server)
//!
//! ```rust,no_run
//! use std::net::SocketAddr;
//! use gwp::server::{GqlServer, GqlBackend};
//!
//! # async fn example(backend: impl GqlBackend) -> Result<(), tonic::transport::Error> {
//! let addr: SocketAddr = "0.0.0.0:7687".parse().unwrap();
//!
//! GqlServer::builder(backend)
//!     .max_sessions(128)
//!     .shutdown(async { drop(tokio::signal::ctrl_c().await) })
//!     .serve(addr)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Quick start (client)
//!
//! ```rust,no_run
//! use gwp::client::GqlConnection;
//!
//! # async fn example() -> Result<(), gwp::error::GqlError> {
//! let mut conn = GqlConnection::connect("http://localhost:7687").await?;
//! let mut session = conn.create_session().await?;
//!
//! let mut cursor = session.execute_simple("MATCH (n:Person) RETURN n.name").await?;
//! let records = cursor.collect().await?;
//!
//! for record in &records {
//!     println!("{:?}", record);
//! }
//!
//! session.close().await?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]

pub mod client;
pub mod error;
pub mod proto;
pub mod server;
pub mod status;
pub mod types;
