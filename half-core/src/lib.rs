//! # Half - A Lightweight, Secure, and High-Performance Rust Web Framework
//!
//! Half is designed to be minimal, secure by default, and blazing fast.
//! It provides a simple API while maintaining type safety and compile-time guarantees.
//!
//! ## Features
//!
//! - **Lightweight**: Minimal dependencies and small binary size
//! - **Fast**: Zero-cost abstractions and efficient routing
//! - **Secure**: Built-in CSRF, XSS protection, and input validation
//! - **Simple**: Intuitive API that feels natural to use
//! - **Type-Safe**: Compile-time route and handler verification
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use half_core::{Router, Server, Request, Response};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut router = Router::new();
//!
//!     router.get("/", |_req: Request| async {
//!         Response::text("Hello, Half!")
//!     });
//!
//!     router.get("/users/:id", |req: Request| async move {
//!         let id = req.param("id").unwrap_or("unknown");
//!         Response::json(&serde_json::json!({
//!             "user_id": id
//!         })).unwrap()
//!     });
//!
//!     Server::new(router)
//!         .bind(([127, 0, 0, 1], 3000))
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```
//!
//! ## Security Features
//!
//! Half includes built-in security features:
//!
//! - **CSRF Protection**: Token-based CSRF protection for state-changing requests
//! - **XSS Prevention**: Automatic HTML escaping and security headers
//! - **Input Validation**: Comprehensive validation utilities
//! - **SQL Injection Prevention**: Validation helpers to detect SQL injection attempts
//! - **Path Traversal Prevention**: Guards against directory traversal attacks
//!
//! ## Middleware
//!
//! Half supports a flexible middleware system:
//!
//! ```rust,ignore
//! use half_core::{Router, middleware::{Logger, Cors}};
//!
//! let mut router = Router::new();
//! router.use_middleware(Logger);
//! router.use_middleware(Cors::new());
//! ```

// Public exports
pub mod error;
pub mod handler;
pub mod health;
pub mod metrics;
pub mod middleware;
pub mod request;
pub mod response;
pub mod router;
pub mod security;
pub mod server;
pub mod session;
pub mod sse;
pub mod websocket;

// Re-export commonly used types
pub use error::{Error, Result};
pub use handler::Handler;
pub use health::{HealthCheck, HealthResponse, HealthStatus, ComponentHealth};
pub use metrics::{Metrics, Counter, Gauge, Histogram, Timer};
pub use request::Request;
pub use response::{Response, Cookie, SameSite};
pub use router::Router;
pub use server::{Server, TlsConfig};
pub use session::{SessionStore, Session, SessionConfig, SessionStats};
pub use sse::{SseChannel, SseEvent, SseStream};
pub use websocket::{WebSocket, WsMessage};

// Re-export macros from half-macros
pub use half_macros::{get, post, put, delete, patch, route};

// Re-export security features
pub use security::{
    CsrfProtection, CsrfToken,
    XssFilter,
    Validator,
    NonceProtection,
    Helmet, CspConfig, PermissionsPolicyConfig,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        error::{Error, Result},
        handler::Handler,
        middleware::Middleware,
        request::Request,
        response::{Cookie, Response, SameSite},
        router::Router,
        security::{
            CsrfProtection, CsrfToken,
            XssFilter,
            Validator,
            NonceProtection,
            Helmet, CspConfig, PermissionsPolicyConfig,
        },
        server::{Server, TlsConfig},
    };

    pub use half_macros::{delete, get, patch, post, put, route};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_routing() {
        let mut router = Router::new();

        router.get("/test", |_req: Request| async {
            Response::text("Hello, Test!")
        });

        // Basic smoke test - just ensure it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_response_creation() {
        let _response = Response::text("Hello");
        // Basic smoke test - just ensure it doesn't panic
        assert!(true);
    }
}
