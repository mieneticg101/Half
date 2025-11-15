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
    /// Generate a new middleware
    Middleware {
        /// Middleware name (e.g., "Auth", "`RateLimit`")
        name: String,
    },
    /// Generate a new controller
    Controller {
        /// Controller name (e.g., "User", "Post")
        name: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => {
            create_project(&name)?;
        }
        Commands::Route { path, method } => {
            generate_route(&path, &method);
        }
        Commands::Middleware { name } => {
            generate_middleware(&name);
        }
        Commands::Controller { name } => {
            generate_controller(&name);
        }
    }

    Ok(())
}

/// Create a new Half project
fn create_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Creating new Half project: {name}");

    // Create project directory
    fs::create_dir(name)?;
    let project_path = Path::new(name);

    // Create Cargo.toml
    let cargo_toml = format!(
        "[package]\n\
name = \"{name}\"\n\
version = \"0.1.0\"\n\
edition = \"2021\"\n\
\n\
[dependencies]\n\
half-core = \"0.1\"\n\
tokio = {{ version = \"1.42\", features = [\"full\"] }}\n\
serde = {{ version = \"1.0\", features = [\"derive\"] }}\n\
serde_json = \"1.0\"\n\
\n\
[[bin]]\n\
name = \"{name}\"\n\
path = \"src/main.rs\"\n"
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
    let gitignore = r"/target
Cargo.lock
*.swp
.DS_Store
";
    fs::write(project_path.join(".gitignore"), gitignore)?;

    // Create README.md
    let readme = format!(
        r"# {name}

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
"
    );
    fs::write(project_path.join("README.md"), readme)?;

    println!("✅ Project created successfully!");
    println!("\nNext steps:");
    println!("  cd {name}");
    println!("  cargo run");

    Ok(())
}

/// Generate a new route handler
fn generate_route(path: &str, method: &str) {
    println!("📝 Generating route:");
    println!("  Method: {method}");
    println!("  Path: {path}");

    // Generate function name from path
    let fn_name = path
        .trim_start_matches('/')
        .replace('/', "_")
        .replace(':', "")
        .replace('-', "_");

    let handler_code = format!(
        "\n\
/// {method} {path} handler\n\
async fn {fn_name}_handler(req: Request) -> Response {{\n\
    // TODO: Implement handler logic\n\
    Response::text(\"{method} {path}\")\n\
}}\n"
    );

    println!("\n📋 Add this to your main.rs:\n");
    println!("{handler_code}");
    println!("\nAnd register it in your router:");
    println!(
        "  router.{}(\"{path}\", {fn_name}_handler);",
        method.to_lowercase(),
    );
}

/// Generate a new middleware
fn generate_middleware(name: &str) {
    println!("🔧 Generating middleware: {name}");

    let name_lower = name.to_lowercase();
    let middleware_code = format!(
        "use half_core::{{middleware::Middleware, Request, Response, Result}};\n\
use std::future::Future;\n\
use std::pin::Pin;\n\
\n\
/// {name} middleware\n\
pub struct {name} {{\n\
    // Add configuration fields here\n\
}}\n\
\n\
impl {name} {{\n\
    /// Create new {name} middleware\n\
    pub fn new() -> Self {{\n\
        Self {{\n\
            // Initialize fields here\n\
        }}\n\
    }}\n\
}}\n\
\n\
impl Default for {name} {{\n\
    fn default() -> Self {{\n\
        Self::new()\n\
    }}\n\
}}\n\
\n\
impl Middleware for {name} {{\n\
    fn handle(\n\
        &self,\n\
        req: Request,\n\
        next: half_core::middleware::Next,\n\
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {{\n\
        Box::pin(async move {{\n\
            // Process request before handler\n\
            println!(\"{name} middleware: processing request\");\n\
\n\
            // Call the next middleware or handler\n\
            let response = next(req).await?;\n\
\n\
            // Process response after handler\n\
            println!(\"{name} middleware: processing response\");\n\
\n\
            Ok(response)\n\
        }})\n\
    }}\n\
}}\n\
\n\
#[cfg(test)]\n\
mod tests {{\n\
    use super::*;\n\
\n\
    #[test]\n\
    fn test_{name_lower}_creation() {{\n\
        let _middleware = {name}::new();\n\
        assert!(true);\n\
    }}\n\
}}\n"
    );

    println!("\n📋 Middleware code:\n");
    println!("{middleware_code}");
    println!("\n💡 Usage:");
    println!("  let mut router = Router::new();");
    println!("  router.use_middleware({name}::new());");
}

/// Generate a new controller
fn generate_controller(name: &str) {
    println!("📦 Generating controller: {name}");

    let controller_name = name.to_lowercase();
    let controller_code = format!(
        "use half_core::{{Request, Response, Result}};\n\
use serde_json::json;\n\
\n\
/// {name} controller\n\
pub struct {name}Controller;\n\
\n\
impl {name}Controller {{\n\
    /// List all {controller_name}s\n\
    pub async fn index(_req: Request) -> Response {{\n\
        // TODO: Fetch {controller_name}s from database\n\
        Response::json(&json!({{\n\
            \"{controller_name}s\": []\n\
        }}))\n\
        .unwrap_or_else(|_| Response::internal_error())\n\
    }}\n\
\n\
    /// Get a single {name} by ID\n\
    pub async fn show(req: Request) -> Response {{\n\
        let id = req.param(\"id\").unwrap_or(\"unknown\");\n\
\n\
        // TODO: Fetch {name} from database\n\
        Response::json(&json!({{\n\
            \"{controller_name}_id\": id,\n\
            \"message\": \"TODO: Implement show\"\n\
        }}))\n\
        .unwrap_or_else(|_| Response::internal_error())\n\
    }}\n\
\n\
    /// Create a new {name}\n\
    pub async fn create(_req: Request) -> Response {{\n\
        // TODO: Parse request body and create {name}\n\
        Response::created(None)\n\
            .body(b\"Created\".to_vec())\n\
    }}\n\
\n\
    /// Update a {name}\n\
    pub async fn update(req: Request) -> Response {{\n\
        let id = req.param(\"id\").unwrap_or(\"unknown\");\n\
\n\
        // TODO: Parse request body and update {name}\n\
        Response::json(&json!({{\n\
            \"{controller_name}_id\": id,\n\
            \"message\": \"Updated\"\n\
        }}))\n\
        .unwrap_or_else(|_| Response::internal_error())\n\
    }}\n\
\n\
    /// Delete a {name}\n\
    pub async fn destroy(req: Request) -> Response {{\n\
        let id = req.param(\"id\").unwrap_or(\"unknown\");\n\
\n\
        // TODO: Delete {name} from database\n\
        Response::no_content()\n\
    }}\n\
}}\n\
\n\
#[cfg(test)]\n\
mod tests {{\n\
    use super::*;\n\
\n\
    #[test]\n\
    fn test_controller_exists() {{\n\
        // Smoke test to ensure controller compiles\n\
        assert!(true);\n\
    }}\n\
}}\n"
    );

    println!("\n📋 Controller code:\n");
    println!("{controller_code}");
    println!("\n💡 Register routes:");
    println!("  router.group(\"/{controller_name}s\", |group| {{");
    println!("      group.get(\"/\", {name}Controller::index);");
    println!("      group.get(\"/:id\", {name}Controller::show);");
    println!("      group.post(\"/\", {name}Controller::create);");
    println!("      group.put(\"/:id\", {name}Controller::update);");
    println!("      group.delete(\"/:id\", {name}Controller::destroy);");
    println!("  }});");
}
