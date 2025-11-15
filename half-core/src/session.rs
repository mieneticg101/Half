//! Session management for user sessions
//!
//! Provides secure session management with automatic cleanup and TTL.

// Session management doesn't use error types directly
use base64::Engine;
use dashmap::DashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Session ID type
pub type SessionId = String;

/// Session data
#[derive(Debug, Clone)]
pub struct Session {
    /// Session ID
    pub id: SessionId,
    /// Session data
    data: HashMap<String, String>,
    /// Creation time
    created_at: Instant,
    /// Last access time
    last_accessed: Instant,
    /// Time-to-live
    ttl: Duration,
}

impl Session {
    /// Create a new session
    fn new(id: SessionId, ttl: Duration) -> Self {
        Self {
            id,
            data: HashMap::new(),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            ttl,
        }
    }

    /// Get a value from the session
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Set a value in the session
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
        self.touch();
    }

    /// Remove a value from the session
    pub fn remove(&mut self, key: &str) -> Option<String> {
        let result = self.data.remove(key);
        self.touch();
        result
    }

    /// Check if a key exists
    pub fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
        self.touch();
    }

    /// Touch the session (update last accessed time)
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }

    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        self.last_accessed.elapsed() > self.ttl
    }

    /// Get session age
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get time since last access
    pub fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed()
    }
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Time-to-live for sessions
    pub ttl: Duration,
    /// Cleanup interval
    pub cleanup_interval: Duration,
    /// Cookie name
    pub cookie_name: String,
    /// Cookie secure flag
    pub secure: bool,
    /// Cookie http_only flag
    pub http_only: bool,
    /// Cookie same_site
    pub same_site: crate::response::SameSite,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            cookie_name: "session_id".to_string(),
            secure: true,
            http_only: true,
            same_site: crate::response::SameSite::Lax,
        }
    }
}

/// Session store
///
/// Thread-safe session storage with automatic cleanup of expired sessions.
///
/// # Example
/// ```
/// use half_core::session::{SessionStore, SessionConfig};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     let store = SessionStore::new(SessionConfig {
///         ttl: Duration::from_secs(3600),
///         ..Default::default()
///     });
///
///     // Create a new session
///     let session_id = store.create_session();
///
///     // Get and modify session
///     if let Some(mut session) = store.get_session(&session_id) {
///         session.set("user_id", "123");
///         session.set("username", "alice");
///         store.update_session(session);
///     }
///
///     // Retrieve session data
///     if let Some(session) = store.get_session(&session_id) {
///         let user_id = session.get("user_id");
///         assert_eq!(user_id.map(|s| s.as_str()), Some("123"));
///     }
///
///     // Destroy session
///     store.destroy_session(&session_id);
/// }
/// ```
pub struct SessionStore {
    sessions: Arc<DashMap<SessionId, Session>>,
    config: SessionConfig,
}

impl SessionStore {
    /// Create a new session store
    pub fn new(config: SessionConfig) -> Self {
        let store = Self {
            sessions: Arc::new(DashMap::new()),
            config,
        };

        // Start cleanup task
        let sessions_clone = Arc::clone(&store.sessions);
        let cleanup_interval = store.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                Self::cleanup_expired_sessions(&sessions_clone);
            }
        });

        store
    }

    /// Generate a new session ID
    fn generate_session_id() -> SessionId {
        let mut rng = rand::rng();
        let random_bytes: [u8; 32] = rng.random();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
    }

    /// Create a new session
    pub fn create_session(&self) -> SessionId {
        let session_id = Self::generate_session_id();
        let session = Session::new(session_id.clone(), self.config.ttl);
        self.sessions.insert(session_id.clone(), session);
        session_id
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        if let Some(mut entry) = self.sessions.get_mut(session_id) {
            if entry.is_expired() {
                drop(entry);
                self.sessions.remove(session_id);
                return None;
            }
            entry.touch();
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Update a session
    pub fn update_session(&self, session: Session) {
        if !session.is_expired() {
            self.sessions.insert(session.id.clone(), session);
        }
    }

    /// Destroy a session
    pub fn destroy_session(&self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    /// Check if a session exists
    pub fn has_session(&self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get(session_id) {
            !session.is_expired()
        } else {
            false
        }
    }

    /// Get session count
    pub fn count(&self) -> usize {
        self.sessions.len()
    }

    /// Cleanup expired sessions
    fn cleanup_expired_sessions(sessions: &Arc<DashMap<SessionId, Session>>) {
        let expired: Vec<SessionId> = sessions
            .iter()
            .filter(|entry| entry.value().is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        for session_id in expired {
            sessions.remove(&session_id);
        }
    }

    /// Clear all sessions
    pub fn clear_all(&self) {
        self.sessions.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new(SessionConfig::default())
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total sessions
    pub total: usize,
    /// Active sessions (non-expired)
    pub active: usize,
    /// Average session age in seconds
    pub average_age_seconds: f64,
}

impl SessionStore {
    /// Get session statistics
    pub fn stats(&self) -> SessionStats {
        let mut total_age = Duration::ZERO;
        let mut active_count = 0;

        for entry in self.sessions.iter() {
            if !entry.is_expired() {
                active_count += 1;
                total_age += entry.age();
            }
        }

        let average_age_seconds = if active_count > 0 {
            total_age.as_secs_f64() / active_count as f64
        } else {
            0.0
        };

        SessionStats {
            total: self.sessions.len(),
            active: active_count,
            average_age_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = Session::new("test_id".to_string(), Duration::from_secs(3600));
        assert_eq!(session.id, "test_id");
        assert!(session.data.is_empty());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_set_get() {
        let mut session = Session::new("test".to_string(), Duration::from_secs(3600));

        session.set("key1", "value1");
        session.set("key2", "value2");

        assert_eq!(session.get("key1"), Some(&"value1".to_string()));
        assert_eq!(session.get("key2"), Some(&"value2".to_string()));
        assert_eq!(session.get("key3"), None);
    }

    #[test]
    fn test_session_remove() {
        let mut session = Session::new("test".to_string(), Duration::from_secs(3600));

        session.set("key", "value");
        assert_eq!(session.remove("key"), Some("value".to_string()));
        assert_eq!(session.get("key"), None);
    }

    #[test]
    fn test_session_has() {
        let mut session = Session::new("test".to_string(), Duration::from_secs(3600));

        session.set("key", "value");
        assert!(session.has("key"));
        assert!(!session.has("nonexistent"));
    }

    #[test]
    fn test_session_clear() {
        let mut session = Session::new("test".to_string(), Duration::from_secs(3600));

        session.set("key1", "value1");
        session.set("key2", "value2");

        session.clear();
        assert!(session.data.is_empty());
    }

    #[tokio::test]
    async fn test_session_store_create() {
        let store = SessionStore::new(SessionConfig::default());

        let session_id = store.create_session();
        assert!(!session_id.is_empty());
        assert!(store.has_session(&session_id));
    }

    #[tokio::test]
    async fn test_session_store_get_update() {
        let store = SessionStore::new(SessionConfig::default());

        let session_id = store.create_session();

        if let Some(mut session) = store.get_session(&session_id) {
            session.set("user", "alice");
            store.update_session(session);
        }

        if let Some(session) = store.get_session(&session_id) {
            assert_eq!(session.get("user"), Some(&"alice".to_string()));
        }
    }

    #[tokio::test]
    async fn test_session_store_destroy() {
        let store = SessionStore::new(SessionConfig::default());

        let session_id = store.create_session();
        assert!(store.has_session(&session_id));

        store.destroy_session(&session_id);
        assert!(!store.has_session(&session_id));
    }

    #[tokio::test]
    async fn test_session_store_count() {
        let store = SessionStore::new(SessionConfig::default());

        let _id1 = store.create_session();
        let _id2 = store.create_session();
        let _id3 = store.create_session();

        assert_eq!(store.count(), 3);
    }

    #[tokio::test]
    async fn test_session_stats() {
        let store = SessionStore::new(SessionConfig::default());

        let _id1 = store.create_session();
        let _id2 = store.create_session();

        let stats = store.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.active, 2);
    }
}
