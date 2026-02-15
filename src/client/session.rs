//! Client-side session wrapper.

use std::collections::HashMap;

use tonic::transport::Channel;

use crate::error::GqlError;
use crate::proto;
use crate::proto::gql_service_client::GqlServiceClient;
use crate::proto::session_service_client::SessionServiceClient;
use crate::types::Value;

use super::result::ResultCursor;
use super::transaction::Transaction;

/// An active session with a GQL server.
///
/// Wraps the handshake response and provides typed methods for
/// executing statements, managing transactions, and configuring
/// session state.
pub struct GqlSession {
    session_id: String,
    session_client: SessionServiceClient<Channel>,
    gql_client: GqlServiceClient<Channel>,
}

impl GqlSession {
    /// Create a new session by performing a handshake.
    pub(crate) async fn new(channel: Channel) -> Result<Self, GqlError> {
        let mut session_client = SessionServiceClient::new(channel.clone());
        let gql_client = GqlServiceClient::new(channel);

        let resp = session_client
            .handshake(proto::HandshakeRequest {
                protocol_version: 1,
                credentials: None,
                client_info: HashMap::new(),
            })
            .await?
            .into_inner();

        Ok(Self {
            session_id: resp.session_id,
            session_client,
            gql_client,
        })
    }

    /// Get the session ID.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Execute a GQL statement and return a cursor over the results.
    ///
    /// # Errors
    ///
    /// Returns an error if the server rejects the request.
    pub async fn execute(
        &mut self,
        statement: &str,
        parameters: HashMap<String, Value>,
    ) -> Result<ResultCursor, GqlError> {
        let proto_params: HashMap<String, proto::Value> = parameters
            .into_iter()
            .map(|(k, v)| (k, proto::Value::from(v)))
            .collect();

        let stream = self
            .gql_client
            .execute(proto::ExecuteRequest {
                session_id: self.session_id.clone(),
                statement: statement.to_owned(),
                parameters: proto_params,
                transaction_id: None,
            })
            .await?
            .into_inner();

        Ok(ResultCursor::new(stream))
    }

    /// Begin an explicit transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be started.
    pub async fn begin_transaction(&mut self) -> Result<Transaction, GqlError> {
        Transaction::begin(
            self.session_id.clone(),
            self.gql_client.clone(),
            proto::TransactionMode::ReadWrite,
        )
        .await
    }

    /// Begin a read-only transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be started.
    pub async fn begin_read_only_transaction(&mut self) -> Result<Transaction, GqlError> {
        Transaction::begin(
            self.session_id.clone(),
            self.gql_client.clone(),
            proto::TransactionMode::ReadOnly,
        )
        .await
    }

    /// Set the current graph for this session.
    ///
    /// # Errors
    ///
    /// Returns an error if the server rejects the configuration.
    pub async fn set_graph(&mut self, graph: &str) -> Result<(), GqlError> {
        self.session_client
            .configure(proto::ConfigureRequest {
                session_id: self.session_id.clone(),
                property: Some(proto::configure_request::Property::Graph(graph.to_owned())),
            })
            .await?;
        Ok(())
    }

    /// Set the current schema for this session.
    ///
    /// # Errors
    ///
    /// Returns an error if the server rejects the configuration.
    pub async fn set_schema(&mut self, schema: &str) -> Result<(), GqlError> {
        self.session_client
            .configure(proto::ConfigureRequest {
                session_id: self.session_id.clone(),
                property: Some(proto::configure_request::Property::Schema(
                    schema.to_owned(),
                )),
            })
            .await?;
        Ok(())
    }

    /// Set the timezone offset for this session.
    ///
    /// # Errors
    ///
    /// Returns an error if the server rejects the configuration.
    pub async fn set_time_zone(&mut self, offset_minutes: i32) -> Result<(), GqlError> {
        self.session_client
            .configure(proto::ConfigureRequest {
                session_id: self.session_id.clone(),
                property: Some(proto::configure_request::Property::TimeZoneOffsetMinutes(
                    offset_minutes,
                )),
            })
            .await?;
        Ok(())
    }

    /// Reset all session state to defaults.
    ///
    /// # Errors
    ///
    /// Returns an error if the server rejects the request.
    pub async fn reset(&mut self) -> Result<(), GqlError> {
        self.session_client
            .reset(proto::ResetRequest {
                session_id: self.session_id.clone(),
                target: proto::ResetTarget::ResetAll.into(),
            })
            .await?;
        Ok(())
    }

    /// Ping the server to check connectivity.
    ///
    /// # Errors
    ///
    /// Returns an error if the server is unreachable.
    pub async fn ping(&mut self) -> Result<i64, GqlError> {
        let resp = self
            .session_client
            .ping(proto::PingRequest {
                session_id: self.session_id.clone(),
            })
            .await?
            .into_inner();

        Ok(resp.timestamp)
    }

    /// Close this session.
    ///
    /// # Errors
    ///
    /// Returns an error if the server rejects the request.
    pub async fn close(mut self) -> Result<(), GqlError> {
        self.session_client
            .close(proto::CloseRequest {
                session_id: self.session_id.clone(),
            })
            .await?;
        Ok(())
    }
}
