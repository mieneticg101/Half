//! SQLite database driver
//!
//! Provides SQLite-specific connection and query handling.

use super::{ConnectionInfo, DatabaseDriver, DatabaseType, SqlDialect, DialectType};
use crate::orm::connection::{Connection, ConnectionError};

/// SQLite database driver
#[derive(Debug, Clone)]
pub struct SqliteDriver {
    dialect: SqlDialect,
}

impl SqliteDriver {
    /// Create a new SQLite driver
    pub fn new() -> Self {
        Self {
            dialect: SqlDialect::new(DialectType::SQLite),
        }
    }

    /// Parse SQLite connection URL
    /// Format: sqlite://path/to/file.db or file:path/to/file.db
    fn parse_sqlite_url(&self, url: &str) -> Result<ConnectionInfo, String> {
        let path = url.strip_prefix("sqlite://")
            .or_else(|| url.strip_prefix("file:"))
            .ok_or("Invalid SQLite URL")?;

        let mut info = ConnectionInfo::new(DatabaseType::SQLite, "", path);

        // Parse query parameters if present
        if let Some(idx) = path.find('?') {
            let (file_path, params) = path.split_at(idx);
            info.database = file_path.to_string();

            // Parse parameters
            for param in params[1..].split('&') {
                if let Some(idx) = param.find('=') {
                    let (key, value) = param.split_at(idx);
                    info.options.insert(key.to_string(), value[1..].to_string());
                }
            }
        } else {
            info.database = path.to_string();
        }

        Ok(info)
    }

    /// Get special SQLite URLs
    pub fn in_memory_url() -> &'static str {
        "sqlite::memory:"
    }

    pub fn temporary_url() -> &'static str {
        "sqlite:"
    }
}

impl Default for SqliteDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseDriver for SqliteDriver {
    fn database_type(&self) -> DatabaseType {
        DatabaseType::SQLite
    }

    fn connect(&self, _url: &str) -> Result<Box<dyn Connection>, ConnectionError> {
        // In a real implementation, this would establish an actual SQLite connection
        // using a library like rusqlite or sqlx
        Err(ConnectionError::ConnectionFailed(
            "SQLite driver not yet fully implemented. Use sqlx or rusqlite.".to_string()
        ))
    }

    fn parse_url(&self, url: &str) -> Result<ConnectionInfo, String> {
        self.parse_sqlite_url(url)
    }

    fn dialect(&self) -> &SqlDialect {
        &self.dialect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_url() {
        let driver = SqliteDriver::new();
        let info = driver.parse_url("sqlite://./mydb.db").unwrap();

        assert_eq!(info.database, "./mydb.db");
        assert_eq!(info.db_type, DatabaseType::SQLite);
    }

    #[test]
    fn test_parse_absolute_path() {
        let driver = SqliteDriver::new();
        let info = driver.parse_url("sqlite:///var/lib/mydb.db").unwrap();

        assert_eq!(info.database, "/var/lib/mydb.db");
    }

    #[test]
    fn test_parse_with_params() {
        let driver = SqliteDriver::new();
        let info = driver
            .parse_url("sqlite://./mydb.db?mode=ro&cache=shared")
            .unwrap();

        assert_eq!(info.database, "./mydb.db");
        assert_eq!(info.options.get("mode"), Some(&"ro".to_string()));
        assert_eq!(info.options.get("cache"), Some(&"shared".to_string()));
    }

    #[test]
    fn test_in_memory_url() {
        assert_eq!(SqliteDriver::in_memory_url(), "sqlite::memory:");
    }

    #[test]
    fn test_dialect() {
        let driver = SqliteDriver::new();
        assert_eq!(driver.dialect().placeholder(1), "?");
        assert_eq!(driver.dialect().auto_increment(), "AUTOINCREMENT");
        assert_eq!(driver.dialect().boolean_type(), "INTEGER");
        assert!(driver.dialect().supports_returning());
    }
}
