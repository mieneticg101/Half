//! CSRF (Cross-Site Request Forgery) protection
//!
//! Provides token-based CSRF protection for state-changing operations.

use crate::{
    error::{Error, Result},
    middleware::{Middleware, Next},
    Request, Response,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;
use std::future::Future;
use std::pin::Pin;

type HmacSha256 = Hmac<Sha256>;

/// CSRF token
#[derive(Debug, Clone)]
pub struct CsrfToken {
    token: String,
}

impl CsrfToken {
    /// Generate a new CSRF token
    pub fn generate(secret: &[u8]) -> Self {
        let mut rng = rand::rng();
        let random_bytes: [u8; 32] = rng.random();

        // Create HMAC of random bytes with secret
        let mut mac = HmacSha256::new_from_slice(secret)
            .expect("HMAC can take key of any size");
        mac.update(&random_bytes);
        let signature = mac.finalize().into_bytes();

        // Combine random bytes and signature
        let mut token_bytes = Vec::with_capacity(64);
        token_bytes.extend_from_slice(&random_bytes);
        token_bytes.extend_from_slice(&signature);

        // Encode as base64
        let token = BASE64.encode(&token_bytes);

        Self { token }
    }

    /// Verify a CSRF token
    pub fn verify(&self, secret: &[u8]) -> bool {
        // Decode from base64
        let Ok(token_bytes) = BASE64.decode(&self.token) else {
            return false;
        };

        // Must be exactly 64 bytes (32 random + 32 signature)
        if token_bytes.len() != 64 {
            return false;
        }

        let (random_bytes, signature) = token_bytes.split_at(32);

        // Verify HMAC
        let mut mac = HmacSha256::new_from_slice(secret)
            .expect("HMAC can take key of any size");
        mac.update(random_bytes);

        mac.verify_slice(signature).is_ok()
    }

    /// Get the token string
    pub fn as_str(&self) -> &str {
        &self.token
    }

    /// Get the token as a String
    pub fn to_string(&self) -> String {
        self.token.clone()
    }
}

/// CSRF protection middleware
///
/// Validates CSRF tokens on state-changing requests (POST, PUT, DELETE, PATCH).
/// Tokens can be sent via:
/// - `X-CSRF-Token` header
/// - `csrf_token` form field
/// - `_csrf` form field
pub struct CsrfProtection {
    secret: Vec<u8>,
    exempt_paths: Vec<String>,
}

impl CsrfProtection {
    /// Create a new CSRF protection middleware
    ///
    /// # Security
    /// The secret should be a cryptographically random value of at least 32 bytes.
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
            exempt_paths: Vec::new(),
        }
    }

    /// Add a path to exempt from CSRF protection
    ///
    /// Use this for API endpoints that use other authentication methods.
    pub fn exempt(mut self, path: impl Into<String>) -> Self {
        self.exempt_paths.push(path.into());
        self
    }

    /// Generate a new CSRF token
    pub fn generate_token(&self) -> CsrfToken {
        CsrfToken::generate(&self.secret)
    }

    /// Check if a path is exempt from CSRF protection
    fn is_exempt(&self, path: &str) -> bool {
        self.exempt_paths.iter().any(|exempt| path.starts_with(exempt))
    }

    /// Extract CSRF token from request
    fn extract_token(&self, req: &Request) -> Option<String> {
        // Try header first
        if let Some(token) = req.header("x-csrf-token") {
            return Some(token.to_string());
        }

        // Try alternative header
        if let Some(token) = req.header("csrf-token") {
            return Some(token.to_string());
        }

        // TODO: Try form fields when we implement form parsing

        None
    }
}

impl Middleware for CsrfProtection {
    fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        Box::pin(async move {
            let method = req.method();
            let path = req.path().to_string();

            // Only check state-changing methods
            let requires_csrf = matches!(
                method.as_str(),
                "POST" | "PUT" | "DELETE" | "PATCH"
            );

            if requires_csrf && !self.is_exempt(&path) {
                // Extract and verify token
                match self.extract_token(&req) {
                    Some(token_str) => {
                        let token = CsrfToken { token: token_str };
                        if !token.verify(&self.secret) {
                            return Err(Error::InvalidCsrfToken);
                        }
                    }
                    None => {
                        return Err(Error::InvalidCsrfToken);
                    }
                }
            }

            next(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_generate_and_verify() {
        let secret = b"test-secret-key-must-be-32-bytes";
        let token = CsrfToken::generate(secret);

        assert!(token.verify(secret));
        assert!(!token.verify(b"wrong-secret"));
    }

    #[test]
    fn test_csrf_token_invalid() {
        let secret = b"test-secret-key-must-be-32-bytes";
        let invalid_token = CsrfToken {
            token: "invalid".to_string(),
        };

        assert!(!invalid_token.verify(secret));
    }

    #[test]
    fn test_csrf_protection_exempt() {
        let csrf = CsrfProtection::new(b"secret".to_vec())
            .exempt("/api/webhook");

        assert!(csrf.is_exempt("/api/webhook"));
        assert!(csrf.is_exempt("/api/webhook/github"));
        assert!(!csrf.is_exempt("/api/users"));
    }
}
