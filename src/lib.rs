//! A standalone, pure Rust gRPC wire protocol for GQL (ISO/IEC 39075).
//!
//! This crate provides the protobuf type definitions, gRPC service
//! implementations, and client library for communicating GQL queries
//! and results over the wire.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]

pub mod client;
pub mod error;
pub mod proto;
pub mod server;
pub mod status;
pub mod types;
