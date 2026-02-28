//! gRPC server implementation.
//!
//! Provides the `SessionService`, `GqlService`, and `CatalogService` implementations,
//! session/transaction state management, and the pluggable `GqlBackend` trait.

mod admin_service;
mod auth;
mod backend;
pub mod builder;
mod catalog_service;
mod gql_service;
pub mod mock_backend;
mod search_service;
mod session_manager;
mod session_service;
mod transaction_manager;

pub use admin_service::AdminServiceImpl;
pub use auth::AuthValidator;
pub use backend::{
    AdminStats, AdminValidationResult, AdminWalStatus, CreateGraphConfig, GqlBackend, GraphInfo,
    GraphTypeInfo, GraphTypeSpec, HybridSearchParams, IndexDefinition, ResetTarget, ResultFrame,
    ResultStream, SchemaInfo, SearchHit, SessionConfig, SessionHandle, SessionProperty,
    TextSearchParams, TransactionHandle, ValidationDiagnostic, VectorSearchParams,
};
pub use builder::GqlServer;
pub use catalog_service::CatalogServiceImpl;
pub use gql_service::GqlServiceImpl;
pub use search_service::SearchServiceImpl;
pub use session_manager::SessionManager;
pub use session_service::SessionServiceImpl;
pub use transaction_manager::TransactionManager;
