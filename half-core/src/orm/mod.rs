//! ORM (Object-Relational Mapping) for Half framework
//!
//! Provides a type-safe, high-performance ORM system with support for:
//! - CRUD operations
//! - Relationships (OneToMany, ManyToOne, ManyToMany)
//! - Migrations
//! - Connection pooling
//! - Transactions

pub mod connection;
pub mod model;
pub mod query;
pub mod relations;
pub mod schema;
pub mod migrations;

pub use connection::{Connection, ConnectionPool, Transaction, DatabaseConfig};
pub use model::{Model, Entity};
pub use query::{Query, QueryExecutor};
pub use relations::{Relation, RelationType, HasOne, HasMany, BelongsTo, BelongsToMany};
pub use schema::{Schema, Column, ColumnType, Constraint, Index, Table};
pub use migrations::{Migration, MigrationRunner, MigrationVersion};
