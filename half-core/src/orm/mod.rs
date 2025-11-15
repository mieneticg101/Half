//! ORM (Object-Relational Mapping) for Half framework
//!
//! Provides a type-safe, high-performance ORM system with support for:
//! - CRUD operations
//! - Relationships (OneToMany, ManyToOne, ManyToMany)
//! - Migrations
//! - Connection pooling
//! - Transactions
//! - Multi-database support (PostgreSQL, MySQL, SQLite, etc.)

pub mod connection;
pub mod drivers;
pub mod migrations;
pub mod model;
pub mod query;
pub mod relations;
pub mod schema;

pub use connection::{Connection, ConnectionPool, DatabaseConfig, Transaction};
pub use drivers::{
    ConnectionInfo, DatabaseDriver, DatabaseType, DialectType, MySqlDriver, PostgresDriver,
    SqlDialect, SqliteDriver,
};
pub use migrations::{Migration, MigrationRunner, MigrationVersion};
pub use model::{Entity, Model};
pub use query::{Query, QueryExecutor};
pub use relations::{BelongsTo, BelongsToMany, HasMany, HasOne, Relation, RelationType};
pub use schema::{Column, ColumnType, Constraint, Index, Schema, Table};
