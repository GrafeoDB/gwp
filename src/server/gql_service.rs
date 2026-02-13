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

use crate::error::GqlError;
use crate::proto;
use crate::proto::gql_service_server::GqlService;
use crate::types::Value;
use crate::{status as gql_status};

use super::backend::{GqlBackend, ResultFrame, ResultStream};
use super::{SessionHandle, SessionManager, TransactionHandle, TransactionManager};

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

    /// Validate a session exists, returning a gRPC error if not.
    async fn validate_session(&self, session_id: &str) -> Result<(), Status> {
        if self.sessions.exists(session_id).await {
            Ok(())
        } else {
            Err(Status::not_found(format!(
                "session {session_id} not found"
            )))
        }
    }
}

#[tonic::async_trait]
impl<B: GqlBackend> GqlService for GqlServiceImpl<B> {
    type ExecuteStream =
        Pin<Box<dyn Stream<Item = Result<proto::ExecuteResponse, Status>> + Send>>;

    async fn execute(
        &self,
        request: Request<proto::ExecuteRequest>,
    ) -> Result<Response<Self::ExecuteStream>, Status> {
        let req = request.into_inner();
        self.validate_session(&req.session_id).await?;

        let session = SessionHandle(req.session_id.clone());
        let transaction = if req.transaction_id.is_empty() {
            None
        } else {
            // Validate the transaction belongs to this session
            self.transactions
                .validate(&req.transaction_id, &req.session_id)
                .await
                .map_err(|e| e.to_grpc_status())?;
            Some(TransactionHandle(req.transaction_id.clone()))
        };

        let parameters: HashMap<String, Value> = req
            .parameters
            .into_iter()
            .map(|(k, v)| (k, Value::from(v)))
            .collect();

        let result_stream = self
            .backend
            .execute(
                &session,
                &req.statement,
                &parameters,
                transaction.as_ref(),
            )
            .await;

        match result_stream {
            Ok(stream) => {
                let output = ResultStreamAdapter { inner: stream };
                Ok(Response::new(Box::pin(output)))
            }
            Err(err) => {
                // GQL errors go in the response payload, not gRPC status
                let status = match err.gql_status() {
                    Some(s) => s.clone(),
                    None => gql_status::error(
                        gql_status::DATA_EXCEPTION,
                        err.to_string(),
                    ),
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

    async fn begin_transaction(
        &self,
        request: Request<proto::BeginRequest>,
    ) -> Result<Response<proto::BeginResponse>, Status> {
        let req = request.into_inner();
        self.validate_session(&req.session_id).await?;

        let session = SessionHandle(req.session_id.clone());
        let mode = proto::TransactionMode::try_from(req.mode)
            .unwrap_or(proto::TransactionMode::ReadWrite);

        match self.backend.begin_transaction(&session, mode).await {
            Ok(handle) => {
                let tx_id = handle.0.clone();

                if let Err(e) = self
                    .transactions
                    .register(&tx_id, &req.session_id, mode)
                    .await
                {
                    // Roll back the backend transaction if we can't register it
                    let _ = self.backend.rollback(&session, &handle).await;
                    return Ok(Response::new(proto::BeginResponse {
                        transaction_id: String::new(),
                        status: Some(gql_status::error(
                            gql_status::ACTIVE_TRANSACTION,
                            e.to_string(),
                        )),
                    }));
                }

                self.sessions
                    .set_active_transaction(&req.session_id, Some(tx_id.clone()))
                    .await
                    .ok();

                Ok(Response::new(proto::BeginResponse {
                    transaction_id: tx_id,
                    status: Some(gql_status::success()),
                }))
            }
            Err(err) => {
                let status = match err.gql_status() {
                    Some(s) => s.clone(),
                    None => gql_status::error(
                        gql_status::ACTIVE_TRANSACTION,
                        err.to_string(),
                    ),
                };
                Ok(Response::new(proto::BeginResponse {
                    transaction_id: String::new(),
                    status: Some(status),
                }))
            }
        }
    }

    async fn commit(
        &self,
        request: Request<proto::CommitRequest>,
    ) -> Result<Response<proto::CommitResponse>, Status> {
        let req = request.into_inner();
        self.validate_session(&req.session_id).await?;

        if let Err(e) = self
            .transactions
            .validate(&req.transaction_id, &req.session_id)
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

        match self.backend.commit(&session, &transaction).await {
            Ok(()) => {
                self.transactions.remove(&req.transaction_id).await.ok();
                self.sessions
                    .set_active_transaction(&req.session_id, None)
                    .await
                    .ok();

                Ok(Response::new(proto::CommitResponse {
                    status: Some(gql_status::success()),
                }))
            }
            Err(err) => {
                let status = match err.gql_status() {
                    Some(s) => s.clone(),
                    None => gql_status::error(
                        gql_status::TRANSACTION_ROLLBACK,
                        err.to_string(),
                    ),
                };
                Ok(Response::new(proto::CommitResponse {
                    status: Some(status),
                }))
            }
        }
    }

    async fn rollback(
        &self,
        request: Request<proto::RollbackRequest>,
    ) -> Result<Response<proto::RollbackResponse>, Status> {
        let req = request.into_inner();
        self.validate_session(&req.session_id).await?;

        if let Err(e) = self
            .transactions
            .validate(&req.transaction_id, &req.session_id)
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

        match self.backend.rollback(&session, &transaction).await {
            Ok(()) => {
                self.transactions.remove(&req.transaction_id).await.ok();
                self.sessions
                    .set_active_transaction(&req.session_id, None)
                    .await
                    .ok();

                Ok(Response::new(proto::RollbackResponse {
                    status: Some(gql_status::success()),
                }))
            }
            Err(err) => {
                let status = match err.gql_status() {
                    Some(s) => s.clone(),
                    None => gql_status::error(
                        gql_status::TRANSACTION_ROLLBACK,
                        err.to_string(),
                    ),
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
struct ResultStreamAdapter {
    inner: Pin<Box<dyn ResultStream>>,
}

impl Stream for ResultStreamAdapter {
    type Item = Result<proto::ExecuteResponse, Status>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            std::task::Poll::Ready(Some(Ok(frame))) => {
                let response = match frame {
                    ResultFrame::Header(h) => proto::ExecuteResponse {
                        frame: Some(proto::execute_response::Frame::Header(h)),
                    },
                    ResultFrame::Batch(b) => proto::ExecuteResponse {
                        frame: Some(proto::execute_response::Frame::RowBatch(b)),
                    },
                    ResultFrame::Summary(s) => proto::ExecuteResponse {
                        frame: Some(proto::execute_response::Frame::Summary(s)),
                    },
                };
                std::task::Poll::Ready(Some(Ok(response)))
            }
            std::task::Poll::Ready(Some(Err(err))) => {
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
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
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
