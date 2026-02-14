//! gRPC connection management.

use tonic::transport::Channel;

use crate::error::GqlError;

use super::GqlSession;

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
        GqlSession::new(self.channel.clone()).await
    }

    /// Get the underlying tonic channel.
    #[must_use]
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}
