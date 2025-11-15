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
pub use auth::{ApiKeyAuth, AuthError, BasicAuth, Claims, JwtAuth};
pub use body::{BodyConfig, BodyData, BodyError, BodyParser};
pub use config::{Config, ConfigBuilder, Environment};
pub use cookies::{CookieBuilder, CookieJar, SignedCookieJar};
pub use database::{JoinType, Order, QueryBuilder, QueryParams};
pub use error::{Error, Result};
pub use handler::Handler;
pub use health::{ComponentHealth, HealthCheck, HealthResponse, HealthStatus};
pub use logging::{LogConfig, MetricsLogger, RequestLogger};
pub use metrics::{Counter, Gauge, Histogram, Metrics, Timer};
pub use orm::{
    connection::{Connection, ConnectionError, ConnectionPool, DatabaseConfig, Transaction},
    drivers::{
        ConnectionInfo, DatabaseDriver, DatabaseType, DialectType, MySqlDriver, PostgresDriver,
        SqlDialect, SqliteDriver,
    },
    migrations::{Migration, MigrationBuilder, MigrationRunner, MigrationVersion},
    model::{Entity, Model, ModelBuilder, ModelError, Value},
    query::{Delete, Insert, Query, QueryExecutor, Update},
    relations::{BelongsTo, BelongsToMany, HasMany, HasOne, Relation, RelationType},
    schema::{Column, ColumnType, Constraint, ForeignKeyAction, Index, Schema, Table},
};
pub use request::Request;
pub use response::{Cookie, Response, SameSite};
pub use router::Router;
pub use server::{Server, TlsConfig};
pub use session::{Session, SessionConfig, SessionStats, SessionStore};
pub use sse::{SseChannel, SseEvent, SseStream};
pub use static_files::{StaticConfig, StaticFileServer};
pub use template::{TemplateConfig, TemplateContext, TemplateEngine};
pub use testing::{TestClient, TestRequest, TestResponse};
pub use trace::{RequestId, RequestTracer, TraceConfig, TraceContext};
pub use upload::{FileUpload, MultipartData, UploadConfig, UploadError, UploadedFile};
pub use websocket::{WebSocket, WsMessage};

// Re-export macros from half-macros
pub use half_macros::{delete, get, patch, post, put, route};

// Re-export security features
pub use security::{
    CspConfig, CsrfProtection, CsrfToken, Helmet, NonceProtection, PermissionsPolicyConfig,
    Validator, XssFilter,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        auth::{ApiKeyAuth, AuthError, BasicAuth, Claims, JwtAuth},
        body::{BodyConfig, BodyData, BodyError, BodyParser},
        cookies::{CookieBuilder, CookieJar, SignedCookieJar},
        database::{JoinType, Order, QueryBuilder, QueryParams},
        error::{Error, Result},
        handler::Handler,
        logging::{LogConfig, MetricsLogger, RequestLogger},
        middleware::Middleware,
        orm::{
            connection::{Connection, ConnectionPool, DatabaseConfig},
            drivers::{DatabaseDriver, DatabaseType, MySqlDriver, PostgresDriver, SqliteDriver},
            migrations::{Migration, MigrationRunner},
            model::{Entity, Model, Value},
            query::{Delete, Insert, Query, Update},
            relations::{BelongsTo, BelongsToMany, HasMany, HasOne},
            schema::{Column, ColumnType, Schema, Table},
        },
        request::Request,
        response::{Cookie, Response, SameSite},
        router::Router,
        security::{
            CspConfig, CsrfProtection, CsrfToken, Helmet, NonceProtection, PermissionsPolicyConfig,
            Validator, XssFilter,
        },
        server::{Server, TlsConfig},
        static_files::{StaticConfig, StaticFileServer},
        template::{TemplateConfig, TemplateContext, TemplateEngine},
        trace::{RequestId, RequestTracer, TraceConfig, TraceContext},
        upload::{FileUpload, MultipartData, UploadConfig, UploadError, UploadedFile},
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
