//! `GqlService` gRPC implementation.
//!
//! All GQL-domain errors are returned as GQLSTATUS codes in the
//! response payload. gRPC status is always OK unless there is a
//! transport-level failure.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use crate::proto;
use crate::proto::gql_service_server::GqlService;
use crate::status as gql_status;
use crate::types::Value;

use super::backend::{GqlBackend, ResultFrame, ResultStream};
use super::{SessionHandle, SessionManager, TransactionHandle, TransactionManager};

/// Number of bytes of a statement recorded in the tracing span.
const STATEMENT_LOG_LIMIT: usize = 100;

/// Returns at most `max_bytes` of `text`, cut at a character boundary.
///
/// Statements are untrusted input: slicing them at a fixed byte offset
/// panics when the offset falls inside a multi-byte character.
fn truncate_for_log(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    let mut end = max_bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Implementation of the `GqlService` gRPC service.
pub struct GqlServiceImpl<B: GqlBackend> {
    backend: Arc<B>,
    sessions: SessionManager,
    transactions: TransactionManager,
}

impl<B: GqlBackend> GqlServiceImpl<B> {
    /// Create a new GQL service.
    pub fn new(
        backend: Arc<B>,
        sessions: SessionManager,
        transactions: TransactionManager,
    ) -> Self {
        Self {
            backend,
            sessions,
            transactions,
        }
    }

    /// Validate a session exists and update its activity timestamp.
    async fn validate_session(&self, session_id: &str) -> Result<(), Status> {
        if self.sessions.exists(session_id).await {
            self.sessions.touch(session_id).await;
            Ok(())
        } else {
            Err(Status::not_found(format!("session {session_id} not found")))
        }
    }

    /// Forget a terminated transaction.
    ///
    /// Returns `false` if the transaction was no longer registered, which
    /// happens when a concurrent close of its session took it over.
    async fn finish_transaction(&self, session_id: &str, transaction_id: &str) -> bool {
        // Clear the session's marker before releasing the transaction: the
        // terminating transaction blocks a new begin until it is removed,
        // so this cannot clear the marker of a newer transaction.
        if self
            .sessions
            .active_transaction(session_id)
            .await
            .is_some_and(|active| active == transaction_id)
        {
            // The session may have been closed meanwhile; nothing to clear then.
            let _ = self.sessions.set_active_transaction(session_id, None).await;
        }
        self.transactions.remove(transaction_id).await.is_ok()
    }
}

#[tonic::async_trait]
impl<B: GqlBackend> GqlService for GqlServiceImpl<B> {
    type ExecuteStream = Pin<Box<dyn Stream<Item = Result<proto::ExecuteResponse, Status>> + Send>>;

    #[tracing::instrument(skip(self, request), fields(session_id, statement))]
    async fn execute(
        &self,
        request: Request<proto::ExecuteRequest>,
    ) -> Result<Response<Self::ExecuteStream>, Status> {
        let req = request.into_inner();
        let span = tracing::Span::current();
        span.record("session_id", &req.session_id);
        span.record(
            "statement",
            tracing::field::display(truncate_for_log(&req.statement, STATEMENT_LOG_LIMIT)),
        );

        self.validate_session(&req.session_id).await?;

        let session = SessionHandle(req.session_id.clone());
        let transaction = if let Some(ref tx_id) = req.transaction_id {
            // Validate the transaction belongs to this session
            self.transactions
                .validate(tx_id, &req.session_id)
                .await
                .map_err(|e| e.to_grpc_status())?;
            Some(TransactionHandle(tx_id.clone()))
        } else {
            None
        };

        let parameters: HashMap<String, Value> = req
            .parameters
            .into_iter()
            .map(|(k, v)| (k, Value::from(v)))
            .collect();

        let result_stream = self
            .backend
            .execute(&session, &req.statement, &parameters, transaction.as_ref())
            .await;

        match result_stream {
            Ok(stream) => {
                let output = ResultStreamAdapter {
                    inner: Some(stream),
                };
                Ok(Response::new(Box::pin(output)))
            }
            Err(err) => {
                tracing::warn!(error = %err, "execute failed");
                // GQL errors go in the response payload, not gRPC status
                let status = match err.gql_status() {
                    Some(s) => s.clone(),
                    None => gql_status::error(gql_status::DATA_EXCEPTION, err.to_string()),
                };

                let summary_stream = futures_single_response(proto::ExecuteResponse {
                    frame: Some(proto::execute_response::Frame::Summary(
                        proto::ResultSummary {
                            status: Some(status),
                            warnings: Vec::new(),
                            rows_affected: 0,
                            counters: HashMap::new(),
                        },
                    )),
                });

                Ok(Response::new(Box::pin(summary_stream)))
            }
        }
    }

    #[tracing::instrument(skip(self, request), fields(session_id))]
    async fn begin_transaction(
        &self,
        request: Request<proto::BeginRequest>,
    ) -> Result<Response<proto::BeginResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("session_id", &req.session_id);
        self.validate_session(&req.session_id).await?;

        let session = SessionHandle(req.session_id.clone());
        // An unknown mode is rejected: silently treating it as READ WRITE
        // would grant more than the client asked for.
        let mode = proto::TransactionMode::try_from(req.mode).map_err(|_| {
            Status::invalid_argument(format!("invalid transaction mode {}", req.mode))
        })?;

        // At most one transaction per session (sec 8.1). Checked before the
        // backend is involved, so that a second begin does not reach it
        // (only two begins racing on one session can both get through).
        if self.transactions.has_transaction(&req.session_id).await {
            tracing::warn!(session_id = %req.session_id, "double begin rejected");
            return Ok(Response::new(proto::BeginResponse {
                transaction_id: String::new(),
                status: Some(gql_status::error(
                    gql_status::ACTIVE_TRANSACTION,
                    "session already has an active transaction",
                )),
            }));
        }

        match self.backend.begin_transaction(&session, mode).await {
            Ok(handle) => {
                let tx_id = handle.0.clone();

                if let Err(e) = self
                    .transactions
                    .register(&tx_id, &req.session_id, mode)
                    .await
                {
                    // A concurrent begin on the same session won the race:
                    // roll back the backend transaction we cannot track.
                    if let Err(err) = self.backend.rollback(&session, &handle).await {
                        tracing::warn!(error = %err, "rollback of rejected transaction failed");
                    }
                    tracing::warn!(session_id = %req.session_id, "double begin rejected");
                    return Ok(Response::new(proto::BeginResponse {
                        transaction_id: String::new(),
                        status: Some(gql_status::error(
                            gql_status::ACTIVE_TRANSACTION,
                            e.to_string(),
                        )),
                    }));
                }

                if self
                    .sessions
                    .set_active_transaction(&req.session_id, Some(tx_id.clone()))
                    .await
                    .is_err()
                {
                    // The session was closed while the backend was starting
                    // the transaction. Unless the close already took it over,
                    // forget the transaction and roll it back here, so that
                    // it does not outlive its session.
                    if self.transactions.remove(&tx_id).await.is_ok() {
                        if let Err(err) = self.backend.rollback(&session, &handle).await {
                            tracing::warn!(error = %err, "rollback after concurrent close failed");
                        }
                    }
                    return Err(Status::not_found(format!(
                        "session {} not found",
                        req.session_id
                    )));
                }

                tracing::info!(session_id = %req.session_id, transaction_id = %tx_id, "transaction started");

                Ok(Response::new(proto::BeginResponse {
                    transaction_id: tx_id,
                    status: Some(gql_status::success()),
                }))
            }
            Err(err) => {
                let status = match err.gql_status() {
                    Some(s) => s.clone(),
                    None => gql_status::error(gql_status::ACTIVE_TRANSACTION, err.to_string()),
                };
                Ok(Response::new(proto::BeginResponse {
                    transaction_id: String::new(),
                    status: Some(status),
                }))
            }
        }
    }

    #[tracing::instrument(skip(self, request), fields(session_id, transaction_id))]
    async fn commit(
        &self,
        request: Request<proto::CommitRequest>,
    ) -> Result<Response<proto::CommitResponse>, Status> {
        let req = request.into_inner();
        let span = tracing::Span::current();
        span.record("session_id", &req.session_id);
        span.record("transaction_id", &req.transaction_id);
        self.validate_session(&req.session_id).await?;

        // Claim the transaction: a concurrent commit, rollback or statement
        // on it is rejected from here on.
        if let Err(e) = self
            .transactions
            .begin_termination(&req.transaction_id, &req.session_id)
            .await
        {
            return Ok(Response::new(proto::CommitResponse {
                status: Some(gql_status::error(
                    gql_status::INVALID_TRANSACTION_STATE,
                    e.to_string(),
                )),
            }));
        }

        let session = SessionHandle(req.session_id.clone());
        let transaction = TransactionHandle(req.transaction_id.clone());

        let result = self.backend.commit(&session, &transaction).await;

        // A commit ends the transaction whatever its outcome (sec 8.4): a
        // failed commit cancels its changes. Keeping it registered would
        // leave the session unable to begin another transaction.
        let still_owned = self
            .finish_transaction(&req.session_id, &req.transaction_id)
            .await;

        match result {
            Ok(()) => {
                tracing::info!("transaction committed");

                Ok(Response::new(proto::CommitResponse {
                    status: Some(gql_status::success()),
                }))
            }
            Err(err) => {
                tracing::warn!(error = %err, "commit failed");
                // Make sure the backend does not keep the failed transaction
                // open (unless a concurrent close already took it over).
                if still_owned {
                    if let Err(rollback_err) = self.backend.rollback(&session, &transaction).await {
                        tracing::debug!(error = %rollback_err, "rollback after failed commit");
                    }
                }
                let status = match err.gql_status() {
                    Some(s) => s.clone(),
                    None => gql_status::error(gql_status::TRANSACTION_ROLLBACK, err.to_string()),
                };
                Ok(Response::new(proto::CommitResponse {
                    status: Some(status),
                }))
            }
        }
    }

    #[tracing::instrument(skip(self, request), fields(session_id, transaction_id))]
    async fn rollback(
        &self,
        request: Request<proto::RollbackRequest>,
    ) -> Result<Response<proto::RollbackResponse>, Status> {
        let req = request.into_inner();
        let span = tracing::Span::current();
        span.record("session_id", &req.session_id);
        span.record("transaction_id", &req.transaction_id);
        self.validate_session(&req.session_id).await?;

        if let Err(e) = self
            .transactions
            .begin_termination(&req.transaction_id, &req.session_id)
            .await
        {
            return Ok(Response::new(proto::RollbackResponse {
                status: Some(gql_status::error(
                    gql_status::INVALID_TRANSACTION_STATE,
                    e.to_string(),
                )),
            }));
        }

        let session = SessionHandle(req.session_id.clone());
        let transaction = TransactionHandle(req.transaction_id.clone());

        let result = self.backend.rollback(&session, &transaction).await;

        // The transaction is terminated even when the backend reports an
        // error (sec 8.3): there is nothing left for the client to retry.
        self.finish_transaction(&req.session_id, &req.transaction_id)
            .await;

        match result {
            Ok(()) => {
                tracing::info!("transaction rolled back");

                Ok(Response::new(proto::RollbackResponse {
                    status: Some(gql_status::success()),
                }))
            }
            Err(err) => {
                tracing::warn!(error = %err, "rollback failed");
                let status = match err.gql_status() {
                    Some(s) => s.clone(),
                    None => gql_status::error(gql_status::TRANSACTION_ROLLBACK, err.to_string()),
                };
                Ok(Response::new(proto::RollbackResponse {
                    status: Some(status),
                }))
            }
        }
    }
}

// ============================================================================
// Stream adapters
// ============================================================================

/// Adapts a `ResultStream` into a tonic-compatible `Stream`.
///
/// The summary is always the last frame: once the backend stream has
/// produced a summary (or an error, which is sent as a summary), the
/// backend stream is dropped and nothing else is sent. The backend stream
/// is also dropped when the client goes away, since tonic then drops this
/// adapter.
struct ResultStreamAdapter {
    inner: Option<Pin<Box<dyn ResultStream>>>,
}

impl Stream for ResultStreamAdapter {
    type Item = Result<proto::ExecuteResponse, Status>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let Some(inner) = self.inner.as_mut() else {
            return std::task::Poll::Ready(None);
        };
        match inner.as_mut().poll_next(cx) {
            std::task::Poll::Ready(Some(Ok(frame))) => {
                let response = match frame {
                    ResultFrame::Header(h) => proto::ExecuteResponse {
                        frame: Some(proto::execute_response::Frame::Header(h)),
                    },
                    ResultFrame::Batch(b) => proto::ExecuteResponse {
                        frame: Some(proto::execute_response::Frame::RowBatch(b)),
                    },
                    ResultFrame::Summary(s) => {
                        self.inner = None;
                        proto::ExecuteResponse {
                            frame: Some(proto::execute_response::Frame::Summary(s)),
                        }
                    }
                };
                std::task::Poll::Ready(Some(Ok(response)))
            }
            std::task::Poll::Ready(Some(Err(err))) => {
                self.inner = None;
                // Convert backend error to a summary frame with GQLSTATUS
                let status = match err.gql_status() {
                    Some(s) => s.clone(),
                    None => gql_status::error(gql_status::DATA_EXCEPTION, err.to_string()),
                };
                let response = proto::ExecuteResponse {
                    frame: Some(proto::execute_response::Frame::Summary(
                        proto::ResultSummary {
                            status: Some(status),
                            warnings: Vec::new(),
                            rows_affected: 0,
                            counters: HashMap::new(),
                        },
                    )),
                };
                std::task::Poll::Ready(Some(Ok(response)))
            }
            std::task::Poll::Ready(None) => {
                self.inner = None;
                std::task::Poll::Ready(None)
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// Create a stream that yields a single response then completes.
fn futures_single_response(
    response: proto::ExecuteResponse,
) -> impl Stream<Item = Result<proto::ExecuteResponse, Status>> {
    tokio_stream::once(Ok(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_for_log_respects_char_boundaries() {
        assert_eq!(truncate_for_log("short", 100), "short");
        assert_eq!(truncate_for_log("abcdef", 3), "abc");

        // 'é' is two bytes: a cut at byte 3 falls inside the second one.
        assert_eq!(truncate_for_log("éé", 3), "é");
        // '€' is three bytes.
        let statement = format!("{}€ tail", "a".repeat(99));
        assert_eq!(truncate_for_log(&statement, 100), "a".repeat(99));
        assert_eq!(truncate_for_log("€", 2), "");
    }
}
