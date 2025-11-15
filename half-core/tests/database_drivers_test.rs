//! Comprehensive integration tests for database drivers
//!
//! Tests all multi-database functionality including:
//! - Driver detection from URLs
//! - Connection pool with drivers
//! - SQL dialect features
//! - Database-specific features

use half_core::orm::{
    DatabaseConfig, ConnectionPool, DatabaseType,
    drivers::{
        DatabaseDriver, PostgresDriver, MySqlDriver, SqliteDriver,
        SqlDialect, DialectType, ConnectionInfo,
    },
};
use std::sync::Arc;

#[test]
fn test_connection_pool_sqlite_auto_detection() {
    let config = DatabaseConfig::new("sqlite://./test.db");
    let pool = ConnectionPool::new(config);

    assert_eq!(pool.database_type(), DatabaseType::SQLite);
    assert_eq!(pool.config().url, "sqlite://./test.db");
}

#[test]
fn test_connection_pool_postgres_auto_detection() {
    let config = DatabaseConfig::new("postgresql://localhost/test");
    let pool = ConnectionPool::new(config);

    assert_eq!(pool.database_type(), DatabaseType::PostgreSQL);
}

#[test]
fn test_connection_pool_mysql_auto_detection() {
    let config = DatabaseConfig::new("mysql://localhost/test");
    let pool = ConnectionPool::new(config);

    assert_eq!(pool.database_type(), DatabaseType::MySQL);
}

#[test]
fn test_connection_pool_with_explicit_driver() {
    let config = DatabaseConfig::new("custom://test");
    let driver = Arc::new(PostgresDriver::new());
    let pool = ConnectionPool::with_driver(config, driver);

    assert_eq!(pool.database_type(), DatabaseType::PostgreSQL);
}

#[test]
fn test_connection_pool_configuration() {
    let config = DatabaseConfig::new("sqlite://./test.db")
        .max_connections(20)
        .min_connections(5)
        .connect_timeout(60);

    let pool = ConnectionPool::new(config);

    assert_eq!(pool.config().max_connections, 20);
    assert_eq!(pool.config().min_connections, 5);
    assert_eq!(pool.config().connect_timeout, 60);
}

#[test]
fn test_database_type_from_url() {
    assert_eq!(
        DatabaseType::from_url("postgresql://localhost/db").unwrap(),
        DatabaseType::PostgreSQL
    );
    assert_eq!(
        DatabaseType::from_url("postgres://localhost/db").unwrap(),
        DatabaseType::PostgreSQL
    );
    assert_eq!(
        DatabaseType::from_url("mysql://localhost/db").unwrap(),
        DatabaseType::MySQL
    );
    assert_eq!(
        DatabaseType::from_url("sqlite://./db").unwrap(),
        DatabaseType::SQLite
    );
    assert_eq!(
        DatabaseType::from_url("file:./db").unwrap(),
        DatabaseType::SQLite
    );
}

#[test]
fn test_database_type_default_ports() {
    assert_eq!(DatabaseType::PostgreSQL.default_port(), 5432);
    assert_eq!(DatabaseType::MySQL.default_port(), 3306);
    assert_eq!(DatabaseType::SqlServer.default_port(), 1433);
    assert_eq!(DatabaseType::MongoDB.default_port(), 27017);
}

#[test]
fn test_postgres_driver() {
    let driver = PostgresDriver::new();
    assert_eq!(driver.database_type(), DatabaseType::PostgreSQL);

    let dialect = driver.dialect();
    assert_eq!(dialect.placeholder(1), "$1");
    assert_eq!(dialect.placeholder(2), "$2");
    assert_eq!(dialect.quote_identifier("user"), "\"user\"");
    assert_eq!(dialect.boolean_type(), "BOOLEAN");
    assert_eq!(dialect.json_type(), "JSONB");
    assert!(dialect.supports_returning());
    assert!(dialect.supports_upsert());
}

#[test]
fn test_mysql_driver() {
    let driver = MySqlDriver::new();
    assert_eq!(driver.database_type(), DatabaseType::MySQL);

    let dialect = driver.dialect();
    assert_eq!(dialect.placeholder(1), "?");
    assert_eq!(dialect.placeholder(5), "?");
    assert_eq!(dialect.quote_identifier("user"), "`user`");
    assert_eq!(dialect.boolean_type(), "BOOLEAN");
    assert_eq!(dialect.json_type(), "JSON");
    assert!(!dialect.supports_returning());
    assert!(dialect.supports_upsert());
}

#[test]
fn test_sqlite_driver() {
    let driver = SqliteDriver::new();
    assert_eq!(driver.database_type(), DatabaseType::SQLite);

    let dialect = driver.dialect();
    assert_eq!(dialect.placeholder(1), "?");
    assert_eq!(dialect.quote_identifier("user"), "\"user\"");
    assert_eq!(dialect.boolean_type(), "INTEGER");
    assert_eq!(dialect.json_type(), "TEXT");
    assert!(dialect.supports_returning());
    assert!(dialect.supports_upsert());
}

#[test]
fn test_postgres_url_parsing_simple() {
    let driver = PostgresDriver::new();
    let info = driver.parse_url("postgresql://localhost/mydb").unwrap();

    assert_eq!(info.db_type, DatabaseType::PostgreSQL);
    assert_eq!(info.host, "localhost");
    assert_eq!(info.port, 5432);
    assert_eq!(info.database, "mydb");
    assert_eq!(info.username, None);
    assert_eq!(info.password, None);
}

#[test]
fn test_postgres_url_parsing_with_auth() {
    let driver = PostgresDriver::new();
    let info = driver.parse_url("postgresql://user:pass@localhost:5433/mydb").unwrap();

    assert_eq!(info.db_type, DatabaseType::PostgreSQL);
    assert_eq!(info.host, "localhost");
    assert_eq!(info.port, 5433);
    assert_eq!(info.database, "mydb");
    assert_eq!(info.username, Some("user".to_string()));
    assert_eq!(info.password, Some("pass".to_string()));
}

#[test]
fn test_mysql_url_parsing() {
    let driver = MySqlDriver::new();
    let info = driver.parse_url("mysql://user:pass@localhost:3307/mydb").unwrap();

    assert_eq!(info.db_type, DatabaseType::MySQL);
    assert_eq!(info.host, "localhost");
    assert_eq!(info.port, 3307);
    assert_eq!(info.database, "mydb");
    assert_eq!(info.username, Some("user".to_string()));
    assert_eq!(info.password, Some("pass".to_string()));
}

#[test]
fn test_sqlite_url_parsing() {
    let driver = SqliteDriver::new();
    let info = driver.parse_url("sqlite://./mydb.db").unwrap();

    assert_eq!(info.db_type, DatabaseType::SQLite);
    assert_eq!(info.database, "./mydb.db");
}

#[test]
fn test_sqlite_in_memory_url() {
    assert_eq!(SqliteDriver::in_memory_url(), "sqlite::memory:");
}

#[test]
fn test_sql_dialect_placeholders() {
    let pg = SqlDialect::new(DialectType::PostgreSQL);
    assert_eq!(pg.placeholder(1), "$1");
    assert_eq!(pg.placeholder(10), "$10");

    let mysql = SqlDialect::new(DialectType::MySQL);
    assert_eq!(mysql.placeholder(1), "?");
    assert_eq!(mysql.placeholder(100), "?");

    let mssql = SqlDialect::new(DialectType::SqlServer);
    assert_eq!(mssql.placeholder(1), "@p1");
    assert_eq!(mssql.placeholder(5), "@p5");
}

#[test]
fn test_sql_dialect_identifier_quoting() {
    let pg = SqlDialect::new(DialectType::PostgreSQL);
    assert_eq!(pg.quote_identifier("table"), "\"table\"");

    let mysql = SqlDialect::new(DialectType::MySQL);
    assert_eq!(mysql.quote_identifier("table"), "`table`");

    let sqlite = SqlDialect::new(DialectType::SQLite);
    assert_eq!(sqlite.quote_identifier("table"), "\"table\"");

    let mssql = SqlDialect::new(DialectType::SqlServer);
    assert_eq!(mssql.quote_identifier("table"), "[table]");
}

#[test]
fn test_sql_dialect_type_mappings() {
    let pg = SqlDialect::new(DialectType::PostgreSQL);
    assert_eq!(pg.boolean_type(), "BOOLEAN");
    assert_eq!(pg.json_type(), "JSONB");
    assert_eq!(pg.uuid_type(), "UUID");
    assert_eq!(pg.text_type(), "TEXT");

    let mysql = SqlDialect::new(DialectType::MySQL);
    assert_eq!(mysql.boolean_type(), "BOOLEAN");
    assert_eq!(mysql.json_type(), "JSON");
    assert_eq!(mysql.uuid_type(), "CHAR(36)");
    assert_eq!(mysql.text_type(), "TEXT");

    let sqlite = SqlDialect::new(DialectType::SQLite);
    assert_eq!(sqlite.boolean_type(), "INTEGER");
    assert_eq!(sqlite.json_type(), "TEXT");
    assert_eq!(sqlite.uuid_type(), "TEXT");
    assert_eq!(sqlite.text_type(), "TEXT");
}

#[test]
fn test_sql_dialect_auto_increment() {
    let pg = SqlDialect::new(DialectType::PostgreSQL);
    assert_eq!(pg.auto_increment(), "SERIAL");

    let mysql = SqlDialect::new(DialectType::MySQL);
    assert_eq!(mysql.auto_increment(), "AUTO_INCREMENT");

    let sqlite = SqlDialect::new(DialectType::SQLite);
    assert_eq!(sqlite.auto_increment(), "AUTOINCREMENT");
}

#[test]
fn test_sql_dialect_limit_clause() {
    let pg = SqlDialect::new(DialectType::PostgreSQL);
    assert_eq!(pg.limit_clause(10, None), "LIMIT 10");
    assert_eq!(pg.limit_clause(10, Some(20)), "LIMIT 10 OFFSET 20");

    let mysql = SqlDialect::new(DialectType::MySQL);
    assert_eq!(mysql.limit_clause(10, None), "LIMIT 10");
    assert_eq!(mysql.limit_clause(10, Some(20)), "LIMIT 10 OFFSET 20");
}

#[test]
fn test_sql_dialect_upsert_postgres() {
    let pg = SqlDialect::new(DialectType::PostgreSQL);
    let upsert = pg.upsert_clause(&["email"], &["name", "updated_at"]).unwrap();

    assert!(upsert.contains("ON CONFLICT"));
    assert!(upsert.contains("email"));
    assert!(upsert.contains("DO UPDATE SET"));
    assert!(upsert.contains("EXCLUDED.name"));
}

#[test]
fn test_sql_dialect_upsert_mysql() {
    let mysql = SqlDialect::new(DialectType::MySQL);
    let upsert = mysql.upsert_clause(&["email"], &["name", "updated_at"]).unwrap();

    assert!(upsert.contains("ON DUPLICATE KEY UPDATE"));
    assert!(upsert.contains("name = VALUES(name)"));
}

#[test]
fn test_sql_dialect_upsert_sqlite() {
    let sqlite = SqlDialect::new(DialectType::SQLite);
    let upsert = sqlite.upsert_clause(&["email"], &["name"]).unwrap();

    assert!(upsert.contains("ON CONFLICT"));
    assert!(upsert.contains("DO UPDATE SET"));
}

#[test]
fn test_sql_dialect_returning_clause() {
    let pg = SqlDialect::new(DialectType::PostgreSQL);
    assert_eq!(pg.returning_clause(&["id", "created_at"]), Some("RETURNING id, created_at".to_string()));

    let sqlite = SqlDialect::new(DialectType::SQLite);
    assert_eq!(sqlite.returning_clause(&["id"]), Some("RETURNING id".to_string()));

    let mysql = SqlDialect::new(DialectType::MySQL);
    assert_eq!(mysql.returning_clause(&["id"]), None);
}

#[test]
fn test_sql_dialect_feature_support() {
    let pg = SqlDialect::new(DialectType::PostgreSQL);
    assert!(pg.supports_returning());
    assert!(pg.supports_upsert());

    let mysql = SqlDialect::new(DialectType::MySQL);
    assert!(!mysql.supports_returning());
    assert!(mysql.supports_upsert());

    let sqlite = SqlDialect::new(DialectType::SQLite);
    assert!(sqlite.supports_returning());
    assert!(sqlite.supports_upsert());
}

#[test]
fn test_connection_info_creation() {
    let info = ConnectionInfo::new(
        DatabaseType::PostgreSQL,
        "localhost",
        "mydb"
    );

    assert_eq!(info.db_type, DatabaseType::PostgreSQL);
    assert_eq!(info.host, "localhost");
    assert_eq!(info.database, "mydb");
    assert_eq!(info.port, 5432); // Default PostgreSQL port
}

#[test]
fn test_connection_pool_get_and_release() {
    let config = DatabaseConfig::new("sqlite::memory:");
    let pool = ConnectionPool::new(config);

    let initial_stats = pool.stats();
    assert_eq!(initial_stats.total_connections, 0);
    assert_eq!(initial_stats.active_connections, 0);

    // Get a connection
    let conn = pool.get().unwrap();
    let after_get_stats = pool.stats();
    assert_eq!(after_get_stats.total_connections, 1);
    assert_eq!(after_get_stats.active_connections, 1);

    // Release connection
    pool.release(&conn);
    let after_release_stats = pool.stats();
    assert_eq!(after_release_stats.total_connections, 1);
    assert_eq!(after_release_stats.active_connections, 0);
}

#[test]
fn test_connection_pool_multiple_connections() {
    let config = DatabaseConfig::new("sqlite::memory:")
        .max_connections(3);
    let pool = ConnectionPool::new(config);

    let conn1 = pool.get().unwrap();
    let _conn2 = pool.get().unwrap();
    let _conn3 = pool.get().unwrap();

    let stats = pool.stats();
    assert_eq!(stats.total_connections, 3);
    assert_eq!(stats.active_connections, 3);

    // Should fail - pool exhausted
    let result = pool.get();
    assert!(result.is_err());

    // Release one and try again
    pool.release(&conn1);
    let conn4 = pool.get().unwrap();
    assert!(conn4.is_in_use());
}

#[test]
fn test_all_database_types_have_drivers() {
    // Ensure we can create drivers for all supported types
    let pg = PostgresDriver::new();
    assert_eq!(pg.database_type(), DatabaseType::PostgreSQL);

    let mysql = MySqlDriver::new();
    assert_eq!(mysql.database_type(), DatabaseType::MySQL);

    let sqlite = SqliteDriver::new();
    assert_eq!(sqlite.database_type(), DatabaseType::SQLite);
}

#[test]
fn test_dialect_value_to_sql() {
    use half_core::orm::model::Value;

    let pg = SqlDialect::new(DialectType::PostgreSQL);

    assert_eq!(pg.value_to_sql(&Value::Null), "NULL");
    assert_eq!(pg.value_to_sql(&Value::Boolean(true)), "TRUE");
    assert_eq!(pg.value_to_sql(&Value::Boolean(false)), "FALSE");
    assert_eq!(pg.value_to_sql(&Value::Integer(42)), "42");
    assert_eq!(pg.value_to_sql(&Value::Float(3.14)), "3.14");
    assert_eq!(pg.value_to_sql(&Value::String("test".to_string())), "'test'");
}

#[test]
fn test_pool_stats_utilization() {
    let config = DatabaseConfig::new("sqlite::memory:")
        .max_connections(10);
    let pool = ConnectionPool::new(config);

    // Get 5 connections (50% utilization)
    let _c1 = pool.get().unwrap();
    let _c2 = pool.get().unwrap();
    let _c3 = pool.get().unwrap();
    let _c4 = pool.get().unwrap();
    let _c5 = pool.get().unwrap();

    let stats = pool.stats();
    assert_eq!(stats.utilization(), 100.0); // 5 active / 5 total = 100%
}
