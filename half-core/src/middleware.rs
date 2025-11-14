//! Middleware system for Half framework
//!
//! Provides a flexible middleware system for request/response processing.

use crate::{Request, Response, error::Result};
use std::future::Future;
use std::pin::Pin;

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

            println!("[{}] {}", method, path);

            let result = next(req).await;

            match &result {
                Ok(_response) => {
                    // Note: We can't access status from Response directly in current implementation
                    println!("[{}] {} - OK", method, path);
                }
                Err(error) => {
                    println!("[{}] {} - Error: {}", method, path, error);
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
        let _allow_origin = self.allow_origin.clone();
        let _allow_methods = self.allow_methods.join(", ");
        let _allow_headers = self.allow_headers.join(", ");

        Box::pin(async move {
            let response = next(req).await?;

            // Add CORS headers
            // Note: In the current Response implementation, we need to modify this
            // For now, this is a placeholder showing the intended behavior

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
            .allow_methods(vec!["GET".to_string(), "POST".to_string()]);

        assert_eq!(cors.allow_origin, "https://example.com");
        assert_eq!(cors.allow_methods.len(), 2);
    }
}
