//! Authentication for the GQL wire protocol server.

use crate::error::GqlError;
use crate::proto;

/// Authenticated identity returned by the validator.
#[derive(Debug, Clone, Default)]
pub struct AuthInfo {
    /// Authenticated principal (user/token identifier).
    pub principal: String,
}

/// Validates client credentials during handshake.
///
/// Implement this trait to add authentication to the server.
/// If no validator is configured on the server builder, all
/// connections are accepted.
#[tonic::async_trait]
pub trait AuthValidator: Send + Sync + 'static {
    /// Validate the given credentials.
    ///
    /// Return `Ok(AuthInfo)` to accept with identity, or `Err(GqlError)` to reject.
    async fn validate(&self, credentials: &proto::AuthCredentials) -> Result<AuthInfo, GqlError>;
}
