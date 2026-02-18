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
            super::backend::ResetTarget::All => *state = SessionState::default(),
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
