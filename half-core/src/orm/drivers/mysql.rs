//! MySQL/MariaDB database driver
//!
//! Provides MySQL-specific connection and query handling.

use super::{ConnectionInfo, DatabaseDriver, DatabaseType, DialectType, SqlDialect};
use crate::orm::connection::{Connection, ConnectionError};

/// MySQL database driver
#[derive(Debug, Clone)]
pub struct MySqlDriver {
    dialect: SqlDialect,
}

impl MySqlDriver {
    /// Create a new MySQL driver
    pub fn new() -> Self {
        Self {
            dialect: SqlDialect::new(DialectType::MySQL),
        }
    }

    /// Parse MySQL connection URL
    /// Format: mysql://[user[:password]@][host][:port][/database][?param1=value1&...]
    fn parse_mysql_url(&self, url: &str) -> Result<ConnectionInfo, String> {
        let url = url.strip_prefix("mysql://").ok_or("Invalid MySQL URL")?;

        let mut info = ConnectionInfo::new(DatabaseType::MySQL, "localhost", "mysql");

        // Parse authentication and host/port/database
        let (auth_part, rest) = if let Some(idx) = url.find('@') {
            let (auth, rest) = url.split_at(idx);
            (Some(auth), &rest[1..])
        } else {
            (None, url)
        };

        // Parse username and password
        if let Some(auth) = auth_part {
            let parts: Vec<&str> = auth.split(':').collect();
            if !parts.is_empty() {
                info.username = Some(parts[0].to_string());
                if parts.len() > 1 {
                    info.password = Some(parts[1].to_string());
                }
            }
        }

        // Parse host, port, and database
        let (host_port, db_and_params) = if let Some(idx) = rest.find('/') {
            let (hp, rest) = rest.split_at(idx);
            (hp, &rest[1..])
        } else {
            (rest, "")
        };

        // Parse host and port
        if !host_port.is_empty() {
            let parts: Vec<&str> = host_port.split(':').collect();
            info.host = parts[0].to_string();
            if parts.len() > 1 {
                info.port = parts[1].parse().map_err(|_| "Invalid port")?;
            }
        }

        // Parse database and parameters
        if !db_and_params.is_empty() {
            let (db, params) = if let Some(idx) = db_and_params.find('?') {
                let (d, p) = db_and_params.split_at(idx);
                (d, &p[1..])
            } else {
                (db_and_params, "")
            };

            if !db.is_empty() {
                info.database = db.to_string();
            }

            // Parse query parameters
            if !params.is_empty() {
                for param in params.split('&') {
                    if let Some(idx) = param.find('=') {
                        let (key, value) = param.split_at(idx);
                        info.options.insert(key.to_string(), value[1..].to_string());
                    }
                }
            }
        }

        Ok(info)
    }
}

impl Default for MySqlDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseDriver for MySqlDriver {
    fn database_type(&self) -> DatabaseType {
        DatabaseType::MySQL
    }

    fn connect(&self, _url: &str) -> Result<Box<dyn Connection>, ConnectionError> {
        // In a real implementation, this would establish an actual MySQL connection
        // using a library like mysql_async or sqlx
        Err(ConnectionError::ConnectionFailed(
            "MySQL driver not yet fully implemented. Use sqlx or mysql_async.".to_string(),
        ))
    }

    fn parse_url(&self, url: &str) -> Result<ConnectionInfo, String> {
        self.parse_mysql_url(url)
    }

    fn dialect(&self) -> &SqlDialect {
        &self.dialect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let driver = MySqlDriver::new();
        let info = driver.parse_url("mysql://localhost/mydb").unwrap();

        assert_eq!(info.host, "localhost");
        assert_eq!(info.database, "mydb");
        assert_eq!(info.port, 3306);
    }

    #[test]
    fn test_parse_full_url() {
        let driver = MySqlDriver::new();
        let info = driver
            .parse_url("mysql://user:pass@localhost:3307/mydb?charset=utf8mb4")
            .unwrap();

        assert_eq!(info.username, Some("user".to_string()));
        assert_eq!(info.password, Some("pass".to_string()));
        assert_eq!(info.host, "localhost");
        assert_eq!(info.port, 3307);
        assert_eq!(info.database, "mydb");
        assert_eq!(info.options.get("charset"), Some(&"utf8mb4".to_string()));
    }

    #[test]
    fn test_dialect() {
        let driver = MySqlDriver::new();
        assert_eq!(driver.dialect().placeholder(1), "?");
        assert_eq!(driver.dialect().auto_increment(), "AUTO_INCREMENT");
        assert_eq!(driver.dialect().uuid_type(), "CHAR(36)");
        assert_eq!(driver.dialect().quote_identifier("column"), "`column`");
    }
}
