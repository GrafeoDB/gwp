//! gRPC connection management.

use tonic::transport::Channel;

use crate::error::GqlError;
use crate::proto;

use super::GqlSession;
use super::admin::AdminClient;
use super::catalog::CatalogClient;
use super::search::SearchClient;

/// A connection to a GQL wire protocol server.
///
/// Manages the gRPC channel and provides session creation.
#[derive(Debug, Clone)]
pub struct GqlConnection {
    channel: Channel,
}

impl GqlConnection {
    /// Connect to a GQL server at the given endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use gwp::client::GqlConnection;
    ///
    /// let conn = GqlConnection::connect("http://localhost:50051").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(endpoint: &str) -> Result<Self, GqlError> {
        let channel = Channel::from_shared(endpoint.to_owned())
            .map_err(|e| GqlError::Protocol(e.to_string()))?
            .connect()
            .await?;

        Ok(Self { channel })
    }

    /// Create a connection from an existing tonic channel.
    #[must_use]
    pub fn from_channel(channel: Channel) -> Self {
        Self { channel }
    }

    /// Perform a handshake and return a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the handshake fails.
    pub async fn create_session(&self) -> Result<GqlSession, GqlError> {
        GqlSession::new(self.channel.clone(), None).await
    }

    /// Perform a handshake with credentials and return a session.
    ///
    /// Use this against a server that has an
    /// [`AuthValidator`](crate::server::AuthValidator) configured.
    ///
    /// ```rust,no_run
    /// use gwp::client::GqlConnection;
    /// use gwp::proto::{AuthCredentials, auth_credentials::Method};
    ///
    /// # async fn example() -> Result<(), gwp::error::GqlError> {
    /// let conn = GqlConnection::connect("http://localhost:7687").await?;
    /// let session = conn
    ///     .create_session_with_credentials(AuthCredentials {
    ///         method: Some(Method::BearerToken("my-token".to_owned())),
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the handshake fails, for example with an
    /// `UNAUTHENTICATED` gRPC status when the credentials are rejected.
    pub async fn create_session_with_credentials(
        &self,
        credentials: proto::AuthCredentials,
    ) -> Result<GqlSession, GqlError> {
        GqlSession::new(self.channel.clone(), Some(credentials)).await
    }

    /// Create a catalog management client (schemas, graphs, graph types).
    #[must_use]
    pub fn create_catalog_client(&self) -> CatalogClient {
        CatalogClient::new(self.channel.clone())
    }

    /// Create an admin client (stats, WAL, validation, indexes).
    #[must_use]
    pub fn create_admin_client(&self) -> AdminClient {
        AdminClient::new(self.channel.clone())
    }

    /// Create a search client (vector, text, hybrid).
    #[must_use]
    pub fn create_search_client(&self) -> SearchClient {
        SearchClient::new(self.channel.clone())
    }

    /// Connect to a GQL server with TLS.
    ///
    /// Requires the `tls` feature to be enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established.
    #[cfg(feature = "tls")]
    pub async fn connect_tls(
        endpoint: &str,
        tls_config: tonic::transport::ClientTlsConfig,
    ) -> Result<Self, GqlError> {
        let channel = Channel::from_shared(endpoint.to_owned())
            .map_err(|e| GqlError::Protocol(e.to_string()))?
            .tls_config(tls_config)
            .map_err(|e| GqlError::Protocol(e.to_string()))?
            .connect()
            .await?;

        Ok(Self { channel })
    }

    /// Get the underlying tonic channel.
    #[must_use]
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}
