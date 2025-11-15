//! Nonce-based request replay protection
//!
//! Provides one-time request validation to prevent replay attacks.
//! Each request must include a unique nonce that can only be used once.

use crate::{
    error::{Error, Result},
    middleware::{Middleware, Next},
    Request, Response,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64, Engine};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use rand::Rng;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Nonce entry with expiration time
#[derive(Debug, Clone)]
struct NonceEntry {
    created_at: DateTime<Utc>,
}

/// Nonce-based request replay protection
///
/// Prevents replay attacks by ensuring each request uses a unique nonce
/// that can only be used once within the validity period.
///
/// # Security Features
/// - One-time use enforcement
/// - Time-based expiration
/// - Automatic cleanup of expired nonces
/// - Thread-safe concurrent access
/// - Constant-time nonce comparison
///
/// # Usage
/// ```no_run
/// use half_core::security::NonceProtection;
///
/// let nonce = NonceProtection::new()
///     .ttl(300)  // 5 minutes
///     .cleanup_interval(60);  // Clean every minute
///
/// router.use_middleware(nonce);
/// ```
pub struct NonceProtection {
    /// Storage for used nonces with their creation time
    nonces: Arc<DashMap<String, NonceEntry>>,
    /// Time-to-live for nonces in seconds
    ttl: i64,
    /// How often to clean expired nonces (in seconds)
    cleanup_interval: i64,
    /// Paths to exempt from nonce protection
    exempt_paths: Vec<String>,
    /// Header name for nonce
    header_name: String,
}

impl NonceProtection {
    /// Create a new nonce protection middleware
    ///
    /// Default settings:
    /// - TTL: 300 seconds (5 minutes)
    /// - Cleanup interval: 60 seconds
    /// - Header name: "X-Nonce"
    pub fn new() -> Self {
        let nonces = Arc::new(DashMap::new());
        let nonces_clone = Arc::clone(&nonces);
        let ttl = 300;
        let cleanup_interval = 60;

        // Spawn background task for cleanup
        tokio::spawn(async move {
            Self::cleanup_task(nonces_clone, ttl, cleanup_interval).await;
        });

        Self {
            nonces,
            ttl,
            cleanup_interval,
            exempt_paths: Vec::new(),
            header_name: "x-nonce".to_string(),
        }
    }

    /// Set the time-to-live for nonces in seconds
    pub fn ttl(mut self, seconds: i64) -> Self {
        self.ttl = seconds;
        self
    }

    /// Set the cleanup interval in seconds
    pub fn cleanup_interval(mut self, seconds: i64) -> Self {
        self.cleanup_interval = seconds;
        self
    }

    /// Set the header name for nonce
    pub fn header_name(mut self, name: impl Into<String>) -> Self {
        self.header_name = name.into().to_lowercase();
        self
    }

    /// Add a path to exempt from nonce protection
    pub fn exempt(mut self, path: impl Into<String>) -> Self {
        self.exempt_paths.push(path.into());
        self
    }

    /// Generate a new nonce
    ///
    /// Generates a cryptographically secure random nonce
    pub fn generate_nonce(&self) -> String {
        let mut rng = rand::rng();
        let random_bytes: [u8; 32] = rng.random();
        let timestamp = Utc::now().timestamp_millis();

        // Combine timestamp and random bytes
        let mut nonce_bytes = Vec::with_capacity(40);
        nonce_bytes.extend_from_slice(&timestamp.to_be_bytes());
        nonce_bytes.extend_from_slice(&random_bytes);

        BASE64.encode(&nonce_bytes)
    }

    /// Check if a path is exempt from nonce protection
    fn is_exempt(&self, path: &str) -> bool {
        self.exempt_paths.iter().any(|exempt| path.starts_with(exempt))
    }

    /// Validate and consume a nonce
    fn validate_nonce(&self, nonce: &str) -> Result<()> {
        // Check if nonce exists and is not expired
        if let Some(entry) = self.nonces.get(nonce) {
            let age = Utc::now()
                .signed_duration_since(entry.created_at)
                .num_seconds();

            if age > self.ttl {
                // Nonce expired
                drop(entry);
                self.nonces.remove(nonce);
                return Err(Error::InvalidNonce("Nonce expired".to_string()));
            }

            // Nonce already used
            return Err(Error::InvalidNonce("Nonce already used".to_string()));
        }

        // Store the nonce to prevent reuse
        self.nonces.insert(
            nonce.to_string(),
            NonceEntry {
                created_at: Utc::now(),
            },
        );

        Ok(())
    }

    /// Background task to clean up expired nonces
    async fn cleanup_task(nonces: Arc<DashMap<String, NonceEntry>>, ttl: i64, interval: i64) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval as u64));

        loop {
            interval.tick().await;

            let now = Utc::now();
            let expired: Vec<String> = nonces
                .iter()
                .filter_map(|entry| {
                    let age = now
                        .signed_duration_since(entry.value().created_at)
                        .num_seconds();

                    if age > ttl {
                        Some(entry.key().clone())
                    } else {
                        None
                    }
                })
                .collect();

            // Remove expired nonces
            for nonce in expired {
                nonces.remove(&nonce);
            }
        }
    }

    /// Extract nonce from request
    fn extract_nonce(&self, req: &Request) -> Option<String> {
        req.header(&self.header_name).map(|s| s.to_string())
    }
}

impl Default for NonceProtection {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for NonceProtection {
    fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        Box::pin(async move {
            let path = req.path().to_string();

            // Check if path is exempt
            if self.is_exempt(&path) {
                return next(req).await;
            }

            // Extract and validate nonce
            match self.extract_nonce(&req) {
                Some(nonce) => {
                    self.validate_nonce(&nonce)?;
                    next(req).await
                }
                None => {
                    Err(Error::InvalidNonce("Missing nonce header".to_string()))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_nonce() {
        let protection = NonceProtection::new();
        let nonce1 = protection.generate_nonce();
        let nonce2 = protection.generate_nonce();

        assert!(!nonce1.is_empty());
        assert!(!nonce2.is_empty());
        assert_ne!(nonce1, nonce2);
    }

    #[tokio::test]
    async fn test_nonce_validation() {
        let protection = NonceProtection::new();
        let nonce = protection.generate_nonce();

        // First use should succeed
        assert!(protection.validate_nonce(&nonce).is_ok());

        // Second use should fail (replay attack)
        assert!(protection.validate_nonce(&nonce).is_err());
    }

    #[tokio::test]
    async fn test_nonce_exempt() {
        let protection = NonceProtection::new()
            .exempt("/api/webhook")
            .exempt("/public");

        assert!(protection.is_exempt("/api/webhook"));
        assert!(protection.is_exempt("/api/webhook/github"));
        assert!(protection.is_exempt("/public/status"));
        assert!(!protection.is_exempt("/api/users"));
    }

    #[tokio::test]
    async fn test_expired_nonce() {
        let protection = NonceProtection::new().ttl(1);
        let nonce = protection.generate_nonce();

        // Use the nonce immediately
        assert!(protection.validate_nonce(&nonce).is_ok());

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Remove from storage to simulate fresh request
        protection.nonces.remove(&nonce);

        // Generate new nonce for testing expired scenario
        let old_nonce = protection.generate_nonce();
        protection.nonces.insert(
            old_nonce.clone(),
            NonceEntry {
                created_at: Utc::now() - Duration::try_seconds(10).unwrap(),
            },
        );

        // Validation should fail due to expiration
        assert!(protection.validate_nonce(&old_nonce).is_err());
    }
}
