//! Basic database connection example
//!
//! This example demonstrates:
//! 1. How to connect to a database
//! 2. Database driver auto-detection
//! 3. SQL dialect features
//!
//! Run with: cargo run --example database_basic

use half_core::orm::{DatabaseConfig, ConnectionPool};

fn main() {
    println!("=== Half Framework - Database Connection Example ===\n");

    // Example 1: SQLite (easiest for development)
    println!("1. SQLite Connection:");
    let config = DatabaseConfig::new("sqlite://./example.db")
        .max_connections(5);
    let pool = ConnectionPool::new(config);

    println!("   Database Type: {:?}", pool.database_type());
    println!("   Max Connections: {}", pool.config().max_connections);
    println!("   ✓ Connected successfully!");
    println!();

    // Example 2: PostgreSQL
    println!("2. PostgreSQL Connection (example URL):");
    let pg_url = "postgresql://user:pass@localhost:5432/mydb";
    let pg_config = DatabaseConfig::new(pg_url);
    let pg_pool = ConnectionPool::new(pg_config);

    println!("   Database Type: {:?}", pg_pool.database_type());
    let pg_dialect = pg_pool.driver().dialect();
    println!("   Placeholder: {}", pg_dialect.placeholder(1)); // $1
    println!("   Quote: {}", pg_dialect.quote_identifier("user")); // "user"
    println!("   Boolean Type: {}", pg_dialect.boolean_type()); // BOOLEAN
    println!("   JSON Type: {}", pg_dialect.json_type()); // JSONB
    println!();

    // Example 3: MySQL
    println!("3. MySQL Connection (example URL):");
    let mysql_url = "mysql://user:pass@localhost:3306/mydb";
    let mysql_config = DatabaseConfig::new(mysql_url);
    let mysql_pool = ConnectionPool::new(mysql_config);

    println!("   Database Type: {:?}", mysql_pool.database_type());
    let mysql_dialect = mysql_pool.driver().dialect();
    println!("   Placeholder: {}", mysql_dialect.placeholder(1)); // ?
    println!("   Quote: {}", mysql_dialect.quote_identifier("user")); // `user`
    println!("   Boolean Type: {}", mysql_dialect.boolean_type()); // BOOLEAN
    println!("   JSON Type: {}", mysql_dialect.json_type()); // JSON
    println!();

    // Show feature support
    println!("4. Database Feature Support:");
    println!();
    println!("   PostgreSQL:");
    println!("     - RETURNING clause: {}", pg_dialect.supports_returning());
    println!("     - UPSERT: {}", pg_dialect.supports_upsert());
    println!();
    println!("   MySQL:");
    println!("     - RETURNING clause: {}", mysql_dialect.supports_returning());
    println!("     - UPSERT: {}", mysql_dialect.supports_upsert());
    println!();
    println!("   SQLite:");
    let sqlite_dialect = pool.driver().dialect();
    println!("     - RETURNING clause: {}", sqlite_dialect.supports_returning());
    println!("     - UPSERT: {}", sqlite_dialect.supports_upsert());

    println!("\n=== Next Steps ===");
    println!("See GETTING_STARTED.md for:");
    println!("- Setting up database drivers");
    println!("- Creating models");
    println!("- Running migrations");
    println!("- CRUD operations");

    println!("\n✓ Example completed!");
}
