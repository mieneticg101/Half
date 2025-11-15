//! SQL dialect handling for different database engines
//!
//! Each database has its own SQL syntax variations. This module
//! provides dialect-specific SQL generation.

use crate::orm::model::Value;

/// SQL dialect types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialectType {
    /// PostgreSQL dialect
    PostgreSQL,
    /// MySQL/MariaDB dialect
    MySQL,
    /// SQLite dialect
    SQLite,
    /// Microsoft SQL Server dialect
    SqlServer,
    /// MongoDB query language (not SQL)
    MongoDB,
}

/// SQL dialect for generating database-specific SQL
#[derive(Debug, Clone)]
pub struct SqlDialect {
    dialect_type: DialectType,
}

impl SqlDialect {
    /// Create a new SQL dialect
    pub fn new(dialect_type: DialectType) -> Self {
        Self { dialect_type }
    }

    /// Get the dialect type
    pub fn dialect_type(&self) -> DialectType {
        self.dialect_type
    }

    /// Get the parameter placeholder
    /// PostgreSQL: $1, $2, $3
    /// MySQL: ?, ?, ?
    /// SQLite: ?, ?, ?
    pub fn placeholder(&self, position: usize) -> String {
        match self.dialect_type {
            DialectType::PostgreSQL => format!("${}", position),
            DialectType::MySQL | DialectType::SQLite => "?".to_string(),
            DialectType::SqlServer => format!("@p{}", position),
            DialectType::MongoDB => String::new(), // N/A
        }
    }

    /// Get the quote character for identifiers
    /// PostgreSQL: "column"
    /// MySQL: `column`
    /// SQLite: "column" or `column`
    pub fn quote_identifier(&self, identifier: &str) -> String {
        match self.dialect_type {
            DialectType::PostgreSQL | DialectType::SQLite => format!("\"{}\"", identifier),
            DialectType::MySQL => format!("`{}`", identifier),
            DialectType::SqlServer => format!("[{}]", identifier),
            DialectType::MongoDB => identifier.to_string(),
        }
    }

    /// Get the LIMIT clause syntax
    /// PostgreSQL/MySQL/SQLite: LIMIT n OFFSET m
    /// SQL Server: OFFSET m ROWS FETCH NEXT n ROWS ONLY
    pub fn limit_clause(&self, limit: usize, offset: Option<usize>) -> String {
        match self.dialect_type {
            DialectType::PostgreSQL | DialectType::MySQL | DialectType::SQLite => {
                let mut clause = format!("LIMIT {}", limit);
                if let Some(off) = offset {
                    clause.push_str(&format!(" OFFSET {}", off));
                }
                clause
            }
            DialectType::SqlServer => {
                if let Some(off) = offset {
                    format!("OFFSET {} ROWS FETCH NEXT {} ROWS ONLY", off, limit)
                } else {
                    format!("OFFSET 0 ROWS FETCH NEXT {} ROWS ONLY", limit)
                }
            }
            DialectType::MongoDB => String::new(),
        }
    }

    /// Get the AUTO_INCREMENT syntax
    /// PostgreSQL: SERIAL or GENERATED ALWAYS AS IDENTITY
    /// MySQL: AUTO_INCREMENT
    /// SQLite: AUTOINCREMENT
    pub fn auto_increment(&self) -> &str {
        match self.dialect_type {
            DialectType::PostgreSQL => "SERIAL",
            DialectType::MySQL => "AUTO_INCREMENT",
            DialectType::SQLite => "AUTOINCREMENT",
            DialectType::SqlServer => "IDENTITY(1,1)",
            DialectType::MongoDB => "",
        }
    }

    /// Get the BOOLEAN type
    /// PostgreSQL: BOOLEAN
    /// MySQL: TINYINT(1) or BOOLEAN (alias for TINYINT(1))
    /// SQLite: INTEGER (0 or 1)
    pub fn boolean_type(&self) -> &str {
        match self.dialect_type {
            DialectType::PostgreSQL => "BOOLEAN",
            DialectType::MySQL => "BOOLEAN",
            DialectType::SQLite => "INTEGER",
            DialectType::SqlServer => "BIT",
            DialectType::MongoDB => "boolean",
        }
    }

    /// Get the TEXT type
    pub fn text_type(&self) -> &str {
        match self.dialect_type {
            DialectType::PostgreSQL => "TEXT",
            DialectType::MySQL => "TEXT",
            DialectType::SQLite => "TEXT",
            DialectType::SqlServer => "NVARCHAR(MAX)",
            DialectType::MongoDB => "string",
        }
    }

    /// Get the BLOB/BINARY type
    pub fn blob_type(&self) -> &str {
        match self.dialect_type {
            DialectType::PostgreSQL => "BYTEA",
            DialectType::MySQL => "BLOB",
            DialectType::SQLite => "BLOB",
            DialectType::SqlServer => "VARBINARY(MAX)",
            DialectType::MongoDB => "binData",
        }
    }

    /// Get the JSON type
    /// PostgreSQL: JSON or JSONB
    /// MySQL: JSON (5.7+)
    /// SQLite: TEXT (stores as text)
    pub fn json_type(&self) -> &str {
        match self.dialect_type {
            DialectType::PostgreSQL => "JSONB",
            DialectType::MySQL => "JSON",
            DialectType::SQLite => "TEXT",
            DialectType::SqlServer => "NVARCHAR(MAX)",
            DialectType::MongoDB => "object",
        }
    }

    /// Get the UUID type
    /// PostgreSQL: UUID
    /// MySQL: CHAR(36)
    /// SQLite: TEXT
    pub fn uuid_type(&self) -> &str {
        match self.dialect_type {
            DialectType::PostgreSQL => "UUID",
            DialectType::MySQL => "CHAR(36)",
            DialectType::SQLite => "TEXT",
            DialectType::SqlServer => "UNIQUEIDENTIFIER",
            DialectType::MongoDB => "string",
        }
    }

    /// Get the TIMESTAMP type with timezone support
    /// PostgreSQL: TIMESTAMP WITH TIME ZONE
    /// MySQL: TIMESTAMP
    /// SQLite: TEXT or INTEGER
    pub fn timestamp_type(&self) -> &str {
        match self.dialect_type {
            DialectType::PostgreSQL => "TIMESTAMP WITH TIME ZONE",
            DialectType::MySQL => "TIMESTAMP",
            DialectType::SQLite => "INTEGER",
            DialectType::SqlServer => "DATETIME2",
            DialectType::MongoDB => "date",
        }
    }

    /// Check if RETURNING clause is supported
    /// PostgreSQL: Yes
    /// MySQL: No (before 8.0.21)
    /// SQLite: Yes (3.35.0+)
    pub fn supports_returning(&self) -> bool {
        matches!(
            self.dialect_type,
            DialectType::PostgreSQL | DialectType::SQLite
        )
    }

    /// Get RETURNING clause for INSERT
    pub fn returning_clause(&self, columns: &[&str]) -> Option<String> {
        if self.supports_returning() {
            Some(format!("RETURNING {}", columns.join(", ")))
        } else {
            None
        }
    }

    /// Check if UPSERT (INSERT ... ON CONFLICT) is supported
    pub fn supports_upsert(&self) -> bool {
        matches!(
            self.dialect_type,
            DialectType::PostgreSQL | DialectType::SQLite | DialectType::MySQL
        )
    }

    /// Get UPSERT clause
    /// PostgreSQL: ON CONFLICT ... DO UPDATE
    /// MySQL: ON DUPLICATE KEY UPDATE
    /// SQLite: ON CONFLICT ... DO UPDATE
    pub fn upsert_clause(
        &self,
        conflict_columns: &[&str],
        update_columns: &[&str],
    ) -> Option<String> {
        match self.dialect_type {
            DialectType::PostgreSQL | DialectType::SQLite => {
                let updates = update_columns
                    .iter()
                    .map(|col| format!("{} = EXCLUDED.{}", col, col))
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!(
                    "ON CONFLICT ({}) DO UPDATE SET {}",
                    conflict_columns.join(", "),
                    updates
                ))
            }
            DialectType::MySQL => {
                let updates = update_columns
                    .iter()
                    .map(|col| format!("{} = VALUES({})", col, col))
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!("ON DUPLICATE KEY UPDATE {}", updates))
            }
            _ => None,
        }
    }

    /// Convert Value to SQL literal for this dialect
    pub fn value_to_sql(&self, value: &Value) -> String {
        match value {
            Value::Null => "NULL".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => {
                // Escape single quotes
                let escaped = s.replace('\'', "''");
                format!("'{}'", escaped)
            }
            Value::Boolean(b) => match self.dialect_type {
                DialectType::PostgreSQL => if *b { "TRUE" } else { "FALSE" }.to_string(),
                DialectType::MySQL | DialectType::SQLite => if *b { "1" } else { "0" }.to_string(),
                DialectType::SqlServer => if *b { "1" } else { "0" }.to_string(),
                DialectType::MongoDB => b.to_string(),
            },
            Value::Binary(_) => match self.dialect_type {
                DialectType::PostgreSQL => "'\\x'".to_string(), // Simplified
                _ => "BLOB".to_string(),
            },
            Value::Json(j) => {
                let escaped = j.replace('\'', "''");
                match self.dialect_type {
                    DialectType::PostgreSQL | DialectType::MySQL => format!("'{}'", escaped),
                    _ => format!("'{}'", escaped),
                }
            }
        }
    }

    /// Get the current timestamp function
    /// PostgreSQL: NOW() or CURRENT_TIMESTAMP
    /// MySQL: NOW() or CURRENT_TIMESTAMP
    /// SQLite: CURRENT_TIMESTAMP
    pub fn current_timestamp(&self) -> &str {
        match self.dialect_type {
            DialectType::PostgreSQL | DialectType::MySQL => "NOW()",
            DialectType::SQLite => "CURRENT_TIMESTAMP",
            DialectType::SqlServer => "GETDATE()",
            DialectType::MongoDB => "new Date()",
        }
    }

    /// Check if database supports window functions
    pub fn supports_window_functions(&self) -> bool {
        !matches!(self.dialect_type, DialectType::MongoDB)
    }

    /// Check if database supports CTEs (Common Table Expressions)
    pub fn supports_cte(&self) -> bool {
        !matches!(self.dialect_type, DialectType::MongoDB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let pg = SqlDialect::new(DialectType::PostgreSQL);
        assert_eq!(pg.placeholder(1), "$1");
        assert_eq!(pg.placeholder(2), "$2");

        let mysql = SqlDialect::new(DialectType::MySQL);
        assert_eq!(mysql.placeholder(1), "?");
        assert_eq!(mysql.placeholder(2), "?");
    }

    #[test]
    fn test_quote_identifier() {
        let pg = SqlDialect::new(DialectType::PostgreSQL);
        assert_eq!(pg.quote_identifier("column"), "\"column\"");

        let mysql = SqlDialect::new(DialectType::MySQL);
        assert_eq!(mysql.quote_identifier("column"), "`column`");

        let mssql = SqlDialect::new(DialectType::SqlServer);
        assert_eq!(mssql.quote_identifier("column"), "[column]");
    }

    #[test]
    fn test_limit_clause() {
        let pg = SqlDialect::new(DialectType::PostgreSQL);
        assert_eq!(pg.limit_clause(10, Some(20)), "LIMIT 10 OFFSET 20");
        assert_eq!(pg.limit_clause(10, None), "LIMIT 10");

        let mssql = SqlDialect::new(DialectType::SqlServer);
        assert_eq!(
            mssql.limit_clause(10, Some(20)),
            "OFFSET 20 ROWS FETCH NEXT 10 ROWS ONLY"
        );
    }

    #[test]
    fn test_auto_increment() {
        assert_eq!(
            SqlDialect::new(DialectType::PostgreSQL).auto_increment(),
            "SERIAL"
        );
        assert_eq!(
            SqlDialect::new(DialectType::MySQL).auto_increment(),
            "AUTO_INCREMENT"
        );
        assert_eq!(
            SqlDialect::new(DialectType::SQLite).auto_increment(),
            "AUTOINCREMENT"
        );
    }

    #[test]
    fn test_type_mappings() {
        let pg = SqlDialect::new(DialectType::PostgreSQL);
        assert_eq!(pg.boolean_type(), "BOOLEAN");
        assert_eq!(pg.text_type(), "TEXT");
        assert_eq!(pg.json_type(), "JSONB");
        assert_eq!(pg.uuid_type(), "UUID");
        assert_eq!(pg.blob_type(), "BYTEA");

        let mysql = SqlDialect::new(DialectType::MySQL);
        assert_eq!(mysql.uuid_type(), "CHAR(36)");
        assert_eq!(mysql.blob_type(), "BLOB");
    }

    #[test]
    fn test_supports_returning() {
        assert!(SqlDialect::new(DialectType::PostgreSQL).supports_returning());
        assert!(!SqlDialect::new(DialectType::MySQL).supports_returning());
    }

    #[test]
    fn test_returning_clause() {
        let pg = SqlDialect::new(DialectType::PostgreSQL);
        assert_eq!(
            pg.returning_clause(&["id", "created_at"]),
            Some("RETURNING id, created_at".to_string())
        );

        let mysql = SqlDialect::new(DialectType::MySQL);
        assert_eq!(mysql.returning_clause(&["id"]), None);
    }

    #[test]
    fn test_upsert_clause() {
        let pg = SqlDialect::new(DialectType::PostgreSQL);
        assert_eq!(
            pg.upsert_clause(&["id"], &["name", "email"]),
            Some(
                "ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, email = EXCLUDED.email"
                    .to_string()
            )
        );

        let mysql = SqlDialect::new(DialectType::MySQL);
        assert_eq!(
            mysql.upsert_clause(&["id"], &["name", "email"]),
            Some("ON DUPLICATE KEY UPDATE name = VALUES(name), email = VALUES(email)".to_string())
        );
    }

    #[test]
    fn test_value_to_sql() {
        let pg = SqlDialect::new(DialectType::PostgreSQL);
        assert_eq!(pg.value_to_sql(&Value::Integer(42)), "42");
        assert_eq!(
            pg.value_to_sql(&Value::String("test".to_string())),
            "'test'"
        );
        assert_eq!(pg.value_to_sql(&Value::Boolean(true)), "TRUE");
        assert_eq!(pg.value_to_sql(&Value::Null), "NULL");

        let mysql = SqlDialect::new(DialectType::MySQL);
        assert_eq!(mysql.value_to_sql(&Value::Boolean(true)), "1");
        assert_eq!(mysql.value_to_sql(&Value::Boolean(false)), "0");
    }
}
