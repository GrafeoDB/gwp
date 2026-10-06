//! Server-side session state tracking.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time::Instant;

use super::SessionProperty;

/// Tracks the mutable state for a single session.
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Current schema.
    pub schema: Option<String>,
    /// Current graph.
    pub graph: Option<String>,
    /// Timezone offset in minutes.
    pub time_zone_offset_minutes: i32,
    /// Session parameters.
    pub parameters: HashMap<String, crate::types::Value>,
    /// Active transaction ID, if any.
    pub active_transaction: Option<String>,
    /// Timestamp of last activity for idle detection.
    pub last_activity: Instant,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            schema: None,
            graph: None,
            time_zone_offset_minutes: 0,
            parameters: HashMap::new(),
            active_transaction: None,
            last_activity: Instant::now(),
        }
    }
}

/// Manages session state for all active sessions.
#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    max_sessions: Option<usize>,
}

impl SessionManager {
    /// Create a new session manager with no capacity limit.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_sessions: None,
        }
    }

    /// Create a session manager with a maximum number of concurrent sessions.
    #[must_use]
    pub fn with_capacity(max_sessions: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_sessions: Some(max_sessions),
        }
    }

    /// Register a new session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session limit has been reached.
    pub async fn register(&self, session_id: &str) -> Result<(), crate::error::GqlError> {
        let mut sessions = self.sessions.write().await;
        if let Some(max) = self.max_sessions {
            if sessions.len() >= max {
                return Err(crate::error::GqlError::Session(
                    "session limit reached".to_owned(),
                ));
            }
        }
        sessions.insert(session_id.to_owned(), SessionState::default());
        tracing::info!(session_id, "session registered");
        Ok(())
    }

    /// Remove a session.
    pub async fn remove(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        let removed = sessions.remove(session_id).is_some();
        if removed {
            tracing::info!(session_id, "session removed");
        }
        removed
    }

    /// Check if a session exists.
    pub async fn exists(&self, session_id: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(session_id)
    }

    /// Update the last-activity timestamp for a session.
    pub async fn touch(&self, session_id: &str) {
        if let Some(state) = self.sessions.write().await.get_mut(session_id) {
            state.last_activity = Instant::now();
        }
    }

    /// Remove sessions that have been idle longer than `max_idle`.
    ///
    /// Returns the IDs of reaped sessions.
    pub async fn reap_idle(&self, max_idle: std::time::Duration) -> Vec<String> {
        let mut sessions = self.sessions.write().await;
        let now = Instant::now();
        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| now.duration_since(s.last_activity) > max_idle)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &expired {
            sessions.remove(id);
        }
        if !expired.is_empty() {
            tracing::info!(count = expired.len(), "idle sessions reaped");
        }
        expired
    }

    /// Apply a session property.
    ///
    /// # Errors
    ///
    /// Returns an error if the session does not exist.
    pub async fn configure(
        &self,
        session_id: &str,
        property: &SessionProperty,
    ) -> Result<(), crate::error::GqlError> {
        let mut sessions = self.sessions.write().await;
        let state = sessions.get_mut(session_id).ok_or_else(|| {
            crate::error::GqlError::Session(format!("session {session_id} not found"))
        })?;

        match property {
            SessionProperty::Schema(s) => state.schema = Some(s.clone()),
            SessionProperty::Graph(g) => state.graph = Some(g.clone()),
            SessionProperty::TimeZone(offset) => state.time_zone_offset_minutes = *offset,
            SessionProperty::Parameter { name, value } => {
                state.parameters.insert(name.clone(), value.clone());
            }
        }
        Ok(())
    }

    /// Reset session state.
    ///
    /// # Errors
    ///
    /// Returns an error if the session does not exist.
    pub async fn reset(
        &self,
        session_id: &str,
        target: super::backend::ResetTarget,
    ) -> Result<(), crate::error::GqlError> {
        let mut sessions = self.sessions.write().await;
        let state = sessions.get_mut(session_id).ok_or_else(|| {
            crate::error::GqlError::Session(format!("session {session_id} not found"))
        })?;

        match target {
            // A session reset changes session characteristics only
            // (ISO/IEC 39075 sec 7.2): an active transaction stays active
            // and must stay tracked.
            super::backend::ResetTarget::All => {
                *state = SessionState {
                    active_transaction: state.active_transaction.take(),
                    ..SessionState::default()
                };
            }
            super::backend::ResetTarget::Schema => state.schema = None,
            super::backend::ResetTarget::Graph => state.graph = None,
            super::backend::ResetTarget::TimeZone => state.time_zone_offset_minutes = 0,
            super::backend::ResetTarget::Parameters => state.parameters.clear(),
        }
        Ok(())
    }

    /// Get the active transaction for a session.
    pub async fn active_transaction(&self, session_id: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .and_then(|s| s.active_transaction.clone())
    }

    /// Set the active transaction for a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session does not exist.
    pub async fn set_active_transaction(
        &self,
        session_id: &str,
        transaction_id: Option<String>,
    ) -> Result<(), crate::error::GqlError> {
        let mut sessions = self.sessions.write().await;
        let state = sessions.get_mut(session_id).ok_or_else(|| {
            crate::error::GqlError::Session(format!("session {session_id} not found"))
        })?;
        state.active_transaction = transaction_id;
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::ResetTarget;
    use crate::types::Value;

    async fn state(manager: &SessionManager, session_id: &str) -> SessionState {
        manager.sessions.read().await[session_id].clone()
    }

    async fn configured() -> SessionManager {
        let manager = SessionManager::new();
        manager.register("s1").await.unwrap();
        for property in [
            SessionProperty::Schema("analytics".to_owned()),
            SessionProperty::Graph("social".to_owned()),
            SessionProperty::TimeZone(-330),
            SessionProperty::Parameter {
                name: "limit".to_owned(),
                value: Value::Integer(10),
            },
            SessionProperty::Parameter {
                name: "label".to_owned(),
                value: Value::String("Person".to_owned()),
            },
        ] {
            manager.configure("s1", &property).await.unwrap();
        }
        manager
            .set_active_transaction("s1", Some("tx1".to_owned()))
            .await
            .unwrap();
        manager
    }

    #[tokio::test]
    async fn configure_applies_every_property() {
        let manager = configured().await;
        let s = state(&manager, "s1").await;
        assert_eq!(s.schema.as_deref(), Some("analytics"));
        assert_eq!(s.graph.as_deref(), Some("social"));
        assert_eq!(s.time_zone_offset_minutes, -330);
        assert_eq!(s.parameters.len(), 2);
        assert_eq!(s.parameters["limit"], Value::Integer(10));

        // Setting a parameter again replaces its value.
        manager
            .configure(
                "s1",
                &SessionProperty::Parameter {
                    name: "limit".to_owned(),
                    value: Value::Null,
                },
            )
            .await
            .unwrap();
        let s = state(&manager, "s1").await;
        assert_eq!(s.parameters.len(), 2);
        assert_eq!(s.parameters["limit"], Value::Null);
    }

    #[tokio::test]
    async fn reset_targets_one_characteristic_each() {
        let manager = configured().await;

        manager.reset("s1", ResetTarget::Schema).await.unwrap();
        let s = state(&manager, "s1").await;
        assert_eq!(s.schema, None);
        assert_eq!(s.graph.as_deref(), Some("social"));

        manager.reset("s1", ResetTarget::Graph).await.unwrap();
        let s = state(&manager, "s1").await;
        assert_eq!(s.graph, None);
        assert_eq!(s.time_zone_offset_minutes, -330);

        manager.reset("s1", ResetTarget::TimeZone).await.unwrap();
        let s = state(&manager, "s1").await;
        assert_eq!(s.time_zone_offset_minutes, 0);
        assert_eq!(s.parameters.len(), 2);

        manager.reset("s1", ResetTarget::Parameters).await.unwrap();
        let s = state(&manager, "s1").await;
        assert!(s.parameters.is_empty());
        assert_eq!(s.active_transaction.as_deref(), Some("tx1"));
    }

    #[tokio::test]
    async fn reset_all_keeps_the_active_transaction() {
        let manager = configured().await;

        manager.reset("s1", ResetTarget::All).await.unwrap();

        let s = state(&manager, "s1").await;
        assert_eq!(s.schema, None);
        assert_eq!(s.graph, None);
        assert_eq!(s.time_zone_offset_minutes, 0);
        assert!(s.parameters.is_empty());
        // Regression: a full reset used to forget the active transaction.
        assert_eq!(s.active_transaction.as_deref(), Some("tx1"));
        assert_eq!(
            manager.active_transaction("s1").await.as_deref(),
            Some("tx1")
        );
    }

    #[tokio::test]
    async fn unknown_session_is_an_error() {
        let manager = SessionManager::new();
        assert!(
            manager
                .configure("missing", &SessionProperty::TimeZone(60))
                .await
                .is_err()
        );
        assert!(manager.reset("missing", ResetTarget::All).await.is_err());
        assert!(
            manager
                .set_active_transaction("missing", None)
                .await
                .is_err()
        );
        assert_eq!(manager.active_transaction("missing").await, None);
        assert!(!manager.remove("missing").await);
    }

    #[tokio::test]
    async fn capacity_is_enforced_and_freed_on_remove() {
        let manager = SessionManager::with_capacity(1);
        manager.register("s1").await.unwrap();
        assert!(manager.register("s2").await.is_err());
        assert!(manager.remove("s1").await);
        manager.register("s2").await.unwrap();
    }
}
