//! `SessionService` gRPC implementation.
//!
//! All errors are returned as gRPC status codes - no GQLSTATUS here.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tonic::{Request, Response, Status};

use crate::proto;
use crate::proto::session_service_server::SessionService;

use super::auth::AuthValidator;
use super::backend::{GqlBackend, ResetTarget, SessionConfig, SessionProperty};
use super::{SessionManager, TransactionManager};

/// Implementation of the `SessionService` gRPC service.
pub struct SessionServiceImpl<B: GqlBackend> {
    backend: Arc<B>,
    sessions: SessionManager,
    transactions: TransactionManager,
    auth: Option<Arc<dyn AuthValidator>>,
}

impl<B: GqlBackend> SessionServiceImpl<B> {
    /// Create a new session service.
    pub fn new(
        backend: Arc<B>,
        sessions: SessionManager,
        transactions: TransactionManager,
        auth: Option<Arc<dyn AuthValidator>>,
    ) -> Self {
        Self {
            backend,
            sessions,
            transactions,
            auth,
        }
    }
}

#[tonic::async_trait]
impl<B: GqlBackend> SessionService for SessionServiceImpl<B> {
    async fn handshake(
        &self,
        request: Request<proto::HandshakeRequest>,
    ) -> Result<Response<proto::HandshakeResponse>, Status> {
        let req = request.into_inner();

        if let Some(ref auth) = self.auth {
            match req.credentials {
                Some(ref creds) => {
                    auth.validate(creds)
                        .await
                        .map_err(|_| Status::unauthenticated("invalid credentials"))?;
                }
                None => return Err(Status::unauthenticated("credentials required")),
            }
        }

        let config = SessionConfig {
            protocol_version: req.protocol_version,
            client_info: req.client_info,
        };

        let handle = self
            .backend
            .create_session(&config)
            .await
            .map_err(|e| e.to_grpc_status())?;

        if let Err(e) = self.sessions.register(&handle.0).await {
            let _ = self.backend.close_session(&handle).await;
            return Err(Status::resource_exhausted(e.to_string()));
        }

        Ok(Response::new(proto::HandshakeResponse {
            protocol_version: 1,
            session_id: handle.0,
            server_info: Some(proto::ServerInfo {
                name: "gql-wire-protocol".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                features: Vec::new(),
            }),
            limits: std::collections::HashMap::new(),
        }))
    }

    async fn configure(
        &self,
        request: Request<proto::ConfigureRequest>,
    ) -> Result<Response<proto::ConfigureResponse>, Status> {
        let req = request.into_inner();
        let session_id = &req.session_id;

        if !self.sessions.exists(session_id).await {
            return Err(Status::not_found(format!("session {session_id} not found")));
        }
        self.sessions.touch(session_id).await;

        let property = match req.property {
            Some(proto::configure_request::Property::Schema(s)) => SessionProperty::Schema(s),
            Some(proto::configure_request::Property::Graph(g)) => SessionProperty::Graph(g),
            Some(proto::configure_request::Property::TimeZoneOffsetMinutes(tz)) => {
                SessionProperty::TimeZone(tz)
            }
            Some(proto::configure_request::Property::Parameter(p)) => SessionProperty::Parameter {
                name: p.name,
                value: p
                    .value
                    .map_or(crate::types::Value::Null, crate::types::Value::from),
            },
            None => return Err(Status::invalid_argument("no property specified")),
        };

        self.backend
            .configure_session(&super::SessionHandle(session_id.clone()), property.clone())
            .await
            .map_err(|e| e.to_grpc_status())?;

        self.sessions
            .configure(session_id, &property)
            .await
            .map_err(|e| e.to_grpc_status())?;

        Ok(Response::new(proto::ConfigureResponse {}))
    }

    async fn reset(
        &self,
        request: Request<proto::ResetRequest>,
    ) -> Result<Response<proto::ResetResponse>, Status> {
        let req = request.into_inner();
        let session_id = &req.session_id;

        if !self.sessions.exists(session_id).await {
            return Err(Status::not_found(format!("session {session_id} not found")));
        }
        self.sessions.touch(session_id).await;

        let target = match proto::ResetTarget::try_from(req.target) {
            Ok(proto::ResetTarget::ResetAll) => ResetTarget::All,
            Ok(proto::ResetTarget::ResetSchema) => ResetTarget::Schema,
            Ok(proto::ResetTarget::ResetGraph) => ResetTarget::Graph,
            Ok(proto::ResetTarget::ResetTimeZone) => ResetTarget::TimeZone,
            Ok(proto::ResetTarget::ResetParameters) => ResetTarget::Parameters,
            Err(_) => return Err(Status::invalid_argument("invalid reset target")),
        };

        self.backend
            .reset_session(&super::SessionHandle(session_id.clone()), target)
            .await
            .map_err(|e| e.to_grpc_status())?;

        self.sessions
            .reset(session_id, target)
            .await
            .map_err(|e| e.to_grpc_status())?;

        Ok(Response::new(proto::ResetResponse {}))
    }

    async fn close(
        &self,
        request: Request<proto::CloseRequest>,
    ) -> Result<Response<proto::CloseResponse>, Status> {
        let req = request.into_inner();
        let session_id = &req.session_id;

        if !self.sessions.exists(session_id).await {
            return Err(Status::not_found(format!("session {session_id} not found")));
        }

        // Roll back any active transactions
        let active_txns = self.transactions.remove_for_session(session_id).await;
        for tx_id in &active_txns {
            let _ = self
                .backend
                .rollback(
                    &super::SessionHandle(session_id.clone()),
                    &super::TransactionHandle(tx_id.clone()),
                )
                .await;
        }

        self.backend
            .close_session(&super::SessionHandle(session_id.clone()))
            .await
            .map_err(|e| e.to_grpc_status())?;

        self.sessions.remove(session_id).await;

        Ok(Response::new(proto::CloseResponse {}))
    }

    async fn ping(
        &self,
        request: Request<proto::PingRequest>,
    ) -> Result<Response<proto::PongResponse>, Status> {
        let req = request.into_inner();

        if !self.sessions.exists(&req.session_id).await {
            return Err(Status::not_found(format!(
                "session {} not found",
                req.session_id
            )));
        }
        self.sessions.touch(&req.session_id).await;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));

        Ok(Response::new(proto::PongResponse { timestamp }))
    }
}
