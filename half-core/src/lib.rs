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
pub mod auth;
pub mod body;
pub mod config;
pub mod cookies;
pub mod database;
pub mod error;
pub mod handler;
pub mod health;
pub mod logging;
pub mod metrics;
pub mod middleware;
pub mod orm;
pub mod request;
pub mod response;
pub mod router;
pub mod security;
pub mod server;
pub mod session;
pub mod sse;
pub mod static_files;
pub mod template;
pub mod testing;
pub mod trace;
pub mod upload;
pub mod websocket;

// Re-export commonly used types
pub use auth::{JwtAuth, BasicAuth, ApiKeyAuth, Claims, AuthError};
pub use body::{BodyParser, BodyConfig, BodyData, BodyError};
pub use config::{Config, ConfigBuilder, Environment};
pub use cookies::{CookieJar, SignedCookieJar, CookieBuilder};
pub use database::{QueryBuilder, QueryParams, Order, JoinType};
pub use error::{Error, Result};
pub use orm::{
    connection::{Connection, ConnectionPool, DatabaseConfig, Transaction, ConnectionError},
    model::{Model, Entity, Value, ModelError, ModelBuilder},
    query::{Query, QueryExecutor, Insert, Update, Delete},
    schema::{Schema, Table, Column, ColumnType, Constraint, Index, ForeignKeyAction},
    migrations::{Migration, MigrationRunner, MigrationVersion, MigrationBuilder},
    relations::{Relation, RelationType, HasOne, HasMany, BelongsTo, BelongsToMany},
    drivers::{
        DatabaseType, DatabaseDriver, ConnectionInfo,
        PostgresDriver, MySqlDriver, SqliteDriver,
        SqlDialect, DialectType,
    },
};
pub use handler::Handler;
pub use health::{HealthCheck, HealthResponse, HealthStatus, ComponentHealth};
pub use logging::{LogConfig, RequestLogger, MetricsLogger};
pub use metrics::{Metrics, Counter, Gauge, Histogram, Timer};
pub use request::Request;
pub use response::{Response, Cookie, SameSite};
pub use router::Router;
pub use server::{Server, TlsConfig};
pub use session::{SessionStore, Session, SessionConfig, SessionStats};
pub use sse::{SseChannel, SseEvent, SseStream};
pub use static_files::{StaticFileServer, StaticConfig};
pub use template::{TemplateEngine, TemplateConfig, TemplateContext};
pub use testing::{TestClient, TestRequest, TestResponse};
pub use trace::{RequestId, TraceContext, TraceConfig, RequestTracer};
pub use upload::{FileUpload, UploadConfig, UploadedFile, MultipartData, UploadError};
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
        auth::{JwtAuth, BasicAuth, ApiKeyAuth, Claims, AuthError},
        body::{BodyParser, BodyConfig, BodyData, BodyError},
        cookies::{CookieJar, SignedCookieJar, CookieBuilder},
        database::{QueryBuilder, QueryParams, Order, JoinType},
        error::{Error, Result},
        handler::Handler,
        logging::{LogConfig, RequestLogger, MetricsLogger},
        middleware::Middleware,
        orm::{
            connection::{Connection, ConnectionPool, DatabaseConfig},
            model::{Model, Entity, Value},
            query::{Query, Insert, Update, Delete},
            schema::{Schema, Table, Column, ColumnType},
            migrations::{Migration, MigrationRunner},
            relations::{HasOne, HasMany, BelongsTo, BelongsToMany},
            drivers::{DatabaseType, DatabaseDriver, PostgresDriver, MySqlDriver, SqliteDriver},
        },
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
        static_files::{StaticFileServer, StaticConfig},
        template::{TemplateEngine, TemplateConfig, TemplateContext},
        trace::{RequestId, TraceContext, TraceConfig, RequestTracer},
        upload::{FileUpload, UploadConfig, UploadedFile, MultipartData, UploadError},
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
    }

    #[test]
    fn test_response_creation() {
        let _response = Response::text("Hello");
        // Basic smoke test - just ensure it doesn't panic
    }
}
