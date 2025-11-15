//! Database SQL dialects example
//!
//! This example demonstrates:
//! 1. SQL dialect differences between databases
//! 2. SQL generation for different databases
//! 3. Feature support detection
//!
//! Run with: cargo run --example database_dialects

use half_core::orm::drivers::{SqlDialect, DialectType};
use half_core::orm::model::Value;

fn main() {
    println!("=== Half Framework - SQL Dialects Example ===\n");

    let dialects = vec![
        ("PostgreSQL", DialectType::PostgreSQL),
        ("MySQL", DialectType::MySQL),
        ("SQLite", DialectType::SQLite),
        ("SQL Server", DialectType::SqlServer),
    ];

    // 1. Placeholders
    println!("1. Parameter Placeholders:");
    println!();
    for (name, dtype) in &dialects {
        let dialect = SqlDialect::new(*dtype);
        println!("   {}: {} {} {}",
            name,
            dialect.placeholder(1),
            dialect.placeholder(2),
            dialect.placeholder(3)
        );
    }

    // 2. Identifier quoting
    println!("\n2. Identifier Quoting:");
    println!();
    for (name, dtype) in &dialects {
        let dialect = SqlDialect::new(*dtype);
        println!("   {}: {}",
            name,
            dialect.quote_identifier("user")
        );
    }

    // 3. Type mappings
    println!("\n3. Type Mappings:");
    println!();
    println!("   | Type    | PostgreSQL | MySQL     | SQLite  | SQL Server |");
    println!("   |---------|------------|-----------|---------|------------|");

    print!("   | Boolean | ");
    for (_, dtype) in &dialects {
        let d = SqlDialect::new(*dtype);
        print!("{:10} | ", d.boolean_type());
    }
    println!();

    print!("   | JSON    | ");
    for (_, dtype) in &dialects {
        let d = SqlDialect::new(*dtype);
        print!("{:10} | ", d.json_type());
    }
    println!();

    print!("   | UUID    | ");
    for (_, dtype) in &dialects {
        let d = SqlDialect::new(*dtype);
        print!("{:10} | ", d.uuid_type());
    }
    println!();

    print!("   | Text    | ");
    for (_, dtype) in &dialects {
        let d = SqlDialect::new(*dtype);
        print!("{:10} | ", d.text_type());
    }
    println!();

    // 4. Feature support
    println!("\n4. Feature Support:");
    println!();
    println!("   | Feature   | PostgreSQL | MySQL | SQLite | SQL Server |");
    println!("   |-----------|------------|-------|--------|------------|");

    print!("   | RETURNING | ");
    for (_, dtype) in &dialects {
        let d = SqlDialect::new(*dtype);
        print!("{:10} | ", if d.supports_returning() { "✓" } else { "✗" });
    }
    println!();

    print!("   | UPSERT    | ");
    for (_, dtype) in &dialects {
        let d = SqlDialect::new(*dtype);
        print!("{:10} | ", if d.supports_upsert() { "✓" } else { "✗" });
    }
    println!();

    // 5. UPSERT syntax
    println!("\n5. UPSERT Syntax Examples:");
    println!();

    let pg = SqlDialect::new(DialectType::PostgreSQL);
    if let Some(upsert) = pg.upsert_clause(&["email"], &["name", "updated_at"]) {
        println!("   PostgreSQL:");
        println!("     {}", upsert);
    }
    println!();

    let mysql = SqlDialect::new(DialectType::MySQL);
    if let Some(upsert) = mysql.upsert_clause(&["email"], &["name", "updated_at"]) {
        println!("   MySQL:");
        println!("     {}", upsert);
    }
    println!();

    let sqlite = SqlDialect::new(DialectType::SQLite);
    if let Some(upsert) = sqlite.upsert_clause(&["email"], &["name", "updated_at"]) {
        println!("   SQLite:");
        println!("     {}", upsert);
    }

    // 6. Value to SQL
    println!("\n6. Value to SQL:");
    println!();

    let values = vec![
        ("NULL", Value::Null),
        ("Boolean", Value::Boolean(true)),
        ("Integer", Value::Integer(42)),
        ("String", Value::String("hello".to_string())),
    ];

    println!("   PostgreSQL:");
    for (name, value) in &values {
        println!("     {}: {}", name, pg.value_to_sql(value));
    }

    println!("\n✓ Example completed!");
}
