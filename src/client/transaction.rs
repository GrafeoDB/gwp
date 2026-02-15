//! Client-side transaction wrapper.

use std::collections::HashMap;

use tonic::transport::Channel;

use crate::error::GqlError;
use crate::proto;
use crate::proto::gql_service_client::GqlServiceClient;
use crate::status;
use crate::types::Value;

use super::result::ResultCursor;

/// An active transaction within a session.
///
/// Provides `execute`, `commit`, and `rollback`. If dropped without
/// committing, the transaction is automatically rolled back.
pub struct Transaction {
    session_id: String,
    id: String,
    client: GqlServiceClient<Channel>,
    committed: bool,
    rolled_back: bool,
}

impl Transaction {
    /// Begin a transaction (called by `GqlSession`).
    pub(crate) async fn begin(
        session_id: String,
        mut client: GqlServiceClient<Channel>,
        mode: proto::TransactionMode,
    ) -> Result<Self, GqlError> {
        let resp = client
            .begin_transaction(proto::BeginRequest {
                session_id: session_id.clone(),
                mode: mode.into(),
            })
            .await?
            .into_inner();

        // Check for GQLSTATUS error
        if let Some(ref s) = resp.status {
            if status::is_exception(&s.code) {
                return Err(GqlError::Status { status: s.clone() });
            }
        }

        if resp.transaction_id.is_empty() {
            return Err(GqlError::Transaction(
                "server returned empty transaction ID".to_owned(),
            ));
        }

        Ok(Self {
            session_id,
            id: resp.transaction_id,
            client,
            committed: false,
            rolled_back: false,
        })
    }

    /// Get the transaction ID.
    #[must_use]
    pub fn transaction_id(&self) -> &str {
        &self.id
    }

    /// Execute a statement within this transaction.
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
            .client
            .execute(proto::ExecuteRequest {
                session_id: self.session_id.clone(),
                statement: statement.to_owned(),
                parameters: proto_params,
                transaction_id: Some(self.id.clone()),
            })
            .await?
            .into_inner();

        Ok(ResultCursor::new(stream))
    }

    /// Commit the transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the commit fails.
    pub async fn commit(mut self) -> Result<(), GqlError> {
        let resp = self
            .client
            .commit(proto::CommitRequest {
                session_id: self.session_id.clone(),
                transaction_id: self.id.clone(),
            })
            .await?
            .into_inner();

        self.committed = true;

        if let Some(ref s) = resp.status {
            if status::is_exception(&s.code) {
                return Err(GqlError::Status { status: s.clone() });
            }
        }

        Ok(())
    }

    /// Roll back the transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the rollback fails.
    pub async fn rollback(mut self) -> Result<(), GqlError> {
        self.do_rollback().await
    }

    /// Internal rollback implementation.
    async fn do_rollback(&mut self) -> Result<(), GqlError> {
        if self.committed || self.rolled_back {
            return Ok(());
        }

        let resp = self
            .client
            .rollback(proto::RollbackRequest {
                session_id: self.session_id.clone(),
                transaction_id: self.id.clone(),
            })
            .await?
            .into_inner();

        self.rolled_back = true;

        if let Some(ref s) = resp.status {
            if status::is_exception(&s.code) {
                return Err(GqlError::Status { status: s.clone() });
            }
        }

        Ok(())
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed && !self.rolled_back {
            // Fire-and-forget rollback on drop.
            // We can't await in drop, so we spawn a task.
            let mut client = self.client.clone();
            let session_id = self.session_id.clone();
            let transaction_id = self.id.clone();
            tokio::spawn(async move {
                let _ = client
                    .rollback(proto::RollbackRequest {
                        session_id,
                        transaction_id,
                    })
                    .await;
            });
        }
    }
}
