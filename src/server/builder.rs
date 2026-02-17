//! Server builder for configuring and starting the gRPC server.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tonic::transport::Server;

use crate::proto::database_service_server::DatabaseServiceServer;
use crate::proto::gql_service_server::GqlServiceServer;
use crate::proto::session_service_server::SessionServiceServer;

use super::auth::AuthValidator;
use super::backend::{GqlBackend, SessionHandle};
use super::database_service::DatabaseServiceImpl;
use super::gql_service::GqlServiceImpl;
use super::session_service::SessionServiceImpl;
use super::{SessionManager, TransactionManager};

/// Builder for the GQL wire protocol server.
pub struct GqlServer<B: GqlBackend> {
    backend: B,
    #[cfg(feature = "tls")]
    tls_config: Option<tonic::transport::ServerTlsConfig>,
    auth_validator: Option<Arc<dyn AuthValidator>>,
    idle_timeout: Option<Duration>,
    max_sessions: Option<usize>,
}

impl<B: GqlBackend> GqlServer<B> {
    /// Start building a server with the given backend.
    #[must_use]
    pub fn builder(backend: B) -> Self {
        Self {
            backend,
            #[cfg(feature = "tls")]
            tls_config: None,
            auth_validator: None,
            idle_timeout: None,
            max_sessions: None,
        }
    }

    /// Set TLS configuration for the server.
    ///
    /// Requires the `tls` feature to be enabled.
    #[cfg(feature = "tls")]
    #[must_use]
    pub fn tls(mut self, config: tonic::transport::ServerTlsConfig) -> Self {
        self.tls_config = Some(config);
        self
    }

    /// Set an authentication validator.
    ///
    /// When set, the server requires valid credentials on every handshake.
    /// When not set, all connections are accepted.
    #[must_use]
    pub fn auth(mut self, validator: impl AuthValidator) -> Self {
        self.auth_validator = Some(Arc::new(validator));
        self
    }

    /// Set the idle timeout for sessions.
    ///
    /// Sessions with no activity for longer than this duration will be
    /// automatically closed and their transactions rolled back.
    /// When not set, sessions live until explicitly closed.
    #[must_use]
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = Some(timeout);
        self
    }

    /// Set the maximum number of concurrent sessions.
    ///
    /// When the limit is reached, new handshake requests will be
    /// rejected with `RESOURCE_EXHAUSTED`.
    #[must_use]
    pub fn max_sessions(mut self, limit: usize) -> Self {
        self.max_sessions = Some(limit);
        self
    }

    /// Build and start serving on the given address.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to bind or start.
    pub async fn serve(self, addr: SocketAddr) -> Result<(), tonic::transport::Error> {
        let backend = Arc::new(self.backend);
        let sessions = match self.max_sessions {
            Some(limit) => SessionManager::with_capacity(limit),
            None => SessionManager::new(),
        };
        let transactions = TransactionManager::new();

        let session_service = SessionServiceImpl::new(
            Arc::clone(&backend),
            sessions.clone(),
            transactions.clone(),
            self.auth_validator,
        );

        let gql_service =
            GqlServiceImpl::new(Arc::clone(&backend), sessions.clone(), transactions.clone());

        let database_service = DatabaseServiceImpl::new(Arc::clone(&backend));

        if let Some(timeout) = self.idle_timeout {
            let reaper_sessions = sessions.clone();
            let reaper_transactions = transactions.clone();
            let reaper_backend = Arc::clone(&backend);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(timeout / 2);
                loop {
                    interval.tick().await;
                    let expired = reaper_sessions.reap_idle(timeout).await;
                    for session_id in &expired {
                        reaper_transactions.remove_for_session(session_id).await;
                        let _ = reaper_backend
                            .close_session(&SessionHandle(session_id.clone()))
                            .await;
                    }
                }
            });
        }

        let mut server = Server::builder();

        #[cfg(feature = "tls")]
        if let Some(tls) = self.tls_config {
            server = server.tls_config(tls)?;
        }

        server
            .add_service(SessionServiceServer::new(session_service))
            .add_service(GqlServiceServer::new(gql_service))
            .add_service(DatabaseServiceServer::new(database_service))
            .serve(addr)
            .await
    }

    /// Convenience method: build and serve with default settings.
    ///
    /// Equivalent to `GqlServer::builder(backend).serve(addr)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to bind or start.
    pub async fn start(backend: B, addr: SocketAddr) -> Result<(), tonic::transport::Error> {
        Self::builder(backend).serve(addr).await
    }
}
