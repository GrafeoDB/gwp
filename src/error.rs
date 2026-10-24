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
        /// The GQLSTATUS from the failed operation, boxed to keep
        /// `GqlError` (and every `Result` that carries it) small.
        status: Box<proto::GqlStatus>,
    },

    /// Transport-level error from tonic/gRPC.
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    /// gRPC status error from tonic.
    #[error("gRPC error: {0}")]
    Grpc(tonic::Status),
}

/// Metadata key that carries the GQLSTATUS code of a [`GqlError::Status`]
/// sent as a gRPC error.
pub const GQL_STATUS_METADATA_KEY: &str = "gql-status";

/// The gRPC code a GQLSTATUS class is sent with.
fn grpc_code_for_class(class: &str) -> tonic::Code {
    match class {
        "08" => tonic::Code::Unavailable,
        "22" | "42" => tonic::Code::InvalidArgument,
        "25" | "2D" | "G1" | "G2" => tonic::Code::FailedPrecondition,
        "40" => tonic::Code::Aborted,
        _ => tonic::Code::Internal,
    }
}

impl From<tonic::Status> for GqlError {
    /// A gRPC error that carries a `gql-status` entry becomes
    /// [`GqlError::Status`]; any other becomes [`GqlError::Grpc`].
    fn from(grpc: tonic::Status) -> Self {
        let code = grpc
            .metadata()
            .get(GQL_STATUS_METADATA_KEY)
            .and_then(|value| value.to_str().ok());
        match code {
            Some(code) => {
                let message = grpc.message();
                let message = message
                    .strip_prefix(code)
                    .and_then(|rest| rest.strip_prefix(": "))
                    .unwrap_or(message);
                Self::status(code, message)
            }
            None => Self::Grpc(grpc),
        }
    }
}

impl GqlError {
    /// Create a backend error from any error type.
    ///
    /// ```
    /// use gwp::error::GqlError;
    ///
    /// let err = GqlError::backend(std::io::Error::new(
    ///     std::io::ErrorKind::ConnectionRefused,
    ///     "connection refused",
    /// ));
    /// assert!(err.to_string().contains("connection refused"));
    /// ```
    pub fn backend(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Backend {
            source: Box::new(err),
        }
    }

    /// Create a GQL-domain error from a status.
    ///
    /// ```
    /// use gwp::error::GqlError;
    /// use gwp::status::INVALID_SYNTAX;
    ///
    /// let err = GqlError::status(INVALID_SYNTAX, "unexpected token");
    /// assert!(err.gql_status().is_some());
    /// assert!(err.to_string().contains("42001"));
    /// ```
    #[must_use]
    pub fn status(code: &str, message: impl Into<String>) -> Self {
        Self::Status {
            status: Box::new(crate::status::error(code, message)),
        }
    }

    /// Convert this error to a `tonic::Status` for `SessionService` responses.
    ///
    /// Maps crate errors to appropriate gRPC status codes. A
    /// [`GqlError::Status`] gets its gRPC code from the GQLSTATUS class:
    ///
    /// | Class | gRPC code |
    /// |-------|-----------|
    /// | `08` | `UNAVAILABLE` |
    /// | `22`, `42` | `INVALID_ARGUMENT` |
    /// | `25`, `2D`, `G1`, `G2` | `FAILED_PRECONDITION` |
    /// | `40` | `ABORTED` |
    /// | any other | `INTERNAL` |
    ///
    /// and carries its code in the `gql-status` metadata entry
    /// ([`GQL_STATUS_METADATA_KEY`]), which the client turns back into a
    /// [`GqlError::Status`].
    ///
    /// ```
    /// use gwp::error::GqlError;
    /// use gwp::status::ACTIVE_TRANSACTION;
    ///
    /// let grpc = GqlError::status(ACTIVE_TRANSACTION, "transaction open").to_grpc_status();
    /// assert_eq!(grpc.code(), tonic::Code::FailedPrecondition);
    /// assert_eq!(grpc.metadata().get("gql-status").unwrap(), "25G01");
    /// ```
    #[must_use]
    pub fn to_grpc_status(&self) -> tonic::Status {
        match self {
            Self::Session(msg) => tonic::Status::not_found(msg.clone()),
            Self::Transaction(msg) => tonic::Status::failed_precondition(msg.clone()),
            Self::Protocol(msg) => tonic::Status::invalid_argument(msg.clone()),
            Self::Backend { source } => tonic::Status::internal(source.to_string()),
            Self::Status { status } => {
                let mut grpc = tonic::Status::new(
                    grpc_code_for_class(crate::status::class(&status.code)),
                    format!("{}: {}", status.code, status.message),
                );
                // A code that is not valid metadata (it came from the backend
                // unchecked) is still in the message.
                if let Ok(value) = status.code.parse() {
                    grpc.metadata_mut().insert(GQL_STATUS_METADATA_KEY, value);
                }
                grpc
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
    ///
    /// ```
    /// use gwp::error::GqlError;
    ///
    /// let protocol_err = GqlError::Protocol("bad frame".into());
    /// assert!(protocol_err.gql_status().is_none());
    ///
    /// let status_err = GqlError::status("42001", "syntax error");
    /// assert!(status_err.gql_status().is_some());
    /// ```
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
    fn status_to_grpc_code_by_class() {
        use crate::status;
        for (code, expected) in [
            (status::CONNECTION_EXCEPTION, tonic::Code::Unavailable),
            (status::DIVISION_BY_ZERO, tonic::Code::InvalidArgument),
            (status::INVALID_SYNTAX, tonic::Code::InvalidArgument),
            (status::ACTIVE_TRANSACTION, tonic::Code::FailedPrecondition),
            (
                status::INVALID_TRANSACTION_TERMINATION,
                tonic::Code::FailedPrecondition,
            ),
            (status::EDGES_STILL_EXIST, tonic::Code::FailedPrecondition),
            (
                status::GRAPH_TYPE_VIOLATION,
                tonic::Code::FailedPrecondition,
            ),
            (status::TRANSACTION_ROLLBACK, tonic::Code::Aborted),
            ("40001", tonic::Code::Aborted),
            ("XX000", tonic::Code::Internal),
            ("", tonic::Code::Internal),
        ] {
            let grpc = GqlError::status(code, "boom").to_grpc_status();
            assert_eq!(grpc.code(), expected, "{code:?}");
            assert_eq!(grpc.message(), format!("{code}: boom"));
        }
    }

    #[test]
    fn status_round_trips_through_grpc() {
        let grpc = GqlError::status("25G01", "transaction open").to_grpc_status();
        assert_eq!(
            grpc.metadata().get(GQL_STATUS_METADATA_KEY).unwrap(),
            "25G01"
        );
        let back = GqlError::from(grpc);
        let status = back.gql_status().expect("a GqlError::Status");
        assert_eq!(status.code, "25G01");
        assert_eq!(status.message, "transaction open");
    }

    #[test]
    fn status_with_invalid_metadata_code_keeps_it_in_the_message() {
        let grpc = GqlError::status("\n", "bad code").to_grpc_status();
        assert!(grpc.metadata().get(GQL_STATUS_METADATA_KEY).is_none());
        assert_eq!(grpc.message(), "\n: bad code");
        assert!(matches!(GqlError::from(grpc), GqlError::Grpc(_)));
    }

    #[test]
    fn grpc_without_gql_status_stays_grpc() {
        let err = GqlError::from(tonic::Status::not_found("session x not found"));
        assert!(matches!(err, GqlError::Grpc(ref s) if s.code() == tonic::Code::NotFound));
    }

    #[test]
    fn non_status_has_no_gql_status() {
        let err = GqlError::Protocol("bad frame".to_owned());
        assert!(err.gql_status().is_none());
    }
}
