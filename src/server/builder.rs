//! Server builder for configuring and starting the gRPC server.

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tonic::transport::server::{Router, TcpIncoming};

use crate::proto::admin_service_server::AdminServiceServer;
use crate::proto::catalog_service_server::CatalogServiceServer;
use crate::proto::gql_service_server::GqlServiceServer;
use crate::proto::search_service_server::SearchServiceServer;
use crate::proto::session_service_server::SessionServiceServer;

use super::admin_service::AdminServiceImpl;
use super::auth::AuthValidator;
use super::backend::GqlBackend;
use super::catalog_service::CatalogServiceImpl;
use super::gql_service::GqlServiceImpl;
use super::search_service::SearchServiceImpl;
use super::session_service::{SessionServiceImpl, release_session};
use super::{SessionManager, TransactionManager};

/// Shortest interval between two runs of the idle session reaper.
///
/// The reaper runs every `idle_timeout / 2`; this floor keeps a zero or
/// sub-millisecond timeout from turning into a zero period, which
/// `tokio::time::interval` rejects with a panic.
const MIN_REAPER_INTERVAL: Duration = Duration::from_millis(1);

/// Longest interval between two runs of the idle session reaper.
///
/// Keeps a huge timeout (such as `Duration::MAX`) from overflowing the
/// interval's next deadline, which panics inside tokio when a tick is late.
const MAX_REAPER_INTERVAL: Duration = Duration::from_secs(3600);

/// How often the reaper looks for sessions idle longer than `timeout`.
fn reaper_interval(timeout: Duration) -> Duration {
    (timeout / 2).clamp(MIN_REAPER_INTERVAL, MAX_REAPER_INTERVAL)
}

/// Builder for the GQL wire protocol server.
///
/// ```rust,no_run
/// use std::net::SocketAddr;
/// use std::time::Duration;
/// use gwp::server::{GqlServer, GqlBackend};
///
/// # async fn example(backend: impl GqlBackend) -> Result<(), tonic::transport::Error> {
/// let addr: SocketAddr = "0.0.0.0:7687".parse().unwrap();
///
/// GqlServer::builder(backend)
///     .idle_timeout(Duration::from_secs(300))
///     .max_sessions(256)
///     .shutdown(async { drop(tokio::signal::ctrl_c().await) })
///     .serve(addr)
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct GqlServer<B: GqlBackend> {
    backend: B,
    #[cfg(feature = "tls")]
    tls_config: Option<tonic::transport::ServerTlsConfig>,
    auth_validator: Option<Arc<dyn AuthValidator>>,
    idle_timeout: Option<Duration>,
    max_sessions: Option<usize>,
    shutdown: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

/// A configured router plus the background tasks that live as long as it.
struct Prepared {
    router: Router,
    reaper: Option<Reaper>,
    shutdown: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

/// The idle session reaper task and the token that stops it.
struct Reaper {
    handle: JoinHandle<()>,
    token: CancellationToken,
}

/// Stop the idle session reaper once the server has stopped.
async fn stop(reaper: Option<Reaper>) {
    if let Some(Reaper { handle, token }) = reaper {
        token.cancel();
        let _ = handle.await;
    }
    tracing::info!("GWP server stopped");
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
        let Prepared {
            router,
            reaper,
            shutdown,
        } = self.prepare().await?;

        tracing::info!(%addr, "GWP server listening");

        let result = if let Some(signal) = shutdown {
            router.serve_with_shutdown(addr, signal).await
        } else {
            router.serve(addr).await
        };

        stop(reaper).await;
        result
    }

    /// Build and start serving on an already bound TCP listener.
    ///
    /// Behaves like [`serve`](Self::serve), but takes the listener instead
    /// of an address. Binding to port 0 first and passing the listener lets
    /// tests and embedders learn the port before the server starts, without
    /// the race of binding, dropping and binding again.
    ///
    /// ```rust,no_run
    /// use gwp::server::{GqlServer, GqlBackend};
    ///
    /// # async fn example(backend: impl GqlBackend) -> Result<(), Box<dyn std::error::Error>> {
    /// let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    /// let addr = listener.local_addr()?;
    /// tokio::spawn(GqlServer::builder(backend).serve_with_listener(listener));
    /// // connect to `addr` ...
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start.
    pub async fn serve_with_listener(
        self,
        listener: tokio::net::TcpListener,
    ) -> Result<(), tonic::transport::Error> {
        let Prepared {
            router,
            reaper,
            shutdown,
        } = self.prepare().await?;

        if let Ok(addr) = listener.local_addr() {
            tracing::info!(%addr, "GWP server listening");
        }
        // Same socket option as `Server::builder()` applies in `serve`.
        let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));

        let result = if let Some(signal) = shutdown {
            router.serve_with_incoming_shutdown(incoming, signal).await
        } else {
            router.serve_with_incoming(incoming).await
        };

        stop(reaper).await;
        result
    }

    /// Wire up the services, the health reporter and the idle reaper.
    async fn prepare(self) -> Result<Prepared, tonic::transport::Error> {
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

        let catalog_service = CatalogServiceImpl::new(Arc::clone(&backend));
        let admin_service = AdminServiceImpl::new(Arc::clone(&backend));
        let search_service = SearchServiceImpl::new(Arc::clone(&backend));

        // Health check service
        let (health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter
            .set_serving::<SessionServiceServer<SessionServiceImpl<B>>>()
            .await;
        health_reporter
            .set_serving::<GqlServiceServer<GqlServiceImpl<B>>>()
            .await;
        health_reporter
            .set_serving::<CatalogServiceServer<CatalogServiceImpl<B>>>()
            .await;
        health_reporter
            .set_serving::<AdminServiceServer<AdminServiceImpl<B>>>()
            .await;
        health_reporter
            .set_serving::<SearchServiceServer<SearchServiceImpl<B>>>()
            .await;

        let mut server = Server::builder();

        #[cfg(feature = "tls")]
        if let Some(tls) = self.tls_config {
            server = server.tls_config(tls)?;
        }

        let router = server
            .add_service(health_service)
            .add_service(SessionServiceServer::new(session_service))
            .add_service(GqlServiceServer::new(gql_service))
            .add_service(CatalogServiceServer::new(catalog_service))
            .add_service(AdminServiceServer::new(admin_service))
            .add_service(SearchServiceServer::new(search_service));

        let reaper = self
            .idle_timeout
            .map(|timeout| spawn_reaper(backend, sessions, transactions, timeout));

        Ok(Prepared {
            router,
            reaper,
            shutdown: self.shutdown,
        })
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

/// Spawn the idle session reaper.
///
/// Expired sessions are cleaned up exactly like an explicit `CloseSession`:
/// their active transaction is rolled back, then the backend session closed.
fn spawn_reaper<B: GqlBackend>(
    backend: Arc<B>,
    sessions: SessionManager,
    transactions: TransactionManager,
    timeout: Duration,
) -> Reaper {
    let token = CancellationToken::new();
    let reaper_token = token.clone();
    let period = reaper_interval(timeout);
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let expired = sessions.reap_idle(timeout).await;
                    for session_id in &expired {
                        if let Err(err) =
                            release_session(&*backend, &transactions, session_id).await
                        {
                            tracing::warn!(session_id, error = %err, "closing idle session failed");
                        }
                    }
                }
                () = reaper_token.cancelled() => {
                    tracing::info!("session reaper stopped");
                    break;
                }
            }
        }
    });
    Reaper { handle, token }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaper_interval_is_half_the_timeout() {
        assert_eq!(
            reaper_interval(Duration::from_secs(300)),
            Duration::from_secs(150)
        );
        assert_eq!(
            reaper_interval(Duration::from_millis(10)),
            Duration::from_millis(5)
        );
    }

    #[test]
    fn reaper_interval_is_never_zero() {
        // `tokio::time::interval` panics on a zero period.
        for timeout in [
            Duration::ZERO,
            Duration::from_nanos(1),
            Duration::from_micros(1),
        ] {
            assert_eq!(reaper_interval(timeout), MIN_REAPER_INTERVAL);
        }
    }

    #[test]
    fn reaper_interval_of_a_huge_timeout_stays_addable() {
        // A late tick adds the period to an `Instant` without a check.
        for timeout in [Duration::MAX, Duration::from_secs(u64::MAX / 3)] {
            let period = reaper_interval(timeout);
            assert_eq!(period, MAX_REAPER_INTERVAL);
            assert!(tokio::time::Instant::now().checked_add(period).is_some());
        }
    }
}
