# Half Framework

<div align="center">

A **lightweight**, **secure**, and **high-performance** Rust web framework with built-in **ORM** and **advanced security**.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-green.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![Version](https://img.shields.io/badge/version-0.13.0-blue.svg)](CHANGELOG.md)
[![TLS](https://img.shields.io/badge/TLS-1.3-green.svg)](https://tools.ietf.org/html/rfc8446)
[![Tests](https://img.shields.io/badge/tests-278%20passing-brightgreen.svg)]()

</div>

## ✨ Highlights

- 🗄️ **Built-in ORM** - Type-safe database operations with migrations and relationships
- ⚡ **Blazing Fast** - Zero-cost abstractions with performance optimizations
- 🔒 **Secure by Default** - CSRF, XSS, Helmet headers, and input validation
- 🔐 **TLS 1.3** - Native HTTPS with modern encryption
- 🎯 **Simple API** - Intuitive and easy to use, yet powerful
- 🛡️ **Type-Safe** - Compile-time route and query verification
- 🔧 **Full-Featured** - WebSockets, SSE, file uploads, templates, and more

## 🚀 Features

### Core Features
- ⚡ **High Performance**: Zero-cost abstractions, optimized hot paths
- 🌐 **Async/Await**: Built on Tokio for efficient concurrency
- 🎯 **Smart Routing**: Fast path matching with parameters and wildcards
- 🔌 **Middleware System**: Flexible request/response pipeline
- 📝 **Request/Response**: Rich API with helpers for common tasks

### Database & ORM
- 🗄️ **Internal ORM System**: Type-safe models with automatic CRUD
- 🔗 **Relationships**: HasOne, HasMany, BelongsTo, BelongsToMany
- 📊 **Query Builder**: Fluent API for complex queries
- 🔄 **Migrations**: Schema versioning with up/down migrations
- 🏊 **Connection Pooling**: Thread-safe connection management
- 💾 **Transactions**: ACID transactions with auto-rollback

### Security
- 🛡️ **CSRF Protection**: Token-based protection for state-changing requests
- 🚫 **XSS Prevention**: Automatic HTML escaping and CSP headers
- 🔐 **Helmet Security**: 10+ security headers (HSTS, X-Frame-Options, etc.)
- 🎫 **Nonce Protection**: One-time request tokens to prevent replay attacks
- ✅ **Input Validation**: Comprehensive validators (email, SQL injection, path traversal)
- 🔒 **TLS 1.3**: Modern HTTPS encryption

### Authentication
- 🔑 **JWT Authentication**: JSON Web Token support with claims
- 👤 **Basic Auth**: HTTP Basic Authentication
- 🔐 **API Key Auth**: API key authentication middleware

### Real-time & Communication
- 🔌 **WebSockets**: Full-duplex communication
- 📡 **Server-Sent Events (SSE)**: Real-time server push
- 📨 **Streaming**: Efficient streaming responses

### Data Handling
- 📤 **File Uploads**: Multipart form data with validation
- 📁 **Static Files**: Efficient static file serving
- 🎨 **Templates**: Handlebars template engine
- 🍪 **Cookies**: Signed and encrypted cookies
- 💼 **Sessions**: In-memory session management

### Performance & Monitoring
- 📊 **Metrics**: Counters, gauges, histograms, and timers
- 🏥 **Health Checks**: Component health monitoring
- 📝 **Logging**: Structured logging with tracing
- 🎯 **Request Tracing**: Request ID tracking
- ⚡ **Compression**: Gzip, Brotli, and Deflate
- 🔄 **Caching**: Flexible caching middleware
- ⏱️ **Rate Limiting**: Token bucket rate limiting

### Developer Experience
- 🛠️ **CLI Tool**: Project scaffolding and code generation
- 🧪 **Testing Utilities**: Test client for integration tests
- 📚 **Rich Documentation**: Comprehensive guides and examples
- 🔍 **Type Safety**: Compile-time guarantees
- 🎯 **Error Handling**: Detailed error messages

## 📦 Installation

**Requirements:**
- Rust 1.85+ (for edition 2024 support)

Add Half to your `Cargo.toml`:

```toml
[dependencies]
half-core = "0.13"
tokio = { version = "1.42", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Or use the CLI to create a new project:

```bash
cargo install half-cli
half new my-project
cd my-project
cargo run
```

## 🏃 Quick Start

```rust
use half_core::{Router, Server, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();

    router.get("/", |_req: Request| async {
        Response::text("Hello, Half!")
    });

    router.get("/users/:id", |req: Request| async move {
        let id = req.param("id").unwrap_or("unknown");
        Response::json(&serde_json::json!({
            "user_id": id,
            "name": format!("User {}", id)
        }))
        .unwrap()
    });

    Server::new(router)
        .bind(([127, 0, 0, 1], 3000))
        .run()
        .await?;

    Ok(())
}
```

## 🗄️ ORM Usage

### Define Models

```rust
use half_core::orm::{Model, Entity, Value, Table, Column, ColumnType};
use std::collections::HashMap;

#[derive(Debug)]
struct User {
    id: Option<i64>,
    name: String,
    email: String,
    created_at: i64,
}

impl Model for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn schema() -> Table {
        Table::new("users")
            .add_column(
                Column::new("id", ColumnType::BigInteger)
                    .primary_key()
                    .auto_increment()
            )
            .add_column(Column::new("name", ColumnType::String(255)).not_null())
            .add_column(Column::new("email", ColumnType::String(255)).unique())
            .add_column(Column::new("created_at", ColumnType::Timestamp).not_null())
    }

    fn to_values(&self) -> HashMap<String, Value> {
        let mut values = HashMap::new();
        if let Some(id) = self.id {
            values.insert("id".to_string(), Value::Integer(id));
        }
        values.insert("name".to_string(), Value::String(self.name.clone()));
        values.insert("email".to_string(), Value::String(self.email.clone()));
        values.insert("created_at".to_string(), Value::Integer(self.created_at));
        values
    }

    fn from_values(values: HashMap<String, Value>) -> Result<Self, ModelError> {
        Ok(Self {
            id: values.get("id").and_then(|v| v.as_i64()),
            name: values.get("name").and_then(|v| v.as_string()).unwrap_or("").to_string(),
            email: values.get("email").and_then(|v| v.as_string()).unwrap_or("").to_string(),
            created_at: values.get("created_at").and_then(|v| v.as_i64()).unwrap_or(0),
        })
    }

    fn columns() -> Vec<&'static str> {
        vec!["id", "name", "email", "created_at"]
    }
}

impl Entity for User {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }
}
```

### Type-Safe Queries

```rust
use half_core::orm::{Query, Insert, Update, Delete};

// SELECT queries
let users: Vec<User> = Query::new()
    .where_eq("active", true)
    .where_gt("age", 18)
    .order_by("created_at", Order::Desc)
    .limit(10)
    .get(&mut executor)?;

// Get first user
let user: Option<User> = Query::new()
    .where_eq("email", "john@example.com")
    .first(&mut executor)?;

// Count users
let count: u64 = Query::new()
    .where_eq("role", "admin")
    .count(&mut executor)?;

// INSERT
Insert::<User>::new()
    .set("name", "John Doe")
    .set("email", "john@example.com")
    .set("created_at", 1234567890)
    .execute(&mut executor)?;

// UPDATE
Update::<User>::new()
    .set("name", "Jane Doe")
    .where_eq("id", 1)
    .execute(&mut executor)?;

// DELETE
Delete::<User>::new()
    .where_eq("id", 1)
    .execute(&mut executor)?;
```

### Relationships

```rust
use half_core::orm::{HasMany, BelongsTo};

// Define relationships
struct User {
    // ... fields
}

impl User {
    fn posts(&self) -> HasMany<User, Post> {
        HasMany::new("user_id", "id")
    }
}

struct Post {
    // ... fields
}

impl Post {
    fn user(&self) -> BelongsTo<Post, User> {
        BelongsTo::new("user_id", "id")
    }
}

// Use relationships
let posts = user.posts().get(&mut executor)?;
let user = post.user().get(&mut executor)?;
```

### Migrations

```rust
use half_core::orm::{Migration, MigrationBuilder};

let migration = MigrationBuilder::new("001", "Create users table")
    .create_table(&User::schema())
    .build();

let mut runner = MigrationRunner::new();
runner.add_migration(Box::new(migration));

// Run migrations
runner.run(&mut conn)?;

// Rollback
runner.rollback(&mut conn)?;

// Check status
let status = runner.status(&mut conn)?;
println!("Applied: {}, Pending: {}", status.applied, status.pending);
```

### Connection Pool

```rust
use half_core::orm::{ConnectionPool, DatabaseConfig};

let config = DatabaseConfig::new("sqlite::memory:")
    .max_connections(10)
    .connect_timeout(30);

let pool = ConnectionPool::new(config);

// Get connection
let conn = pool.get()?;

// Use connection
Query::<User>::new().get(&mut conn)?;

// Automatically returned to pool when dropped
```

## 📚 Examples

### JSON API with ORM

```rust
router.get("/api/users/:id", |req: Request| async move {
    let id: i64 = req.param("id")?.parse()?;

    let user: Option<User> = Query::new()
        .where_eq("id", id)
        .first(&mut executor)?;

    match user {
        Some(user) => Response::json(&user)?,
        None => Response::text("User not found").status(StatusCode::NOT_FOUND),
    }
});

router.post("/api/users", |req: Request| async move {
    let data: UserCreate = req.json().await?;

    Insert::<User>::new()
        .set("name", &data.name)
        .set("email", &data.email)
        .set("created_at", chrono::Utc::now().timestamp())
        .execute(&mut executor)?;

    Response::text("User created").status(StatusCode::CREATED)
});
```

### WebSocket Chat

```rust
use half_core::websocket::{WebSocket, WsMessage};

router.get("/ws", |req: Request| async move {
    let ws = WebSocket::from_request(req).await?;

    tokio::spawn(async move {
        while let Some(msg) = ws.recv().await {
            match msg {
                WsMessage::Text(text) => {
                    println!("Received: {}", text);
                    ws.send(WsMessage::Text(format!("Echo: {}", text))).await?;
                }
                WsMessage::Close(_) => break,
                _ => {}
            }
        }
    });

    Response::empty()
});
```

### Server-Sent Events

```rust
use half_core::sse::{SseChannel, SseEvent};

let (tx, rx) = SseChannel::new();

router.get("/events", |_req: Request| async move {
    SseStream::new(rx)
});

// Send events
tx.send(SseEvent::new("ping").data("heartbeat")).await?;
```

### File Upload

```rust
use half_core::upload::{FileUpload, UploadConfig};

let config = UploadConfig::new()
    .max_file_size(10 * 1024 * 1024) // 10 MB
    .allowed_extensions(vec!["jpg", "png", "pdf"]);

router.post("/upload", |req: Request| async move {
    let upload = FileUpload::from_request(req, config).await?;

    for file in upload.files {
        file.save(&format!("uploads/{}", file.filename)).await?;
    }

    Response::text("Files uploaded successfully")
});
```

### Authentication (JWT)

```rust
use half_core::auth::{JwtAuth, Claims};

let jwt = JwtAuth::new(b"your-secret-key");

router.use_middleware(jwt);

router.get("/protected", |req: Request| async move {
    let claims = req.claims::<Claims>()?;
    Response::json(&claims)?
});

// Generate token
let token = jwt.encode(&Claims {
    sub: "user123".to_string(),
    exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
})?;
```

## 🔒 Security

### Helmet Headers

```rust
use half_core::security::Helmet;

let helmet = Helmet::new()
    .hsts(31536000, true, true)  // 1 year HSTS with preload
    .frame_options("DENY")
    .content_type_options()
    .xss_protection()
    .referrer_policy("strict-origin-when-cross-origin");

router.use_middleware(helmet);
```

### CSRF Protection

```rust
use half_core::security::CsrfProtection;

let csrf = CsrfProtection::new(b"your-secret-key-32-bytes-long!!")
    .exempt("/api/webhook");

router.use_middleware(csrf);
```

### Input Validation

```rust
use half_core::security::Validator;

// Validate email
Validator::email("user@example.com")?;

// Validate length
Validator::length("password", 8, 128)?;

// Prevent SQL injection
Validator::no_sql_injection(user_input)?;

// Prevent path traversal
Validator::no_path_traversal(file_path)?;

// Custom validation
Validator::custom(value, |v| v.len() > 0, "Value cannot be empty")?;
```

## 🎨 Middleware

### Built-in Middleware

```rust
use half_core::middleware::*;

// Logging
router.use_middleware(Logger);

// CORS
router.use_middleware(Cors::new()
    .allow_origin("https://example.com")
    .allow_methods(vec!["GET", "POST"]));

// Rate Limiting
router.use_middleware(RateLimiter::new(100, Duration::from_secs(60)));

// Compression
router.use_middleware(Compression::new());

// Caching
router.use_middleware(Cache::new().ttl(3600));

// Timeout
router.use_middleware(Timeout::new(Duration::from_secs(30)));

// Recovery (panic handler)
router.use_middleware(Recovery::new());
```

### Custom Middleware

```rust
use half_core::middleware::Middleware;

struct CustomMiddleware;

#[async_trait]
impl Middleware for CustomMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Before request
        println!("Before: {}", req.uri());

        let res = next.run(req).await;

        // After request
        println!("After: {}", res.status());

        res
    }
}

router.use_middleware(CustomMiddleware);
```

## 🧪 Testing

```rust
use half_core::testing::TestClient;

#[tokio::test]
async fn test_api() {
    let mut router = Router::new();
    router.get("/health", |_| async { Response::text("OK") });

    let client = TestClient::new(router);

    let res = client.get("/health").send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.text().await.unwrap(), "OK");
}
```

## 📊 Performance

Half is optimized for maximum performance:

- **Fast Routing**: O(1) HashMap lookup for exact routes
- **Zero-Cost Abstractions**: No runtime overhead
- **Optimized Hot Paths**: Strategic `#[inline]` attributes
- **Lazy Operations**: Conditional allocations
- **Connection Pooling**: Reuse database connections
- **Async/Await**: Efficient concurrency with Tokio

### Performance Metrics

- **278 unit tests**: All passing
- **Request handling**: 5-10% faster with optimizations
- **Binary size**: ~8.2 MB (debug), optimized in release
- **Memory efficient**: Minimal allocations in hot paths

See [PERFORMANCE.md](PERFORMANCE.md) for detailed performance documentation.

## 🛠️ CLI Tool

```bash
# Create new project
half new my-app

# Generate route
half route /users/:id --method GET

# Generate model
half model User --table users

# Run migrations
half migrate up
half migrate down
half migrate status
```

## 📖 Documentation

- [Changelog](CHANGELOG.md) - Version history and changes
- [Performance](PERFORMANCE.md) - Performance optimizations guide
- [Examples](examples/) - Code examples
- API Documentation (run `cargo doc --open`)

## 🏗️ Architecture

Half is built with these core principles:

- **Zero-Cost Abstractions**: No runtime overhead
- **Security by Default**: Built-in protection
- **Type Safety**: Compile-time guarantees
- **Developer Experience**: Intuitive and productive
- **Production Ready**: Battle-tested patterns

### Module Structure

```
half-core/
├── auth/          # Authentication (JWT, Basic, API Key)
├── body/          # Request body parsing
├── config/        # Configuration management
├── cookies/       # Cookie handling
├── database/      # Query builder
├── error/         # Error handling
├── handler/       # Request handlers
├── health/        # Health checks
├── logging/       # Logging utilities
├── metrics/       # Metrics collection
├── middleware/    # Middleware system
├── orm/           # ORM system
│   ├── connection.rs    # Connection pooling
│   ├── migrations.rs    # Schema migrations
│   ├── model.rs         # Model traits
│   ├── query.rs         # Query builder
│   ├── relations.rs     # Relationships
│   └── schema.rs        # Schema definition
├── request/       # Request handling
├── response/      # Response building
├── router/        # Routing system
├── security/      # Security features
├── server/        # HTTP server
├── session/       # Session management
├── sse/           # Server-Sent Events
├── static_files/  # Static file serving
├── template/      # Template engine
├── testing/       # Testing utilities
├── trace/         # Request tracing
├── upload/        # File uploads
└── websocket/     # WebSocket support
```

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run with all features
cargo test --all-features

# Run specific crate
cargo test -p half-core
cargo test -p half-macros
cargo test -p half-cli

# Run with coverage
cargo tarpaulin --out Html
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

Half is built on excellent Rust libraries:

- [Tokio](https://tokio.rs/) - Asynchronous runtime
- [Hyper](https://hyper.rs/) - HTTP implementation
- [Serde](https://serde.rs/) - Serialization framework
- [Rustls](https://github.com/rustls/rustls) - Modern TLS library
- And many more amazing crates!

## 📞 Community

- **Issues**: [GitHub Issues](https://github.com/mieneticg101/Half/issues)
- **Discussions**: [GitHub Discussions](https://github.com/mieneticg101/Half/discussions)

---

<div align="center">

**Built with ❤️ and ⚙️ in Rust**

[Documentation](https://github.com/mieneticg101/Half) • [Examples](examples/) • [Changelog](CHANGELOG.md)

</div>
