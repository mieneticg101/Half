# Multi-Database Support

The Half framework ORM provides Prisma-like multi-database support, allowing you to connect to PostgreSQL, MySQL, SQLite, and other databases using a unified API.

## Features

- **Multiple Database Support**: PostgreSQL, MySQL, SQLite, SQL Server, MongoDB (future)
- **Auto-Detection**: Automatically detects database type from connection URL
- **SQL Dialect Support**: Database-specific SQL generation
- **Connection Pooling**: Thread-safe connection pool with driver support
- **Type Safety**: Compile-time type checking for database operations
- **Zero-Cost Abstractions**: No runtime overhead

## Supported Databases

| Database   | URL Prefix              | Default Port | Status       |
|------------|-------------------------|--------------|--------------|
| PostgreSQL | `postgresql://` or `postgres://` | 5432 | ✅ Supported |
| MySQL      | `mysql://`              | 3306         | ✅ Supported |
| SQLite     | `sqlite://` or `file:`  | N/A          | ✅ Supported |
| SQL Server | `sqlserver://`          | 1433         | 🚧 Planned   |
| MongoDB    | `mongodb://`            | 27017        | 🚧 Planned   |

## Quick Start

### PostgreSQL

```rust
use half_core::orm::{DatabaseConfig, ConnectionPool};

// Auto-detect driver from URL
let config = DatabaseConfig::new("postgresql://user:password@localhost:5432/mydb");
let pool = ConnectionPool::new(config);

// Get the database type
println!("Database: {:?}", pool.database_type()); // PostgreSQL

// Get the SQL dialect
let dialect = pool.driver().dialect();
println!("Placeholder: {}", dialect.placeholder(1)); // $1
```

### MySQL

```rust
use half_core::orm::{DatabaseConfig, ConnectionPool};

let config = DatabaseConfig::new("mysql://user:password@localhost:3306/mydb");
let pool = ConnectionPool::new(config);

let dialect = pool.driver().dialect();
println!("Placeholder: {}", dialect.placeholder(1)); // ?
```

### SQLite

```rust
use half_core::orm::{DatabaseConfig, ConnectionPool};

// File-based SQLite
let config = DatabaseConfig::new("sqlite://./data/mydb.db");
let pool = ConnectionPool::new(config);

// In-memory SQLite
let config = DatabaseConfig::new("sqlite::memory:");
let pool = ConnectionPool::new(config);
```

## Connection URLs

### PostgreSQL URL Format

```
postgresql://[user[:password]@][host][:port][/database][?param=value&...]
```

**Examples:**
```rust
// Simple
"postgresql://localhost/mydb"

// With authentication
"postgresql://user:password@localhost/mydb"

// Custom port
"postgresql://user:password@localhost:5433/mydb"

// With SSL
"postgresql://user:password@localhost/mydb?sslmode=require"

// Full URL
"postgresql://admin:secret@db.example.com:5432/production?sslmode=verify-full&connect_timeout=10"
```

### MySQL URL Format

```
mysql://[user[:password]@][host][:port][/database][?param=value&...]
```

**Examples:**
```rust
// Simple
"mysql://localhost/mydb"

// With authentication
"mysql://user:password@localhost/mydb"

// Custom port
"mysql://user:password@localhost:3307/mydb"

// With charset
"mysql://user:password@localhost/mydb?charset=utf8mb4"

// Full URL
"mysql://admin:secret@db.example.com:3306/production?charset=utf8mb4&parseTime=true"
```

### SQLite URL Format

```
sqlite://[path][?param=value&...]
file:[path][?param=value&...]
```

**Examples:**
```rust
// Relative path
"sqlite://./mydb.db"
"file:./mydb.db"

// Absolute path
"sqlite:///var/lib/mydb.db"
"file:/var/lib/mydb.db"

// In-memory database
"sqlite::memory:"

// With query parameters
"sqlite://./mydb.db?mode=ro&cache=shared"
```

## SQL Dialects

Different databases use different SQL syntax. The dialect system handles these differences automatically.

### Placeholders

```rust
use half_core::orm::drivers::{SqlDialect, DialectType};

// PostgreSQL uses $1, $2, $3
let pg_dialect = SqlDialect::new(DialectType::PostgreSQL);
assert_eq!(pg_dialect.placeholder(1), "$1");
assert_eq!(pg_dialect.placeholder(2), "$2");

// MySQL uses ?
let mysql_dialect = SqlDialect::new(DialectType::MySQL);
assert_eq!(mysql_dialect.placeholder(1), "?");

// SQL Server uses @p1, @p2
let mssql_dialect = SqlDialect::new(DialectType::SqlServer);
assert_eq!(mssql_dialect.placeholder(1), "@p1");
```

### Identifier Quoting

```rust
// PostgreSQL uses double quotes
let pg_dialect = SqlDialect::new(DialectType::PostgreSQL);
assert_eq!(pg_dialect.quote_identifier("user"), "\"user\"");

// MySQL uses backticks
let mysql_dialect = SqlDialect::new(DialectType::MySQL);
assert_eq!(mysql_dialect.quote_identifier("user"), "`user`");

// SQL Server uses square brackets
let mssql_dialect = SqlDialect::new(DialectType::SqlServer);
assert_eq!(mssql_dialect.quote_identifier("user"), "[user]");
```

### Type Mappings

Different databases have different type names:

| Type    | PostgreSQL | MySQL     | SQLite  | SQL Server |
|---------|------------|-----------|---------|------------|
| Boolean | `BOOLEAN`  | `BOOLEAN` | `INTEGER` | `BIT`    |
| JSON    | `JSONB`    | `JSON`    | `TEXT`  | `NVARCHAR(MAX)` |
| UUID    | `UUID`     | `CHAR(36)`| `TEXT`  | `UNIQUEIDENTIFIER` |
| Text    | `TEXT`     | `TEXT`    | `TEXT`  | `NVARCHAR(MAX)` |

```rust
let pg_dialect = SqlDialect::new(DialectType::PostgreSQL);
assert_eq!(pg_dialect.boolean_type(), "BOOLEAN");
assert_eq!(pg_dialect.json_type(), "JSONB");
assert_eq!(pg_dialect.uuid_type(), "UUID");
```

### Feature Detection

```rust
let pg_dialect = SqlDialect::new(DialectType::PostgreSQL);

// PostgreSQL supports RETURNING clause
assert!(pg_dialect.supports_returning());

// All SQL databases support UPSERT (ON CONFLICT / ON DUPLICATE KEY)
assert!(pg_dialect.supports_upsert());

// Get the UPSERT syntax
let upsert = pg_dialect.upsert_clause(&["email"], &["name", "updated_at"]);
// PostgreSQL: ON CONFLICT (email) DO UPDATE SET name = EXCLUDED.name, updated_at = EXCLUDED.updated_at

let mysql_dialect = SqlDialect::new(DialectType::MySQL);
let upsert = mysql_dialect.upsert_clause(&["email"], &["name", "updated_at"]);
// MySQL: ON DUPLICATE KEY UPDATE name = VALUES(name), updated_at = VALUES(updated_at)
```

## Advanced Usage

### Explicit Driver Selection

You can explicitly specify a driver instead of auto-detection:

```rust
use half_core::orm::{DatabaseConfig, ConnectionPool, PostgresDriver};
use std::sync::Arc;

let config = DatabaseConfig::new("postgresql://localhost/mydb");
let driver = Arc::new(PostgresDriver::new());
let pool = ConnectionPool::with_driver(config, driver);
```

### Custom Connection Pool

Configure connection pool settings:

```rust
use half_core::orm::DatabaseConfig;

let config = DatabaseConfig::new("postgresql://localhost/mydb")
    .max_connections(20)
    .min_connections(5)
    .connect_timeout(30);
```

### Database-Specific Features

#### PostgreSQL - RETURNING Clause

```rust
// PostgreSQL supports RETURNING for INSERT/UPDATE/DELETE
let dialect = SqlDialect::new(DialectType::PostgreSQL);
if dialect.supports_returning() {
    let sql = "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, created_at";
}
```

#### MySQL - ON DUPLICATE KEY UPDATE

```rust
let dialect = SqlDialect::new(DialectType::MySQL);
let upsert = dialect.upsert_clause(&["email"], &["name"]);
// Result: ON DUPLICATE KEY UPDATE name = VALUES(name)
```

#### SQLite - In-Memory Databases

```rust
use half_core::orm::drivers::SqliteDriver;

// Get special SQLite URLs
let in_memory = SqliteDriver::in_memory_url(); // "sqlite::memory:"
let temporary = SqliteDriver::temporary_url(); // "sqlite:"
```

## Migration Support

Migrations automatically use the correct SQL dialect:

```rust
use half_core::orm::{Migration, MigrationBuilder, ConnectionPool, DatabaseConfig};

let config = DatabaseConfig::new("postgresql://localhost/mydb");
let pool = ConnectionPool::new(config);

let migration = MigrationBuilder::new()
    .version("001")
    .description("Create users table")
    .up(|schema| {
        schema.create_table("users", |t| {
            t.id();
            t.string("name", 255);
            t.string("email", 255).unique();
            t.boolean("active").default("TRUE"); // Automatically uses correct boolean type
            t.json("metadata"); // Automatically uses JSONB for PostgreSQL, JSON for MySQL, TEXT for SQLite
            t.timestamps();
        });
    })
    .build();
```

## Testing

The framework includes comprehensive tests for all database drivers:

```bash
# Run all tests
cargo test

# Run driver-specific tests
cargo test --test drivers

# Test individual drivers
cargo test postgres
cargo test mysql
cargo test sqlite
```

## Examples

### Complete Application

```rust
use half_core::{
    Router, Server, Request, Response,
    orm::{DatabaseConfig, ConnectionPool, Model, Query},
};

#[derive(Model)]
struct User {
    id: Option<i64>,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    // Auto-detect database from environment or use default
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./app.db".to_string());

    let config = DatabaseConfig::new(db_url)
        .max_connections(10);

    let pool = ConnectionPool::new(config);

    println!("Connected to: {:?}", pool.database_type());

    let mut router = Router::new();

    router.get("/users", |req: Request| async {
        // Query will use correct SQL dialect automatically
        let users = Query::<User>::new()
            .where_eq("active", true)
            .order_by("created_at", Order::Desc)
            .limit(10)
            .get(&mut executor)?;

        Response::json(&users).unwrap()
    });

    Server::new(router)
        .bind(([127, 0, 0, 1], 3000))
        .run()
        .await
        .unwrap();
}
```

## Performance

The multi-database support is designed with zero-cost abstractions:

- **No Runtime Overhead**: Driver selection happens at pool creation
- **Inline Functions**: Hot path functions are inlined
- **Compile-Time Optimization**: SQL dialect methods are optimized at compile time
- **Efficient String Handling**: Minimal allocations in SQL generation

See [PERFORMANCE.md](PERFORMANCE.md) for detailed performance metrics.

## Future Enhancements

- [ ] SQL Server driver implementation
- [ ] MongoDB driver implementation
- [ ] Connection pooling with actual database libraries (sqlx, tokio-postgres)
- [ ] Prepared statement caching
- [ ] Automatic database migrations
- [ ] Read replicas support
- [ ] Sharding support

## Contributing

Contributions are welcome! To add support for a new database:

1. Create a new driver in `half-core/src/orm/drivers/`
2. Implement the `DatabaseDriver` trait
3. Add dialect support in `dialect.rs`
4. Add URL parsing logic
5. Write comprehensive tests
6. Update this documentation

## Resources

- [Prisma Database Connectors](https://www.prisma.io/docs/concepts/database-connectors)
- [PostgreSQL Connection Strings](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING)
- [MySQL Connection URLs](https://dev.mysql.com/doc/refman/8.0/en/connecting-using-uri-or-key-value-pairs.html)
- [SQLite URI Filenames](https://www.sqlite.org/uri.html)
