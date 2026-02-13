//! Transaction state tracking and lifecycle management.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::error::GqlError;
use crate::proto;

/// State of an active transaction.
#[derive(Debug, Clone)]
pub struct TransactionState {
    /// Session that owns this transaction.
    pub session_id: String,
    /// Transaction access mode.
    pub mode: proto::TransactionMode,
}

/// Manages transaction state across all sessions.
///
/// Enforces the GQL constraint that at most one transaction
/// can be active per session.
#[derive(Debug, Clone)]
pub struct TransactionManager {
    transactions: Arc<RwLock<HashMap<String, TransactionState>>>,
}

impl TransactionManager {
    /// Create a new transaction manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new transaction for a session.
    ///
    /// Returns an error if the session already has an active transaction.
    pub async fn register(
        &self,
        transaction_id: &str,
        session_id: &str,
        mode: proto::TransactionMode,
    ) -> Result<(), GqlError> {
        let mut txns = self.transactions.write().await;

        // Check no active transaction for this session
        let has_active = txns.values().any(|t| t.session_id == session_id);
        if has_active {
            return Err(GqlError::Transaction(
                "session already has an active transaction".to_owned(),
            ));
        }

        txns.insert(
            transaction_id.to_owned(),
            TransactionState {
                session_id: session_id.to_owned(),
                mode,
            },
        );
        Ok(())
    }

    /// Remove a transaction (on commit or rollback).
    pub async fn remove(&self, transaction_id: &str) -> Result<TransactionState, GqlError> {
        let mut txns = self.transactions.write().await;
        txns.remove(transaction_id).ok_or_else(|| {
            GqlError::Transaction(format!("transaction {transaction_id} not found"))
        })
    }

    /// Validate that a transaction exists and belongs to the given session.
    pub async fn validate(
        &self,
        transaction_id: &str,
        session_id: &str,
    ) -> Result<(), GqlError> {
        let txns = self.transactions.read().await;
        match txns.get(transaction_id) {
            Some(state) if state.session_id == session_id => Ok(()),
            Some(_) => Err(GqlError::Transaction(
                "transaction does not belong to this session".to_owned(),
            )),
            None => Err(GqlError::Transaction(format!(
                "transaction {transaction_id} not found"
            ))),
        }
    }

    /// Remove all transactions for a session (on session close).
    pub async fn remove_for_session(&self, session_id: &str) -> Vec<String> {
        let mut txns = self.transactions.write().await;
        let to_remove: Vec<String> = txns
            .iter()
            .filter(|(_, state)| state.session_id == session_id)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &to_remove {
            txns.remove(id);
        }
        to_remove
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_remove() {
        let tm = TransactionManager::new();
        tm.register("tx1", "sess1", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();

        let state = tm.remove("tx1").await.unwrap();
        assert_eq!(state.session_id, "sess1");
    }

    #[tokio::test]
    async fn double_begin_fails() {
        let tm = TransactionManager::new();
        tm.register("tx1", "sess1", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();

        let result = tm
            .register("tx2", "sess1", proto::TransactionMode::ReadOnly)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn validate_wrong_session() {
        let tm = TransactionManager::new();
        tm.register("tx1", "sess1", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();

        let result = tm.validate("tx1", "sess2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn remove_for_session() {
        let tm = TransactionManager::new();
        tm.register("tx1", "sess1", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();

        let removed = tm.remove_for_session("sess1").await;
        assert_eq!(removed, vec!["tx1"]);

        let result = tm.validate("tx1", "sess1").await;
        assert!(result.is_err());
    }
}
