//! Database drivers for multiple database engines
//!
//! Provides support for PostgreSQL, MySQL, SQLite, and other databases
//! similar to Prisma's multi-database support.

pub mod dialect;
pub mod mysql;
pub mod postgres;
pub mod sqlite;

pub use dialect::{DialectType, SqlDialect};
pub use mysql::MySqlDriver;
pub use postgres::PostgresDriver;
pub use sqlite::SqliteDriver;

use crate::orm::connection::{Connection, ConnectionError};
use std::collections::HashMap;

/// Database driver type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    /// PostgreSQL database
    PostgreSQL,
    /// MySQL/MariaDB database
    MySQL,
    /// SQLite database
    SQLite,
    /// Microsoft SQL Server
    SqlServer,
    /// MongoDB (NoSQL)
    MongoDB,
}

impl DatabaseType {
    /// Parse database type from connection URL
    pub fn from_url(url: &str) -> Result<Self, String> {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            Ok(DatabaseType::PostgreSQL)
        } else if url.starts_with("mysql://") {
            Ok(DatabaseType::MySQL)
        } else if url.starts_with("sqlite://") || url.starts_with("file:") {
            Ok(DatabaseType::SQLite)
        } else if url.starts_with("sqlserver://") || url.starts_with("mssql://") {
            Ok(DatabaseType::SqlServer)
        } else if url.starts_with("mongodb://") || url.starts_with("mongodb+srv://") {
            Ok(DatabaseType::MongoDB)
        } else {
            Err(format!("Unknown database type in URL: {}", url))
        }
    }

    /// Get the SQL dialect for this database type
    pub fn dialect(&self) -> SqlDialect {
        match self {
            DatabaseType::PostgreSQL => SqlDialect::new(DialectType::PostgreSQL),
            DatabaseType::MySQL => SqlDialect::new(DialectType::MySQL),
            DatabaseType::SQLite => SqlDialect::new(DialectType::SQLite),
            DatabaseType::SqlServer => SqlDialect::new(DialectType::SqlServer),
            DatabaseType::MongoDB => SqlDialect::new(DialectType::MongoDB),
        }
    }

    /// Get the default port for this database type
    pub fn default_port(&self) -> u16 {
        match self {
            DatabaseType::PostgreSQL => 5432,
            DatabaseType::MySQL => 3306,
            DatabaseType::SQLite => 0, // N/A for file-based
            DatabaseType::SqlServer => 1433,
            DatabaseType::MongoDB => 27017,
        }
    }
}

/// Database driver trait that all drivers must implement
pub trait DatabaseDriver: Send + Sync {
    /// Get the database type
    fn database_type(&self) -> DatabaseType;

    /// Connect to the database
    fn connect(&self, url: &str) -> Result<Box<dyn Connection>, ConnectionError>;

    /// Parse connection URL into components
    fn parse_url(&self, url: &str) -> Result<ConnectionInfo, String>;

    /// Get the SQL dialect
    fn dialect(&self) -> &SqlDialect;
}

/// Parsed connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Database type
    pub db_type: DatabaseType,
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username
    pub username: Option<String>,
    /// Password
    pub password: Option<String>,
    /// SSL/TLS mode
    pub ssl_mode: Option<String>,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl ConnectionInfo {
    /// Create a new connection info
    pub fn new(
        db_type: DatabaseType,
        host: impl Into<String>,
        database: impl Into<String>,
    ) -> Self {
        Self {
            db_type,
            host: host.into(),
            port: db_type.default_port(),
            database: database.into(),
            username: None,
            password: None,
            ssl_mode: None,
            options: HashMap::new(),
        }
    }

    /// Set username
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Set password
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set port
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set SSL mode
    pub fn ssl_mode(mut self, mode: impl Into<String>) -> Self {
        self.ssl_mode = Some(mode.into());
        self
    }

    /// Add an option
    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Build connection URL
    pub fn to_url(&self) -> String {
        let scheme = match self.db_type {
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::MySQL => "mysql",
            DatabaseType::SQLite => return format!("sqlite://{}", self.database),
            DatabaseType::SqlServer => "sqlserver",
            DatabaseType::MongoDB => "mongodb",
        };

        let auth = if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            format!("{}:{}@", user, pass)
        } else if let Some(user) = &self.username {
            format!("{}@", user)
        } else {
            String::new()
        };

        let mut url = format!(
            "{}://{}{}:{}/{}",
            scheme, auth, self.host, self.port, self.database
        );

        if !self.options.is_empty() {
            let opts: Vec<String> = self
                .options
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push('?');
            url.push_str(&opts.join("&"));
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_from_url() {
        assert_eq!(
            DatabaseType::from_url("postgresql://localhost/mydb").unwrap(),
            DatabaseType::PostgreSQL
        );
        assert_eq!(
            DatabaseType::from_url("mysql://localhost/mydb").unwrap(),
            DatabaseType::MySQL
        );
        assert_eq!(
            DatabaseType::from_url("sqlite://./mydb.db").unwrap(),
            DatabaseType::SQLite
        );
    }

    #[test]
    fn test_database_type_default_port() {
        assert_eq!(DatabaseType::PostgreSQL.default_port(), 5432);
        assert_eq!(DatabaseType::MySQL.default_port(), 3306);
        assert_eq!(DatabaseType::MongoDB.default_port(), 27017);
    }

    #[test]
    fn test_connection_info_builder() {
        let info = ConnectionInfo::new(DatabaseType::PostgreSQL, "localhost", "mydb")
            .username("user")
            .password("pass")
            .port(5433);

        assert_eq!(info.host, "localhost");
        assert_eq!(info.database, "mydb");
        assert_eq!(info.username, Some("user".to_string()));
        assert_eq!(info.port, 5433);
    }

    #[test]
    fn test_connection_info_to_url() {
        let info = ConnectionInfo::new(DatabaseType::PostgreSQL, "localhost", "mydb")
            .username("user")
            .password("pass");

        let url = info.to_url();
        assert!(url.starts_with("postgresql://user:pass@localhost:5432/mydb"));
    }

    #[test]
    fn test_sqlite_url() {
        let info = ConnectionInfo::new(DatabaseType::SQLite, "", "./test.db");
        let url = info.to_url();
        assert_eq!(url, "sqlite://./test.db");
    }
}
