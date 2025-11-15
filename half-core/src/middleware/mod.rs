//! Middleware system for Half framework
//!
//! Provides a flexible middleware system for request/response processing.

pub mod cache;
pub mod compression;
pub mod conditional;
pub mod ratelimit;
pub mod recovery;
pub mod timeout;

use crate::{Request, Response, error::Result};
use std::future::Future;
use std::pin::Pin;

// Re-export middleware types
pub use cache::{Cache, CacheConfig, CacheStats};
pub use compression::{Compression, CompressionAlgorithm, CompressionLevel};
pub use conditional::Conditional;
pub use ratelimit::{RateLimiter, RateLimitConfig};
pub use recovery::{Recovery, RecoveryConfig, RecoveryMode};
pub use timeout::{Timeout, TimeoutConfig};

/// Next middleware in the chain
pub type Next = Box<dyn FnOnce(Request) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> + Send>;

/// Middleware trait
///
/// Middlewares can inspect and modify requests before they reach handlers,
/// and responses before they are sent to clients.
pub trait Middleware: Send + Sync + 'static {
    /// Process a request
    ///
    /// Call `next` to pass the request to the next middleware or handler.
    fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>>;
}

/// Boxed middleware for dynamic dispatch
pub type BoxedMiddleware = Box<dyn Middleware>;

/// Logger middleware
///
/// Logs request method, path, and response status
pub struct Logger;

impl Middleware for Logger {
    fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        Box::pin(async move {
            let method = req.method().clone();
            let path = req.path().to_string();
            let start = std::time::Instant::now();

            println!("→ [{}] {}", method, path);

            let result = next(req).await;

            let duration = start.elapsed();
            match &result {
                Ok(_response) => {
                    println!("← [{}] {} - OK ({:?})", method, path, duration);
                }
                Err(error) => {
                    println!("← [{}] {} - Error: {} ({:?})", method, path, error, duration);
                }
            }

            result
        })
    }
}

/// CORS middleware
///
/// Handles Cross-Origin Resource Sharing
pub struct Cors {
    allow_origin: String,
    allow_methods: Vec<String>,
    allow_headers: Vec<String>,
    allow_credentials: bool,
    max_age: Option<u32>,
}

impl Cors {
    /// Create a new CORS middleware with default settings
    pub fn new() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allow_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
            ],
            allow_credentials: false,
            max_age: Some(3600),
        }
    }

    /// Set allowed origin
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allow_origin = origin.into();
        self
    }

    /// Set allowed methods
    pub fn allow_methods(mut self, methods: Vec<String>) -> Self {
        self.allow_methods = methods;
        self
    }

    /// Set allowed headers
    pub fn allow_headers(mut self, headers: Vec<String>) -> Self {
        self.allow_headers = headers;
        self
    }

    /// Enable credentials
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Set max age for preflight requests
    pub fn max_age(mut self, seconds: u32) -> Self {
        self.max_age = Some(seconds);
        self
    }
}

impl Default for Cors {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for Cors {
    fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        let allow_origin = self.allow_origin.clone();
        let allow_methods = self.allow_methods.join(", ");
        let allow_headers = self.allow_headers.join(", ");
        let allow_credentials = self.allow_credentials;
        let max_age = self.max_age;

        Box::pin(async move {
            // Handle preflight OPTIONS request
            if req.method().as_str() == "OPTIONS" {
                let mut response = Response::text("");
                response = response.header_str("access-control-allow-origin", &allow_origin);
                response = response.header_str("access-control-allow-methods", &allow_methods);
                response = response.header_str("access-control-allow-headers", &allow_headers);

                if allow_credentials {
                    response = response.header_str("access-control-allow-credentials", "true");
                }

                if let Some(age) = max_age {
                    response = response.header_str("access-control-max-age", &age.to_string());
                }

                return Ok(response);
            }

            // Process normal request
            let mut response = next(req).await?;

            response = response.header_str("access-control-allow-origin", &allow_origin);
            if allow_credentials {
                response = response.header_str("access-control-allow-credentials", "true");
            }

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_builder() {
        let cors = Cors::new()
            .allow_origin("https://example.com")
            .allow_methods(vec!["GET".to_string(), "POST".to_string()])
            .allow_credentials(true)
            .max_age(7200);

        assert_eq!(cors.allow_origin, "https://example.com");
        assert_eq!(cors.allow_methods.len(), 2);
        assert!(cors.allow_credentials);
        assert_eq!(cors.max_age, Some(7200));
    }
}
