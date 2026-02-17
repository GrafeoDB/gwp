//! Authentication for the GQL wire protocol server.

use crate::error::GqlError;
use crate::proto;

/// Validates client credentials during handshake.
///
/// Implement this trait to add authentication to the server.
/// If no validator is configured on the server builder, all
/// connections are accepted.
#[tonic::async_trait]
pub trait AuthValidator: Send + Sync + 'static {
    /// Validate the given credentials.
    ///
    /// Return `Ok(())` to accept, or `Err(GqlError)` to reject.
    async fn validate(&self, credentials: &proto::AuthCredentials) -> Result<(), GqlError>;
}
