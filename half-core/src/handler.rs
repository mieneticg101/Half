//! Handler traits and implementations
//!
//! Provides the core Handler trait that powers Half's routing system.

use crate::{Request, Response, error::Result};
use std::future::Future;
use std::pin::Pin;

/// Handler trait for route handlers
///
/// This trait is automatically implemented for async functions that return
/// a Response or Result<Response>.
pub trait Handler: Send + Sync + 'static {
    /// Handle a request and return a response
    fn call(&self, req: Request) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>>;
}

/// Implement Handler for async functions that return Response
impl<F, Fut> Handler for F
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn call(&self, req: Request) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        Box::pin(async move {
            let response = self(req).await;
            Ok(response)
        })
    }
}

/// Boxed handler for dynamic dispatch
pub type BoxedHandler = Box<dyn Handler>;

/// Handler wrapper that provides additional functionality
pub struct HandlerWrapper {
    handler: BoxedHandler,
}

impl HandlerWrapper {
    /// Create a new handler wrapper
    pub fn new<H: Handler>(handler: H) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }

    /// Call the wrapped handler
    pub async fn handle(&self, req: Request) -> Result<Response> {
        self.handler.call(req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_trait() {
        async fn test_handler(_req: Request) -> Response {
            Response::text("Hello")
        }

        // Handler trait should be implemented automatically
        let _wrapper = HandlerWrapper::new(test_handler);
    }
}
