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
    /// Set once a commit or rollback has claimed the transaction. A
    /// terminating transaction accepts no statements and no second
    /// commit or rollback, and still blocks a new begin on its session.
    terminating: bool,
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
    /// # Errors
    ///
    /// Returns an error if the session already has a transaction (including
    /// one that is still being committed or rolled back), or if the
    /// transaction id is already in use.
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
        if txns.contains_key(transaction_id) {
            return Err(GqlError::Transaction(format!(
                "transaction {transaction_id} already exists"
            )));
        }

        txns.insert(
            transaction_id.to_owned(),
            TransactionState {
                session_id: session_id.to_owned(),
                mode,
                terminating: false,
            },
        );
        Ok(())
    }

    /// Remove a transaction (on commit or rollback).
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction does not exist.
    pub async fn remove(&self, transaction_id: &str) -> Result<TransactionState, GqlError> {
        let mut txns = self.transactions.write().await;
        txns.remove(transaction_id)
            .ok_or_else(|| GqlError::Transaction(format!("transaction {transaction_id} not found")))
    }

    /// Validate that a transaction exists, belongs to the given session and
    /// is not being committed or rolled back.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction does not exist, belongs to another
    /// session, or is already terminating.
    pub async fn validate(&self, transaction_id: &str, session_id: &str) -> Result<(), GqlError> {
        let txns = self.transactions.read().await;
        Self::check(txns.get(transaction_id), transaction_id, session_id)
    }

    /// Claim a transaction for commit or rollback.
    ///
    /// Validates like [`validate`](Self::validate) and marks the transaction
    /// as terminating in the same step, so that a concurrent commit, rollback
    /// or statement on the same transaction is rejected. Remove the
    /// transaction with [`remove`](Self::remove) once the backend call is done.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction does not exist, belongs to another
    /// session, or is already terminating.
    pub async fn begin_termination(
        &self,
        transaction_id: &str,
        session_id: &str,
    ) -> Result<(), GqlError> {
        let mut txns = self.transactions.write().await;
        let state = txns.get_mut(transaction_id);
        Self::check(state.as_deref(), transaction_id, session_id)?;
        if let Some(state) = state {
            state.terminating = true;
        }
        Ok(())
    }

    /// Returns `true` if the session has a transaction, including one that
    /// is still being committed or rolled back.
    pub async fn has_transaction(&self, session_id: &str) -> bool {
        let txns = self.transactions.read().await;
        txns.values().any(|t| t.session_id == session_id)
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

    /// Remove all transactions for a session and return the ones that still
    /// need a rollback: a terminating transaction is already being committed
    /// or rolled back by another request.
    pub(crate) async fn take_for_session(&self, session_id: &str) -> Vec<String> {
        let mut txns = self.transactions.write().await;
        let mut to_roll_back = Vec::new();
        txns.retain(|id, state| {
            if state.session_id != session_id {
                return true;
            }
            if !state.terminating {
                to_roll_back.push(id.clone());
            }
            false
        });
        to_roll_back
    }

    fn check(
        state: Option<&TransactionState>,
        transaction_id: &str,
        session_id: &str,
    ) -> Result<(), GqlError> {
        match state {
            Some(state) if state.session_id != session_id => Err(GqlError::Transaction(
                "transaction does not belong to this session".to_owned(),
            )),
            Some(state) if state.terminating => Err(GqlError::Transaction(format!(
                "transaction {transaction_id} is already being committed or rolled back"
            ))),
            Some(_) => Ok(()),
            None => Err(GqlError::Transaction(format!(
                "transaction {transaction_id} not found"
            ))),
        }
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
    async fn duplicate_transaction_id_is_rejected() {
        let tm = TransactionManager::new();
        tm.register("tx1", "sess1", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();

        // Another session must not take over an existing transaction id.
        let result = tm
            .register("tx1", "sess2", proto::TransactionMode::ReadWrite)
            .await;
        assert!(result.is_err());
        assert!(tm.validate("tx1", "sess1").await.is_ok());
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

    #[tokio::test]
    async fn termination_is_claimed_once() {
        let tm = TransactionManager::new();
        tm.register("tx1", "sess1", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();

        tm.begin_termination("tx1", "sess1").await.unwrap();

        // A second commit or rollback, and any statement, is rejected.
        assert!(tm.begin_termination("tx1", "sess1").await.is_err());
        assert!(tm.validate("tx1", "sess1").await.is_err());

        // The session still counts as having a transaction until removal.
        assert!(tm.has_transaction("sess1").await);
        assert!(
            tm.register("tx2", "sess1", proto::TransactionMode::ReadWrite)
                .await
                .is_err()
        );

        tm.remove("tx1").await.unwrap();
        assert!(!tm.has_transaction("sess1").await);
        tm.register("tx2", "sess1", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn begin_termination_checks_owner() {
        let tm = TransactionManager::new();
        tm.register("tx1", "sess1", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();

        assert!(tm.begin_termination("tx1", "sess2").await.is_err());
        assert!(tm.begin_termination("missing", "sess1").await.is_err());
        // The failed claims did not mark the transaction.
        assert!(tm.validate("tx1", "sess1").await.is_ok());
    }

    #[tokio::test]
    async fn take_for_session_skips_terminating() {
        let tm = TransactionManager::new();
        tm.register("tx1", "sess1", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();
        tm.register("tx2", "sess2", proto::TransactionMode::ReadWrite)
            .await
            .unwrap();
        tm.begin_termination("tx2", "sess2").await.unwrap();

        assert_eq!(tm.take_for_session("sess1").await, vec!["tx1"]);
        assert!(tm.take_for_session("sess2").await.is_empty());
        assert!(!tm.has_transaction("sess1").await);
        assert!(!tm.has_transaction("sess2").await);
    }
}
