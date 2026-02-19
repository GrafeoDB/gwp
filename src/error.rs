//! Crate error types for the GQL wire protocol.
//!
//! Errors are split by domain:
//! - Protocol-level errors (wire format, framing)
//! - Session errors (not found, expired)
//! - Transaction errors (invalid state transitions)
//! - Backend errors (from the pluggable database engine)
//! - GQL-domain errors (carrying a GQLSTATUS code)

use crate::proto;

/// The main error type for the GQL wire protocol crate.
#[derive(Debug, thiserror::Error)]
pub enum GqlError {
    /// Wire-level protocol error.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Session not found or expired.
    #[error("session error: {0}")]
    Session(String),

    /// Invalid transaction state transition.
    #[error("transaction error: {0}")]
    Transaction(String),

    /// Error from the backend database engine.
    #[error("backend error: {source}")]
    Backend {
        /// The underlying backend error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// GQL-domain error carrying a GQLSTATUS code.
    #[error("GQL error {}: {}", .status.code, .status.message)]
    Status {
        /// The GQLSTATUS from the failed operation.
        status: proto::GqlStatus,
    },

    /// Transport-level error from tonic/gRPC.
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    /// gRPC status error from tonic.
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
}

impl GqlError {
    /// Create a backend error from any error type.
    pub fn backend(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Backend {
            source: Box::new(err),
        }
    }

    /// Create a GQL-domain error from a status.
    #[must_use]
    pub fn status(code: &str, message: impl Into<String>) -> Self {
        Self::Status {
            status: crate::status::error(code, message),
        }
    }

    /// Convert this error to a `tonic::Status` for `SessionService` responses.
    ///
    /// Maps crate errors to appropriate gRPC status codes.
    #[must_use]
    pub fn to_grpc_status(&self) -> tonic::Status {
        match self {
            Self::Session(msg) => tonic::Status::not_found(msg.clone()),
            Self::Transaction(msg) => tonic::Status::failed_precondition(msg.clone()),
            Self::Protocol(msg) => tonic::Status::invalid_argument(msg.clone()),
            Self::Backend { source } => tonic::Status::internal(source.to_string()),
            Self::Status { status } => {
                tonic::Status::internal(format!("{}: {}", status.code, status.message))
            }
            Self::Transport(err) => tonic::Status::unavailable(err.to_string()),
            Self::Grpc(status) => status.clone(),
        }
    }

    /// Convert this error to a `tonic::Status` for optional service responses
    /// (`AdminService`, `SearchService`, `DatabaseService`).
    ///
    /// Maps `Protocol` to `UNIMPLEMENTED` (backend doesn't support the operation)
    /// and `Session` containing "not found" to `NOT_FOUND`.
    #[must_use]
    pub fn to_optional_service_status(&self) -> tonic::Status {
        match self {
            Self::Session(msg) if msg.contains("not found") => {
                tonic::Status::not_found(msg.clone())
            }
            Self::Protocol(msg) => tonic::Status::unimplemented(msg.clone()),
            other => other.to_grpc_status(),
        }
    }

    /// Extract the `GqlStatus` if this is a GQL-domain error.
    #[must_use]
    pub fn gql_status(&self) -> Option<&proto::GqlStatus> {
        match self {
            Self::Status { status } => Some(status),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_error_wrapping() {
        let err = GqlError::backend(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "connection refused",
        ));
        assert!(matches!(err, GqlError::Backend { .. }));
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn status_error() {
        let err = GqlError::status(crate::status::INVALID_SYNTAX, "unexpected token 'METCH'");
        assert!(matches!(err, GqlError::Status { .. }));
        assert!(err.to_string().contains("42001"));
        assert!(err.gql_status().is_some());
    }

    #[test]
    fn session_to_grpc() {
        let err = GqlError::Session("session abc123 not found".to_owned());
        let grpc = err.to_grpc_status();
        assert_eq!(grpc.code(), tonic::Code::NotFound);
    }

    #[test]
    fn non_status_has_no_gql_status() {
        let err = GqlError::Protocol("bad frame".to_owned());
        assert!(err.gql_status().is_none());
    }
}
