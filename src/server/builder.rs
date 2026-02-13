//! Server builder for configuring and starting the gRPC server.

use std::net::SocketAddr;
use std::sync::Arc;

use tonic::transport::Server;

use crate::proto::gql_service_server::GqlServiceServer;
use crate::proto::session_service_server::SessionServiceServer;

use super::backend::GqlBackend;
use super::gql_service::GqlServiceImpl;
use super::session_service::SessionServiceImpl;
use super::{SessionManager, TransactionManager};

/// Builder for the GQL wire protocol server.
pub struct GqlServer;

impl GqlServer {
    /// Create a new server with the given backend.
    ///
    /// Returns a configured `tonic::transport::Server` ready to serve
    /// both `SessionService` and `GqlService`.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to bind or start.
    pub async fn serve<B: GqlBackend>(
        backend: B,
        addr: SocketAddr,
    ) -> Result<(), tonic::transport::Error> {
        let backend = Arc::new(backend);
        let sessions = SessionManager::new();
        let transactions = TransactionManager::new();

        let session_service =
            SessionServiceImpl::new(Arc::clone(&backend), sessions.clone(), transactions.clone());

        let gql_service = GqlServiceImpl::new(Arc::clone(&backend), sessions, transactions);

        Server::builder()
            .add_service(SessionServiceServer::new(session_service))
            .add_service(GqlServiceServer::new(gql_service))
            .serve(addr)
            .await
    }
}
