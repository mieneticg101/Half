//! Half CLI - Command-line tool for Half framework
//!
//! Provides commands for creating and managing Half projects.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "half")]
#[command(about = "Half Framework CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Half project
    New {
        /// Project name
        name: String,
    },
    /// Generate a new route handler
    Route {
        /// Route path (e.g., "/users/:id")
        path: String,
        /// HTTP method (GET, POST, PUT, DELETE, PATCH)
        #[arg(short, long, default_value = "GET")]
        method: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => {
            create_project(&name)?;
        }
        Commands::Route { path, method } => {
            generate_route(&path, &method)?;
        }
    }

    Ok(())
}

/// Create a new Half project
fn create_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Creating new Half project: {}", name);

    // Create project directory
    fs::create_dir(name)?;
    let project_path = Path::new(name);

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
half-core = "0.1"
tokio = {{ version = "1.42", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
        name, name
    );
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

    // Create src directory
    fs::create_dir(project_path.join("src"))?;

    // Create main.rs with example code
    let main_rs = r#"use half_core::{Router, Server, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();

    // Define routes
    router.get("/", index_handler);
    router.get("/hello/:name", hello_handler);

    // Start server
    println!("🚀 Server running on http://127.0.0.1:3000");
    Server::new(router)
        .bind(([127, 0, 0, 1], 3000))
        .run()
        .await?;

    Ok(())
}

/// Index route handler
async fn index_handler(_req: Request) -> Response {
    Response::json(&serde_json::json!({
        "message": "Welcome to Half Framework!",
        "version": "0.1.0"
    }))
    .unwrap_or_else(|_| Response::text("Welcome to Half Framework!"))
}

/// Hello route handler with path parameter
async fn hello_handler(req: Request) -> Response {
    let name = req.param("name").unwrap_or("World");
    Response::json(&serde_json::json!({
        "message": format!("Hello, {}!", name)
    }))
    .unwrap_or_else(|_| Response::text(format!("Hello, {}!", name)))
}
"#;
    fs::write(project_path.join("src/main.rs"), main_rs)?;

    // Create .gitignore
    let gitignore = r#"/target
Cargo.lock
*.swp
.DS_Store
"#;
    fs::write(project_path.join(".gitignore"), gitignore)?;

    // Create README.md
    let readme = format!(
        r#"# {}

A Half Framework project.

## Getting Started

```bash
cargo run
```

The server will start on http://127.0.0.1:3000

## Routes

- `GET /` - Index
- `GET /hello/:name` - Hello with name parameter

## Built with Half

[Half Framework](https://github.com/mieneticg101/Half) - A lightweight, secure, and high-performance Rust web framework.
"#,
        name
    );
    fs::write(project_path.join("README.md"), readme)?;

    println!("✅ Project created successfully!");
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  cargo run");

    Ok(())
}

/// Generate a new route handler
fn generate_route(path: &str, method: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Generating route:");
    println!("  Method: {}", method);
    println!("  Path: {}", path);

    // Generate function name from path
    let fn_name = path
        .trim_start_matches('/')
        .replace('/', "_")
        .replace(':', "")
        .replace('-', "_");

    let handler_code = format!(
        r#"
/// {} {} handler
async fn {}_handler(req: Request) -> Response {{
    // TODO: Implement handler logic
    Response::text("{} {}")
}}
"#,
        method, path, fn_name, method, path
    );

    println!("\n📋 Add this to your main.rs:\n");
    println!("{}", handler_code);
    println!("\nAnd register it in your router:");
    println!("  router.{}(\"{}\", {}_handler);", method.to_lowercase(), path, fn_name);

    Ok(())
}
