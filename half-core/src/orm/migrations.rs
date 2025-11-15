//! Database migration system for ORM
//!
//! Provides functionality for creating, running, and rolling back database migrations
//! with version tracking.

use crate::orm::connection::{Connection, ConnectionError};
use crate::orm::model::Value;
use crate::orm::schema::{Column, Index, Table};

/// Migration trait that all migrations must implement
pub trait Migration: Send + Sync {
    /// Get migration version/identifier
    fn version(&self) -> &str;

    /// Get migration description
    fn description(&self) -> &str;

    /// Apply the migration (up)
    fn up(&self) -> Vec<String>;

    /// Revert the migration (down)
    fn down(&self) -> Vec<String>;
}

/// Migration version information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationVersion {
    /// Version identifier
    pub version: String,
    /// Migration description
    pub description: String,
    /// Timestamp when migration was applied
    pub applied_at: i64,
    /// Batch number (for grouping migrations)
    pub batch: i32,
}

impl MigrationVersion {
    /// Create a new migration version
    pub fn new(version: impl Into<String>, description: impl Into<String>, batch: i32) -> Self {
        Self {
            version: version.into(),
            description: description.into(),
            applied_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            batch,
        }
    }
}

/// Migration runner for executing migrations
pub struct MigrationRunner {
    migrations: Vec<Box<dyn Migration>>,
    migrations_table: String,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
            migrations_table: "migrations".to_string(),
        }
    }

    /// Set the migrations table name
    pub fn migrations_table(mut self, table: impl Into<String>) -> Self {
        self.migrations_table = table.into();
        self
    }

    /// Add a migration
    pub fn add_migration(&mut self, migration: Box<dyn Migration>) {
        self.migrations.push(migration);
    }

    /// Ensure the migrations table exists
    fn ensure_migrations_table(&self, conn: &mut dyn Connection) -> Result<(), ConnectionError> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTO_INCREMENT,
                version VARCHAR(255) NOT NULL UNIQUE,
                description TEXT,
                applied_at BIGINT NOT NULL,
                batch INTEGER NOT NULL
            )",
            self.migrations_table
        );

        conn.execute(&sql, &[])?;
        Ok(())
    }

    /// Get applied migrations
    fn get_applied_migrations(
        &self,
        conn: &mut dyn Connection,
    ) -> Result<Vec<MigrationVersion>, ConnectionError> {
        let sql = format!(
            "SELECT version, description, applied_at, batch FROM {} ORDER BY applied_at",
            self.migrations_table
        );

        let results = conn.query(&sql, &[])?;

        Ok(results
            .into_iter()
            .map(|row| MigrationVersion {
                version: row
                    .get("version")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string(),
                description: row
                    .get("description")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string(),
                applied_at: row.get("applied_at").and_then(|v| v.as_i64()).unwrap_or(0),
                batch: row.get("batch").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            })
            .collect())
    }

    /// Get the next batch number
    fn next_batch(&self, conn: &mut dyn Connection) -> Result<i32, ConnectionError> {
        let sql = format!(
            "SELECT MAX(batch) as max_batch FROM {}",
            self.migrations_table
        );

        let results = conn.query(&sql, &[])?;

        if let Some(row) = results.into_iter().next() {
            if let Some(max_batch) = row.get("max_batch").and_then(|v| v.as_i64()) {
                return Ok(max_batch as i32 + 1);
            }
        }

        Ok(1)
    }

    /// Run pending migrations
    pub fn run(&self, conn: &mut dyn Connection) -> Result<Vec<String>, ConnectionError> {
        self.ensure_migrations_table(conn)?;

        let applied = self.get_applied_migrations(conn)?;
        let applied_versions: Vec<&str> = applied.iter().map(|m| m.version.as_str()).collect();

        let batch = self.next_batch(conn)?;
        let mut executed = Vec::new();

        for migration in &self.migrations {
            let version = migration.version();

            // Skip if already applied
            if applied_versions.contains(&version) {
                continue;
            }

            // Run migration
            let statements = migration.up();
            for statement in &statements {
                conn.execute(statement, &[])?;
            }

            // Record migration
            let record_sql = format!(
                "INSERT INTO {} (version, description, applied_at, batch) VALUES (?, ?, ?, ?)",
                self.migrations_table
            );

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            conn.execute(
                &record_sql,
                &[
                    Value::String(version.to_string()),
                    Value::String(migration.description().to_string()),
                    Value::Integer(now),
                    Value::Integer(batch as i64),
                ],
            )?;

            executed.push(version.to_string());
        }

        Ok(executed)
    }

    /// Rollback the last batch of migrations
    pub fn rollback(&self, conn: &mut dyn Connection) -> Result<Vec<String>, ConnectionError> {
        self.ensure_migrations_table(conn)?;

        let applied = self.get_applied_migrations(conn)?;

        if applied.is_empty() {
            return Ok(Vec::new());
        }

        // Get the last batch
        let last_batch = applied.iter().map(|m| m.batch).max().unwrap_or(0);

        let to_rollback: Vec<&MigrationVersion> =
            applied.iter().filter(|m| m.batch == last_batch).collect();

        let mut rolled_back = Vec::new();

        // Rollback in reverse order
        for migration_version in to_rollback.iter().rev() {
            // Find the migration
            if let Some(migration) = self
                .migrations
                .iter()
                .find(|m| m.version() == migration_version.version)
            {
                // Run down migration
                let statements = migration.down();
                for statement in &statements {
                    conn.execute(statement, &[])?;
                }

                // Remove migration record
                let delete_sql = format!("DELETE FROM {} WHERE version = ?", self.migrations_table);
                conn.execute(
                    &delete_sql,
                    &[Value::String(migration.version().to_string())],
                )?;

                rolled_back.push(migration.version().to_string());
            }
        }

        Ok(rolled_back)
    }

    /// Get migration status
    pub fn status(&self, conn: &mut dyn Connection) -> Result<MigrationStatus, ConnectionError> {
        self.ensure_migrations_table(conn)?;

        let applied = self.get_applied_migrations(conn)?;
        let applied_versions: Vec<&str> = applied.iter().map(|m| m.version.as_str()).collect();

        let pending: Vec<String> = self
            .migrations
            .iter()
            .filter(|m| !applied_versions.contains(&m.version()))
            .map(|m| m.version().to_string())
            .collect();

        Ok(MigrationStatus {
            applied: applied.len(),
            pending: pending.len(),
            applied_migrations: applied,
            pending_migrations: pending,
        })
    }

    /// Reset all migrations (rollback all)
    pub fn reset(&self, conn: &mut dyn Connection) -> Result<Vec<String>, ConnectionError> {
        let mut all_rolled_back = Vec::new();

        loop {
            let rolled_back = self.rollback(conn)?;
            if rolled_back.is_empty() {
                break;
            }
            all_rolled_back.extend(rolled_back);
        }

        Ok(all_rolled_back)
    }

    /// Get total number of migrations
    pub fn total(&self) -> usize {
        self.migrations.len()
    }
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration status information
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    /// Number of applied migrations
    pub applied: usize,
    /// Number of pending migrations
    pub pending: usize,
    /// List of applied migrations
    pub applied_migrations: Vec<MigrationVersion>,
    /// List of pending migration versions
    pub pending_migrations: Vec<String>,
}

impl MigrationStatus {
    /// Check if all migrations are applied
    pub fn is_up_to_date(&self) -> bool {
        self.pending == 0
    }

    /// Get total number of migrations
    pub fn total(&self) -> usize {
        self.applied + self.pending
    }
}

/// Builder for creating migrations programmatically
pub struct MigrationBuilder {
    version: String,
    description: String,
    up_statements: Vec<String>,
    down_statements: Vec<String>,
}

impl MigrationBuilder {
    /// Create a new migration builder
    pub fn new(version: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            description: description.into(),
            up_statements: Vec::new(),
            down_statements: Vec::new(),
        }
    }

    /// Add a CREATE TABLE statement to up migration
    pub fn create_table(mut self, table: &Table) -> Self {
        self.up_statements.push(table.to_sql());
        self.down_statements
            .push(format!("DROP TABLE {}", table.name));
        self
    }

    /// Add a DROP TABLE statement to up migration
    pub fn drop_table(mut self, table_name: &str) -> Self {
        self.up_statements
            .push(format!("DROP TABLE {}", table_name));
        // Down migration would need the full table schema, which we don't have here
        self
    }

    /// Add a custom SQL statement to up migration
    pub fn up_sql(mut self, sql: impl Into<String>) -> Self {
        self.up_statements.push(sql.into());
        self
    }

    /// Add a custom SQL statement to down migration
    pub fn down_sql(mut self, sql: impl Into<String>) -> Self {
        self.down_statements.push(sql.into());
        self
    }

    /// Add an ADD COLUMN statement
    pub fn add_column(mut self, table_name: &str, column: &Column) -> Self {
        self.up_statements.push(format!(
            "ALTER TABLE {} ADD COLUMN {}",
            table_name,
            column.to_sql()
        ));
        self.down_statements.push(format!(
            "ALTER TABLE {} DROP COLUMN {}",
            table_name, column.name
        ));
        self
    }

    /// Add a DROP COLUMN statement
    pub fn drop_column(mut self, table_name: &str, column_name: &str) -> Self {
        self.up_statements.push(format!(
            "ALTER TABLE {} DROP COLUMN {}",
            table_name, column_name
        ));
        self
    }

    /// Add an index
    pub fn add_index(mut self, table_name: &str, index: &Index) -> Self {
        self.up_statements.push(index.to_sql(table_name));
        self.down_statements
            .push(format!("DROP INDEX {}", index.name));
        self
    }

    /// Build the migration
    pub fn build(self) -> SimpleMigration {
        SimpleMigration {
            version: self.version,
            description: self.description,
            up_statements: self.up_statements,
            down_statements: self.down_statements,
        }
    }
}

/// Simple migration implementation
pub struct SimpleMigration {
    version: String,
    description: String,
    up_statements: Vec<String>,
    down_statements: Vec<String>,
}

impl Migration for SimpleMigration {
    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn up(&self) -> Vec<String> {
        self.up_statements.clone()
    }

    fn down(&self) -> Vec<String> {
        self.down_statements.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orm::schema::ColumnType;

    #[test]
    fn test_migration_version() {
        let version = MigrationVersion::new("001", "Create users table", 1);
        assert_eq!(version.version, "001");
        assert_eq!(version.description, "Create users table");
        assert_eq!(version.batch, 1);
    }

    #[test]
    fn test_migration_builder() {
        let table = Table::new("users")
            .add_column(
                Column::new("id", ColumnType::Integer)
                    .primary_key()
                    .auto_increment(),
            )
            .add_column(Column::new("name", ColumnType::String(255)));

        let migration = MigrationBuilder::new("001", "Create users table")
            .create_table(&table)
            .build();

        assert_eq!(migration.version(), "001");
        assert_eq!(migration.description(), "Create users table");

        let up = migration.up();
        assert_eq!(up.len(), 1);
        assert!(up[0].contains("CREATE TABLE users"));

        let down = migration.down();
        assert_eq!(down.len(), 1);
        assert!(down[0].contains("DROP TABLE users"));
    }

    #[test]
    fn test_migration_runner_creation() {
        let runner = MigrationRunner::new();
        assert_eq!(runner.total(), 0);
    }

    #[test]
    fn test_migration_status() {
        let status = MigrationStatus {
            applied: 5,
            pending: 3,
            applied_migrations: Vec::new(),
            pending_migrations: Vec::new(),
        };

        assert_eq!(status.total(), 8);
        assert!(!status.is_up_to_date());

        let status2 = MigrationStatus {
            applied: 5,
            pending: 0,
            applied_migrations: Vec::new(),
            pending_migrations: Vec::new(),
        };

        assert!(status2.is_up_to_date());
    }

    #[test]
    fn test_simple_migration() {
        let migration = SimpleMigration {
            version: "001".to_string(),
            description: "Test migration".to_string(),
            up_statements: vec!["CREATE TABLE test (id INT)".to_string()],
            down_statements: vec!["DROP TABLE test".to_string()],
        };

        assert_eq!(migration.version(), "001");
        assert_eq!(migration.description(), "Test migration");
        assert_eq!(migration.up().len(), 1);
        assert_eq!(migration.down().len(), 1);
    }

    #[test]
    fn test_add_column_migration() {
        let column = Column::new("email", ColumnType::String(255)).not_null();

        let migration = MigrationBuilder::new("002", "Add email column")
            .add_column("users", &column)
            .build();

        let up = migration.up();
        assert!(up[0].contains("ALTER TABLE users ADD COLUMN email"));

        let down = migration.down();
        assert!(down[0].contains("ALTER TABLE users DROP COLUMN email"));
    }
}
