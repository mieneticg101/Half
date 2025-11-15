# Getting Started with Half Framework

Complete guide to setting up and using the Half framework with database support.

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Database Setup](#database-setup)
4. [Creating Your First Model](#creating-your-first-model)
5. [Migrations](#migrations)
6. [CRUD Operations](#crud-operations)
7. [Running Examples](#running-examples)

## Installation

### Requirements

- Rust 1.85+ (for edition 2024 support)
- PostgreSQL, MySQL, or SQLite (depending on your choice)

### Step 1: Add Dependencies

Add Half to your `Cargo.toml`:

```toml
[dependencies]
half-core = "0.14"
tokio = { version = "1.42", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Choose your database driver (optional for now, required for production)
# sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "mysql", "sqlite"] }
# or
# rusqlite = { version = "0.30", features = ["bundled"] }
# or
# tokio-postgres = { version = "0.7", features = ["with-serde_json-1"] }
```

### Step 2: Create a New Project

```bash
cargo new my-half-app
cd my-half-app
```

## Quick Start

### 1. Basic Web Server

Create `src/main.rs`:

```rust
use half_core::{Router, Server, Request, Response};

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    router.get("/", |_req: Request| async {
        Response::text("Hello, Half!")
    });

    router.get("/api/health", |_req: Request| async {
        Response::json(&serde_json::json!({
            "status": "ok",
            "version": "0.14.0"
        })).unwrap()
    });

    Server::new(router)
        .bind(([127, 0, 0, 1], 3000))
        .run()
        .await
        .unwrap();
}
```

Run it:

```bash
cargo run
```

Visit: http://localhost:3000

## Database Setup

### Option 1: SQLite (Easiest for Development)

SQLite requires no setup - just use a file path:

```rust
use half_core::orm::{DatabaseConfig, ConnectionPool};

let config = DatabaseConfig::new("sqlite://./app.db")
    .max_connections(5);

let pool = ConnectionPool::new(config);
println!("Connected to: {:?}", pool.database_type()); // SQLite
```

### Option 2: PostgreSQL

**1. Install PostgreSQL:**

```bash
# macOS
brew install postgresql
brew services start postgresql

# Ubuntu/Debian
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql

# Create database
createdb myapp
```

**2. Connect in code:**

```rust
use half_core::orm::{DatabaseConfig, ConnectionPool};

let config = DatabaseConfig::new("postgresql://localhost/myapp")
    .max_connections(10);

let pool = ConnectionPool::new(config);
println!("Connected to: {:?}", pool.database_type()); // PostgreSQL
```

**3. With authentication:**

```rust
let config = DatabaseConfig::new(
    "postgresql://username:password@localhost:5432/myapp"
);
```

### Option 3: MySQL

**1. Install MySQL:**

```bash
# macOS
brew install mysql
brew services start mysql

# Ubuntu/Debian
sudo apt install mysql-server
sudo systemctl start mysql

# Create database
mysql -u root -p
CREATE DATABASE myapp;
```

**2. Connect in code:**

```rust
use half_core::orm::{DatabaseConfig, ConnectionPool};

let config = DatabaseConfig::new(
    "mysql://username:password@localhost:3306/myapp"
);

let pool = ConnectionPool::new(config);
println!("Connected to: {:?}", pool.database_type()); // MySQL
```

### Environment Variables

Best practice: Use environment variables for database URLs:

```bash
# .env file
DATABASE_URL=postgresql://localhost/myapp
```

```rust
use std::env;

let db_url = env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

let config = DatabaseConfig::new(db_url);
let pool = ConnectionPool::new(config);
```

## Creating Your First Model

### Step 1: Define the Model

```rust
use half_core::orm::{Model, Table, ColumnType, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct User {
    id: Option<i64>,
    name: String,
    email: String,
    age: i32,
    active: bool,
    created_at: i64,
    updated_at: i64,
}

impl Model for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn schema() -> Table {
        let mut table = Table::new("users");
        table.add_column("id", ColumnType::Integer, true, true);
        table.add_column("name", ColumnType::String(255), false, false);
        table.add_column("email", ColumnType::String(255), false, false);
        table.add_column("age", ColumnType::Integer, false, false);
        table.add_column("active", ColumnType::Boolean, false, false);
        table.add_column("created_at", ColumnType::Integer, false, false);
        table.add_column("updated_at", ColumnType::Integer, false, false);

        // Add constraints
        table.add_unique_constraint("email_unique", vec!["email".to_string()]);
        table.add_index("idx_active", vec!["active".to_string()], false);

        table
    }

    fn from_row(row: HashMap<String, Value>) -> Result<Self, half_core::orm::model::ModelError> {
        Ok(User {
            id: row.get("id").and_then(|v| v.as_i64()),
            name: row.get("name").and_then(|v| v.as_string()).unwrap_or("").to_string(),
            email: row.get("email").and_then(|v| v.as_string()).unwrap_or("").to_string(),
            age: row.get("age").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            active: row.get("active").and_then(|v| v.as_bool()).unwrap_or(false),
            created_at: row.get("created_at").and_then(|v| v.as_i64()).unwrap_or(0),
            updated_at: row.get("updated_at").and_then(|v| v.as_i64()).unwrap_or(0),
        })
    }

    fn to_values(&self) -> HashMap<String, Value> {
        let mut values = HashMap::new();
        if let Some(id) = self.id {
            values.insert("id".to_string(), Value::Integer(id));
        }
        values.insert("name".to_string(), Value::String(self.name.clone()));
        values.insert("email".to_string(), Value::String(self.email.clone()));
        values.insert("age".to_string(), Value::Integer(self.age as i64));
        values.insert("active".to_string(), Value::Boolean(self.active));
        values.insert("created_at".to_string(), Value::Integer(self.created_at));
        values.insert("updated_at".to_string(), Value::Integer(self.updated_at));
        values
    }

    fn primary_key(&self) -> Option<Value> {
        self.id.map(Value::Integer)
    }
}
```

### Step 2: Use the Model

```rust
// Create a new user
let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

let user = User {
    id: None,
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    age: 30,
    active: true,
    created_at: now,
    updated_at: now,
};

// Get schema information
let schema = User::schema();
println!("Table: {}", schema.name);
for column in &schema.columns {
    println!("  - {} ({:?})", column.name, column.column_type);
}
```

## Migrations

### Creating a Migration

```rust
use half_core::orm::{Migration, MigrationBuilder};

let migration = MigrationBuilder::new()
    .version("001")
    .description("Create users table")
    .up(|schema| {
        schema.create_table("users", |t| {
            t.id();                                    // Primary key
            t.string("name", 255);                     // VARCHAR(255)
            t.string("email", 255).unique();           // VARCHAR(255) UNIQUE
            t.integer("age").nullable();               // INTEGER NULL
            t.boolean("active").default("true");       // BOOLEAN DEFAULT TRUE
            t.text("bio").nullable();                  // TEXT NULL
            t.json("metadata").nullable();             // JSON/JSONB NULL
            t.timestamps();                            // created_at, updated_at
        });
    })
    .down(|schema| {
        schema.drop_table("users");
    })
    .build();
```

### Running Migrations

```rust
use half_core::orm::MigrationRunner;

// Create migration runner
let mut runner = MigrationRunner::new();

// Add migrations
runner.add_migration(migration_001);
runner.add_migration(migration_002);
runner.add_migration(migration_003);

// Run all pending migrations
// Note: Requires actual database connection (not yet fully implemented)
// runner.run(&mut connection)?;

// Check migration status
let status = runner.status();
println!("Applied migrations: {}", status.len());
```

### SQL Dialect Differences

The framework automatically generates correct SQL for each database:

```rust
use half_core::orm::drivers::{SqlDialect, DialectType};

// PostgreSQL
let pg = SqlDialect::new(DialectType::PostgreSQL);
assert_eq!(pg.placeholder(1), "$1");              // $1, $2, $3
assert_eq!(pg.quote_identifier("user"), "\"user\""); // Double quotes
assert_eq!(pg.json_type(), "JSONB");              // JSONB

// MySQL
let mysql = SqlDialect::new(DialectType::MySQL);
assert_eq!(mysql.placeholder(1), "?");            // ?
assert_eq!(mysql.quote_identifier("user"), "`user`"); // Backticks
assert_eq!(mysql.json_type(), "JSON");            // JSON

// SQLite
let sqlite = SqlDialect::new(DialectType::SQLite);
assert_eq!(sqlite.placeholder(1), "?");           // ?
assert_eq!(sqlite.quote_identifier("user"), "\"user\""); // Double quotes
assert_eq!(sqlite.json_type(), "TEXT");           // TEXT
```

## CRUD Operations

### Query Builder (Type-Safe)

```rust
use half_core::orm::Query;

// Note: These examples show the API, but require actual database connection

// SELECT * FROM users WHERE age > 18 ORDER BY name LIMIT 10
let users = Query::<User>::new()
    .where_gt("age", 18)
    .order_by("name", Order::Asc)
    .limit(10)
    .get(&mut executor)?;

// SELECT * FROM users WHERE email = 'john@example.com'
let user = Query::<User>::new()
    .where_eq("email", "john@example.com")
    .first(&mut executor)?;

// Complex query
let active_users = Query::<User>::new()
    .where_eq("active", true)
    .where_gt("created_at", timestamp)
    .where_in("age", vec![25, 30, 35])
    .order_by("created_at", Order::Desc)
    .limit(20)
    .offset(10)
    .get(&mut executor)?;
```

### Insert

```rust
use half_core::orm::Insert;

// Insert a new user
let insert = Insert::<User>::new()
    .value("name", "Jane Doe")
    .value("email", "jane@example.com")
    .value("age", 25)
    .value("active", true)
    .value("created_at", now)
    .value("updated_at", now)
    .execute(&mut executor)?;
```

### Update

```rust
use half_core::orm::Update;

// Update users
let update = Update::<User>::new()
    .set("active", false)
    .set("updated_at", now)
    .where_eq("email", "john@example.com")
    .execute(&mut executor)?;
```

### Delete

```rust
use half_core::orm::Delete;

// Delete users
let delete = Delete::<User>::new()
    .where_eq("active", false)
    .where_lt("created_at", old_timestamp)
    .execute(&mut executor)?;
```

## Complete Example: Web API with Database

```rust
use half_core::{
    Router, Server, Request, Response,
    orm::{DatabaseConfig, ConnectionPool},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
    age: i32,
}

#[tokio::main]
async fn main() {
    // Setup database
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./app.db".to_string());

    let config = DatabaseConfig::new(db_url)
        .max_connections(10);

    let pool = ConnectionPool::new(config);
    println!("✓ Connected to {:?}", pool.database_type());

    // Setup router
    let mut router = Router::new();

    // Health check
    router.get("/health", |_req: Request| async {
        Response::json(&serde_json::json!({
            "status": "healthy",
            "database": "connected"
        })).unwrap()
    });

    // List users
    router.get("/api/users", |_req: Request| async {
        // In real app: query users from database
        Response::json(&serde_json::json!({
            "users": []
        })).unwrap()
    });

    // Create user
    router.post("/api/users", |req: Request| async move {
        // In real app: parse body and insert to database
        Response::json(&serde_json::json!({
            "message": "User created"
        })).unwrap()
    });

    // Start server
    println!("✓ Server starting on http://127.0.0.1:3000");
    Server::new(router)
        .bind(([127, 0, 0, 1], 3000))
        .run()
        .await
        .unwrap();
}
```

## Running Examples

The framework includes several examples:

```bash
# Basic database connection
cargo run --example database_basic

# Migration system
cargo run --example database_migration

# List all examples
ls examples/
```

## Next Steps

1. **Read the Documentation**
   - [Multi-Database Support](MULTI_DATABASE.md)
   - [Performance Guide](PERFORMANCE.md)
   - [API Documentation](https://docs.rs/half-core)

2. **Explore Features**
   - Authentication (JWT, Basic, API Key)
   - WebSockets & SSE
   - Middleware system
   - Security features

3. **Production Deployment**
   - Add real database drivers (sqlx, tokio-postgres, etc.)
   - Implement connection pooling
   - Set up migrations
   - Configure TLS/HTTPS

## Common Issues

### "Connection not implemented"

The current version includes the driver architecture but requires actual database driver implementation (sqlx, rusqlite, etc.). The examples show how the API works, but full database integration requires:

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "sqlite"] }
```

### "Table doesn't exist"

Make sure to run migrations to create tables before querying:

```rust
// Run migrations first
runner.run(&mut connection)?;

// Then query
let users = Query::<User>::new().get(&mut executor)?;
```

### Connection Pool Exhausted

Increase max connections:

```rust
let config = DatabaseConfig::new(db_url)
    .max_connections(20)  // Increase from default 10
    .connect_timeout(60); // Increase timeout
```

## Support

- GitHub Issues: https://github.com/mieneticg101/Half/issues
- Documentation: See docs/ folder
- Examples: See examples/ folder

---

Happy coding with Half! 🚀
