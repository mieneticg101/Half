# Half Framework

<div align="center">

A **lightweight**, **secure**, and **high-performance** Rust web framework.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-green.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![Version](https://img.shields.io/badge/version-0.3.0-blue.svg)](CHANGELOG.md)
[![TLS](https://img.shields.io/badge/TLS-1.3-green.svg)](https://tools.ietf.org/html/rfc8446)

</div>

## 🚀 Features

- **⚡ Blazing Fast**: Zero-cost abstractions and efficient routing
- **🔒 Secure by Default**: Built-in CSRF, XSS protection, and input validation
- **🔐 TLS 1.3 Support**: Native HTTPS with modern encryption
- **🛡️ Advanced Security**: Helmet headers and nonce-based replay protection
- **🪶 Lightweight**: Minimal dependencies and small binary size
- **🎯 Simple API**: Intuitive and easy to use, yet powerful
- **🛡️ Type-Safe**: Compile-time route verification
- **🔧 CLI Tool**: Scaffold projects and generate code with ease

## 📦 Installation

**Requirements:**
- Rust 1.85+ (for edition 2024 support)

Add Half to your `Cargo.toml`:

```toml
[dependencies]
half-core = "0.2"
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
            "user_id": id
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

## 📚 Examples

### Hello World

```rust
router.get("/", |_req: Request| async {
    Response::text("Hello, World!")
});
```

### JSON API

```rust
#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

router.get("/users/:id", |req: Request| async move {
    let id = req.param("id").unwrap_or("0").parse().unwrap_or(0);
    let user = User { id, name: format!("User {}", id) };
    Response::json(&user).unwrap()
});
```

### Path Parameters

```rust
router.get("/posts/:post_id/comments/:comment_id", |req: Request| async move {
    let post_id = req.param("post_id");
    let comment_id = req.param("comment_id");
    // Handle request...
});
```

### Query Parameters

```rust
router.get("/search", |req: Request| async move {
    let query = req.query_param("q").unwrap_or("");
    let page = req.query_param("page").unwrap_or("1");
    // Handle search...
});
```

## 🔒 Security Features

Half includes built-in security features to protect your application:

### TLS 1.3 Support

```rust
use half_core::{Server, Router, TlsConfig};

let tls = TlsConfig::new("cert.pem", "key.pem");

Server::new(router)
    .bind(([0, 0, 0, 0], 443))
    .tls(tls)
    .run()
    .await?;
```

### Helmet Security Headers (Beyond Helmet.js)

```rust
use half_core::security::Helmet;

// Use default strict security
let helmet = Helmet::new();
router.use_middleware(helmet);

// Or customize
let helmet = Helmet::new()
    .hsts(31536000, true, true)  // 1 year HSTS with preload
    .frame_options("DENY")
    .referrer_policy("strict-origin-when-cross-origin");

router.use_middleware(helmet);
```

### Nonce Protection (One-Time Requests)

```rust
use half_core::security::NonceProtection;

let nonce = NonceProtection::new()
    .ttl(300)  // 5 minutes
    .exempt("/public");

router.use_middleware(nonce);

// Generate nonce for clients
let nonce_value = nonce.generate_nonce();
```

### CSRF Protection

```rust
use half_core::security::CsrfProtection;

let csrf = CsrfProtection::new(b"your-secret-key-32-bytes-long!!")
    .exempt("/api/webhook");

router.use_middleware(csrf);
```

### XSS Prevention

```rust
use half_core::security::XssFilter;

let xss = XssFilter::new()
    .csp("default-src 'self'; script-src 'self'");

router.use_middleware(xss);
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
```

## 🎨 Middleware

Half supports a flexible middleware system:

```rust
use half_core::middleware::{Logger, Cors};

let mut router = Router::new();

// Add logging
router.use_middleware(Logger);

// Add CORS support
router.use_middleware(Cors::new()
    .allow_origin("https://example.com")
    .allow_methods(vec!["GET".to_string(), "POST".to_string()]));
```

## 🛠️ CLI Tool

Half comes with a CLI tool for rapid development:

```bash
# Create a new project
half new my-project

# Generate a new route
half route /users/:id --method GET
```

## 📖 Full Examples

Check out the `examples/` directory for complete examples:

- `hello.rs` - Simple Hello World
- `json_api.rs` - JSON API with user management
- `routing.rs` - Advanced routing with parameters
- `secure_server.rs` - HTTPS with TLS 1.3, Nonce Protection, and Helmet
- `helmet_demo.rs` - Comprehensive security headers demonstration

Run an example:

```bash
cargo run --example hello
cargo run --example json_api
cargo run --example routing
cargo run --example secure_server  # Requires cert.pem and key.pem
cargo run --example helmet_demo
```

## 🏗️ Architecture

Half is built with these core principles:

- **Zero-cost abstractions**: No runtime overhead for convenience features
- **Security by default**: Protection against common vulnerabilities built-in
- **Minimal dependencies**: Only essential dependencies included
- **Type safety**: Leverage Rust's type system for compile-time guarantees

## 🔧 Configuration

### Server Configuration

```rust
Server::new(router)
    .bind(([0, 0, 0, 0], 8080))  // Bind to all interfaces
    .run()
    .await?;
```

### Response Types

```rust
// Text response
Response::text("Hello");

// JSON response
Response::json(&data)?;

// HTML response (automatically escaped for XSS protection)
Response::html("<h1>Hello</h1>");

// Redirect
Response::redirect("/new-path", false);

// Custom status
Response::text("Not Found").status(StatusCode::NOT_FOUND);

// With cookies
Response::text("OK")
    .cookie(Cookie::new("session", "abc123")
        .path("/")
        .secure(true)
        .http_only(true));
```

## 🧪 Testing

Run all tests:

```bash
cargo test --workspace
```

Run tests for a specific crate:

```bash
cargo test -p half-core
cargo test -p half-macros
cargo test -p half-cli
```

## 📊 Performance

Half is designed for maximum performance:

- **Fast routing**: O(1) HashMap lookup for exact routes, O(n) for parameterized routes
- **Iterator-based matching**: Zero-allocation path matching using iterators
- **Async/await**: Built on Tokio for efficient concurrency
- **Minimal allocations**: Optimized hot paths to reduce heap allocations
- **Small binaries**: Optimized release builds with LTO and strip
- **Security without overhead**: Built-in protections with minimal runtime cost

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

Half is built on the shoulders of giants:

- [Tokio](https://tokio.rs/) - Asynchronous runtime
- [Hyper](https://hyper.rs/) - HTTP implementation
- [Serde](https://serde.rs/) - Serialization framework

---

<div align="center">

**Built with ❤️ in Rust**

</div>
