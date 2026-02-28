//! Ergonomic Rust client for the GQL wire protocol.
//!
//! Wraps the raw tonic gRPC stubs with a typed, session-oriented API.

mod admin;
mod catalog;
mod connection;
mod result;
mod search;
mod session;
mod transaction;

pub use admin::AdminClient;
pub use catalog::CatalogClient;
pub use connection::GqlConnection;
pub use result::ResultCursor;
pub use search::SearchClient;
pub use session::GqlSession;
pub use transaction::Transaction;
