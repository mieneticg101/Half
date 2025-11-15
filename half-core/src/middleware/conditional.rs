//! Conditional middleware execution
//!
//! Apply middleware only when certain conditions are met.

use crate::{Request, Response, Result, middleware::Middleware};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Condition function type
pub type Condition = Arc<dyn Fn(&Request) -> bool + Send + Sync>;

/// Conditional middleware
///
/// Wraps another middleware and only executes it when the condition is met.
pub struct Conditional<M: Middleware> {
    middleware: M,
    condition: Condition,
}

impl<M: Middleware> Conditional<M> {
    /// Create new conditional middleware
    pub fn new(middleware: M, condition: Condition) -> Self {
        Self {
            middleware,
            condition,
        }
    }

    /// Create conditional middleware from function
    pub fn when<F>(middleware: M, condition: F) -> Self
    where
        F: Fn(&Request) -> bool + Send + Sync + 'static,
    {
        Self {
            middleware,
            condition: Arc::new(condition),
        }
    }

    /// Only apply middleware for specific paths
    pub fn for_path(middleware: M, path: impl Into<String>) -> Self {
        let target_path = path.into();
        Self::when(middleware, move |req| req.path() == target_path)
    }

    /// Only apply middleware for paths matching prefix
    pub fn for_prefix(middleware: M, prefix: impl Into<String>) -> Self {
        let target_prefix = prefix.into();
        Self::when(middleware, move |req| {
            req.path().starts_with(&target_prefix)
        })
    }

    /// Only apply middleware for specific HTTP method
    pub fn for_method(middleware: M, method: impl Into<String>) -> Self {
        let target_method = method.into().to_uppercase();
        Self::when(middleware, move |req| {
            req.method().as_str().to_uppercase() == target_method
        })
    }

    /// Only apply middleware when header exists
    pub fn when_header(middleware: M, header_name: impl Into<String>) -> Self {
        let header = header_name.into();
        Self::when(middleware, move |req| req.header(&header).is_some())
    }

    /// Get the condition function
    pub fn condition(&self) -> &Condition {
        &self.condition
    }
}

impl<M: Middleware> Middleware for Conditional<M> {
    fn handle(
        &self,
        req: Request,
        next: crate::middleware::Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        // Check condition
        if (self.condition)(&req) {
            // Condition met - apply middleware
            self.middleware.handle(req, next)
        } else {
            // Condition not met - skip middleware
            Box::pin(async move { next(req).await })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::Logger;

    #[test]
    fn test_conditional_creation() {
        let logger = Logger;
        let _conditional = Conditional::when(logger, |_req| true);
        // If we got here without panicking, the test passes
    }
}
