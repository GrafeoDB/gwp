//! Ergonomic Rust client for the GQL wire protocol.
//!
//! Wraps the raw tonic gRPC stubs with a typed, session-oriented API.

mod connection;
mod result;
mod session;
mod transaction;

pub use connection::GqlConnection;
pub use result::ResultCursor;
pub use session::GqlSession;
pub use transaction::Transaction;
