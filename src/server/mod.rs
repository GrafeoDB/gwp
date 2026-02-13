//! gRPC server implementation.
//!
//! Provides the `SessionService` and `GqlService` implementations,
//! session/transaction state management, and the pluggable `GqlBackend` trait.

mod backend;
pub mod builder;
mod gql_service;
pub mod mock_backend;
mod session_manager;
mod session_service;
mod transaction_manager;

pub use backend::{
    GqlBackend, ResetTarget, ResultFrame, ResultStream, SessionConfig, SessionHandle,
    SessionProperty, TransactionHandle,
};
pub use builder::GqlServer;
pub use gql_service::GqlServiceImpl;
pub use session_manager::SessionManager;
pub use session_service::SessionServiceImpl;
pub use transaction_manager::TransactionManager;
