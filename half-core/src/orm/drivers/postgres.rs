//! PostgreSQL database driver
//!
//! Provides PostgreSQL-specific connection and query handling.

use super::{ConnectionInfo, DatabaseDriver, DatabaseType, DialectType, SqlDialect};
use crate::orm::connection::{Connection, ConnectionError};

/// PostgreSQL database driver
#[derive(Debug, Clone)]
pub struct PostgresDriver {
    dialect: SqlDialect,
}

impl PostgresDriver {
    /// Create a new PostgreSQL driver
    pub fn new() -> Self {
        Self {
            dialect: SqlDialect::new(DialectType::PostgreSQL),
        }
    }

    /// Parse PostgreSQL connection URL
    /// Format: postgresql://[user[:password]@][host][:port][/database][?param1=value1&...]
    fn parse_postgres_url(&self, url: &str) -> Result<ConnectionInfo, String> {
        let url = url
            .strip_prefix("postgresql://")
            .or_else(|| url.strip_prefix("postgres://"))
            .ok_or("Invalid PostgreSQL URL")?;

        let mut info = ConnectionInfo::new(DatabaseType::PostgreSQL, "localhost", "postgres");

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

impl Default for PostgresDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseDriver for PostgresDriver {
    fn database_type(&self) -> DatabaseType {
        DatabaseType::PostgreSQL
    }

    fn connect(&self, _url: &str) -> Result<Box<dyn Connection>, ConnectionError> {
        // In a real implementation, this would establish an actual PostgreSQL connection
        // using a library like tokio-postgres or sqlx
        Err(ConnectionError::ConnectionFailed(
            "PostgreSQL driver not yet fully implemented. Use sqlx or tokio-postgres.".to_string(),
        ))
    }

    fn parse_url(&self, url: &str) -> Result<ConnectionInfo, String> {
        self.parse_postgres_url(url)
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
        let driver = PostgresDriver::new();
        let info = driver.parse_url("postgresql://localhost/mydb").unwrap();

        assert_eq!(info.host, "localhost");
        assert_eq!(info.database, "mydb");
        assert_eq!(info.port, 5432);
    }

    #[test]
    fn test_parse_full_url() {
        let driver = PostgresDriver::new();
        let info = driver
            .parse_url("postgresql://user:pass@localhost:5433/mydb?sslmode=require")
            .unwrap();

        assert_eq!(info.username, Some("user".to_string()));
        assert_eq!(info.password, Some("pass".to_string()));
        assert_eq!(info.host, "localhost");
        assert_eq!(info.port, 5433);
        assert_eq!(info.database, "mydb");
        assert_eq!(info.options.get("sslmode"), Some(&"require".to_string()));
    }

    #[test]
    fn test_parse_with_params() {
        let driver = PostgresDriver::new();
        let info = driver
            .parse_url("postgresql://localhost/mydb?connect_timeout=10&application_name=myapp")
            .unwrap();

        assert_eq!(info.options.get("connect_timeout"), Some(&"10".to_string()));
        assert_eq!(
            info.options.get("application_name"),
            Some(&"myapp".to_string())
        );
    }

    #[test]
    fn test_dialect() {
        let driver = PostgresDriver::new();
        assert_eq!(driver.dialect().placeholder(1), "$1");
        assert_eq!(driver.dialect().placeholder(2), "$2");
        assert_eq!(driver.dialect().auto_increment(), "SERIAL");
        assert_eq!(driver.dialect().uuid_type(), "UUID");
        assert!(driver.dialect().supports_returning());
    }
}
