# Half Framework

<div align="center">

A **lightweight**, **secure**, and **high-performance** Rust web framework.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

</div>

## 🚀 Features

- **⚡ Blazing Fast**: Zero-cost abstractions and efficient routing
- **🔒 Secure by Default**: Built-in CSRF, XSS protection, and input validation
- **🪶 Lightweight**: Minimal dependencies and small binary size
- **🎯 Simple API**: Intuitive and easy to use, yet powerful
- **🛡️ Type-Safe**: Compile-time route verification
- **🔧 CLI Tool**: Scaffold projects and generate code with ease

## 📦 Installation

Add Half to your `Cargo.toml`:

```toml
[dependencies]
half-core = "0.1"
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

## 📖 Full Example

Check out the `examples/` directory for complete examples:

- `hello.rs` - Simple Hello World
- `json_api.rs` - JSON API with user management
- `routing.rs` - Advanced routing with parameters

Run an example:

```bash
cargo run --example hello
cargo run --example json_api
cargo run --example routing
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

- **Fast routing**: O(n) route matching with path parameters
- **Async/await**: Built on Tokio for efficient concurrency
- **Zero-copy**: Minimal data copying in request/response handling
- **Small binaries**: Optimized release builds with LTO and strip

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
