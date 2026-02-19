//! gRPC server implementation.
//!
//! Provides the `SessionService` and `GqlService` implementations,
//! session/transaction state management, and the pluggable `GqlBackend` trait.

mod admin_service;
mod auth;
mod backend;
pub mod builder;
mod database_service;
mod gql_service;
pub mod mock_backend;
mod search_service;
mod session_manager;
mod session_service;
mod transaction_manager;

pub use admin_service::AdminServiceImpl;
pub use auth::AuthValidator;
pub use backend::{
    AdminStats, AdminValidationResult, AdminWalStatus, CreateDatabaseConfig, DatabaseInfo,
    GqlBackend, HybridSearchParams, IndexDefinition, ResetTarget, ResultFrame, ResultStream,
    SearchHit, SessionConfig, SessionHandle, SessionProperty, TextSearchParams, TransactionHandle,
    ValidationDiagnostic, VectorSearchParams,
};
pub use builder::GqlServer;
pub use database_service::DatabaseServiceImpl;
pub use gql_service::GqlServiceImpl;
pub use search_service::SearchServiceImpl;
pub use session_manager::SessionManager;
pub use session_service::SessionServiceImpl;
pub use transaction_manager::TransactionManager;
