//! Server builder for configuring and starting the gRPC server.

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
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
    shutdown: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
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
            shutdown: None,
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

    /// Set a shutdown signal.
    ///
    /// When the future completes, the server will stop accepting new
    /// connections and drain in-flight requests before returning.
    /// The idle session reaper is also stopped on shutdown.
    #[must_use]
    pub fn shutdown(mut self, signal: impl Future<Output = ()> + Send + 'static) -> Self {
        self.shutdown = Some(Box::pin(signal));
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

        // Health check service
        let (health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter
            .set_serving::<SessionServiceServer<SessionServiceImpl<B>>>()
            .await;
        health_reporter
            .set_serving::<GqlServiceServer<GqlServiceImpl<B>>>()
            .await;
        health_reporter
            .set_serving::<DatabaseServiceServer<DatabaseServiceImpl<B>>>()
            .await;

        // Idle session reaper
        let reaper_handle = if let Some(timeout) = self.idle_timeout {
            let reaper_sessions = sessions.clone();
            let reaper_transactions = transactions.clone();
            let reaper_backend = Arc::clone(&backend);
            let token = tokio_util::sync::CancellationToken::new();
            let reaper_token = token.clone();
            let handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(timeout / 2);
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            let expired = reaper_sessions.reap_idle(timeout).await;
                            for session_id in &expired {
                                reaper_transactions.remove_for_session(session_id).await;
                                let _ = reaper_backend
                                    .close_session(&SessionHandle(session_id.clone()))
                                    .await;
                            }
                        }
                        () = reaper_token.cancelled() => {
                            tracing::info!("session reaper stopped");
                            break;
                        }
                    }
                }
            });
            Some((handle, token))
        } else {
            None
        };

        let mut server = Server::builder();

        #[cfg(feature = "tls")]
        if let Some(tls) = self.tls_config {
            server = server.tls_config(tls)?;
        }

        let router = server
            .add_service(health_service)
            .add_service(SessionServiceServer::new(session_service))
            .add_service(GqlServiceServer::new(gql_service))
            .add_service(DatabaseServiceServer::new(database_service));

        tracing::info!(%addr, "GWP server listening");

        let result = if let Some(signal) = self.shutdown {
            router.serve_with_shutdown(addr, signal).await
        } else {
            router.serve(addr).await
        };

        // Stop the reaper on shutdown
        if let Some((handle, token)) = reaper_handle {
            token.cancel();
            let _ = handle.await;
        }

        tracing::info!("GWP server stopped");

        result
    }

    /// Convenience method: build and serve with default settings.
    ///
    /// Listens for Ctrl-C and shuts down gracefully.
    ///
    /// # Panics
    ///
    /// Panics if the Ctrl-C signal handler cannot be installed.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to bind or start.
    pub async fn start(backend: B, addr: SocketAddr) -> Result<(), tonic::transport::Error> {
        Self::builder(backend)
            .shutdown(async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to listen for ctrl-c");
                tracing::info!("ctrl-c received, shutting down");
            })
            .serve(addr)
            .await
    }
}
