//! Database schema definition for ORM
//!
//! Provides types for defining database schemas including tables, columns,
//! constraints, and indexes.

use std::collections::HashMap;

/// Database column types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    /// Integer type (i32)
    Integer,
    /// Big integer type (i64)
    BigInteger,
    /// Small integer type (i16)
    SmallInteger,
    /// String type with max length
    String(usize),
    /// Text type (unlimited length)
    Text,
    /// Boolean type
    Boolean,
    /// Float type (f32)
    Float,
    /// Double type (f64)
    Double,
    /// Decimal type with precision and scale
    Decimal(u8, u8),
    /// Date type
    Date,
    /// Time type
    Time,
    /// DateTime type
    DateTime,
    /// Timestamp type
    Timestamp,
    /// Binary data
    Binary,
    /// JSON type
    Json,
    /// UUID type
    Uuid,
}

impl ColumnType {
    /// Convert to SQL type string
    pub fn to_sql(&self) -> String {
        match self {
            ColumnType::Integer => "INTEGER".to_string(),
            ColumnType::BigInteger => "BIGINT".to_string(),
            ColumnType::SmallInteger => "SMALLINT".to_string(),
            ColumnType::String(len) => format!("VARCHAR({})", len),
            ColumnType::Text => "TEXT".to_string(),
            ColumnType::Boolean => "BOOLEAN".to_string(),
            ColumnType::Float => "REAL".to_string(),
            ColumnType::Double => "DOUBLE PRECISION".to_string(),
            ColumnType::Decimal(precision, scale) => format!("DECIMAL({}, {})", precision, scale),
            ColumnType::Date => "DATE".to_string(),
            ColumnType::Time => "TIME".to_string(),
            ColumnType::DateTime => "DATETIME".to_string(),
            ColumnType::Timestamp => "TIMESTAMP".to_string(),
            ColumnType::Binary => "BLOB".to_string(),
            ColumnType::Json => "JSON".to_string(),
            ColumnType::Uuid => "UUID".to_string(),
        }
    }
}

/// Column constraint types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    /// Primary key constraint
    PrimaryKey,
    /// Foreign key constraint
    ForeignKey {
        /// Referenced table
        table: String,
        /// Referenced column
        column: String,
        /// On delete action
        on_delete: Option<ForeignKeyAction>,
        /// On update action
        on_update: Option<ForeignKeyAction>,
    },
    /// Unique constraint
    Unique,
    /// Not null constraint
    NotNull,
    /// Default value constraint
    Default(String),
    /// Check constraint
    Check(String),
    /// Auto increment
    AutoIncrement,
}

/// Foreign key actions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForeignKeyAction {
    /// CASCADE - delete/update related rows
    Cascade,
    /// SET NULL - set foreign key to NULL
    SetNull,
    /// SET DEFAULT - set foreign key to default value
    SetDefault,
    /// RESTRICT - prevent delete/update
    Restrict,
    /// NO ACTION - similar to RESTRICT
    NoAction,
}

impl ForeignKeyAction {
    /// Convert to SQL string
    pub fn to_sql(&self) -> &str {
        match self {
            ForeignKeyAction::Cascade => "CASCADE",
            ForeignKeyAction::SetNull => "SET NULL",
            ForeignKeyAction::SetDefault => "SET DEFAULT",
            ForeignKeyAction::Restrict => "RESTRICT",
            ForeignKeyAction::NoAction => "NO ACTION",
        }
    }
}

/// Database column definition
#[derive(Debug, Clone)]
pub struct Column {
    /// Column name
    pub name: String,
    /// Column type
    pub column_type: ColumnType,
    /// Column constraints
    pub constraints: Vec<Constraint>,
    /// Column comment/description
    pub comment: Option<String>,
}

impl Column {
    /// Create a new column
    pub fn new(name: impl Into<String>, column_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            column_type,
            constraints: Vec::new(),
            comment: None,
        }
    }

    /// Add a constraint to this column
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Mark column as primary key
    pub fn primary_key(self) -> Self {
        self.with_constraint(Constraint::PrimaryKey)
    }

    /// Mark column as not null
    pub fn not_null(self) -> Self {
        self.with_constraint(Constraint::NotNull)
    }

    /// Mark column as unique
    pub fn unique(self) -> Self {
        self.with_constraint(Constraint::Unique)
    }

    /// Mark column as auto increment
    pub fn auto_increment(self) -> Self {
        self.with_constraint(Constraint::AutoIncrement)
    }

    /// Set default value
    pub fn default(self, value: impl Into<String>) -> Self {
        self.with_constraint(Constraint::Default(value.into()))
    }

    /// Add foreign key constraint
    pub fn foreign_key(
        self,
        table: impl Into<String>,
        column: impl Into<String>,
        on_delete: Option<ForeignKeyAction>,
        on_update: Option<ForeignKeyAction>,
    ) -> Self {
        self.with_constraint(Constraint::ForeignKey {
            table: table.into(),
            column: column.into(),
            on_delete,
            on_update,
        })
    }

    /// Add a comment
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Generate SQL for this column
    pub fn to_sql(&self) -> String {
        let mut sql = format!("{} {}", self.name, self.column_type.to_sql());

        for constraint in &self.constraints {
            match constraint {
                Constraint::PrimaryKey => sql.push_str(" PRIMARY KEY"),
                Constraint::NotNull => sql.push_str(" NOT NULL"),
                Constraint::Unique => sql.push_str(" UNIQUE"),
                Constraint::AutoIncrement => sql.push_str(" AUTO_INCREMENT"),
                Constraint::Default(value) => sql.push_str(&format!(" DEFAULT {}", value)),
                Constraint::Check(condition) => sql.push_str(&format!(" CHECK ({})", condition)),
                Constraint::ForeignKey {
                    table,
                    column,
                    on_delete,
                    on_update,
                } => {
                    sql.push_str(&format!(" REFERENCES {}({})", table, column));
                    if let Some(action) = on_delete {
                        sql.push_str(&format!(" ON DELETE {}", action.to_sql()));
                    }
                    if let Some(action) = on_update {
                        sql.push_str(&format!(" ON UPDATE {}", action.to_sql()));
                    }
                }
            }
        }

        sql
    }

    /// Check if column has a specific constraint
    pub fn has_constraint(&self, constraint_type: &Constraint) -> bool {
        self.constraints.iter().any(|c| {
            std::mem::discriminant(c) == std::mem::discriminant(constraint_type)
        })
    }
}

/// Index types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexType {
    /// Regular index
    Index,
    /// Unique index
    Unique,
    /// Full-text index
    FullText,
    /// Spatial index
    Spatial,
}

/// Database index definition
#[derive(Debug, Clone)]
pub struct Index {
    /// Index name
    pub name: String,
    /// Columns in this index
    pub columns: Vec<String>,
    /// Index type
    pub index_type: IndexType,
}

impl Index {
    /// Create a new index
    pub fn new(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            columns,
            index_type: IndexType::Index,
        }
    }

    /// Create a unique index
    pub fn unique(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            columns,
            index_type: IndexType::Unique,
        }
    }

    /// Generate SQL for creating this index
    pub fn to_sql(&self, table: &str) -> String {
        let index_type_str = match self.index_type {
            IndexType::Index => "",
            IndexType::Unique => "UNIQUE ",
            IndexType::FullText => "FULLTEXT ",
            IndexType::Spatial => "SPATIAL ",
        };

        format!(
            "CREATE {}INDEX {} ON {} ({})",
            index_type_str,
            self.name,
            table,
            self.columns.join(", ")
        )
    }
}

/// Database table definition
#[derive(Debug, Clone)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Table columns
    pub columns: HashMap<String, Column>,
    /// Table indexes
    pub indexes: Vec<Index>,
    /// Table comment/description
    pub comment: Option<String>,
}

impl Table {
    /// Create a new table
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: HashMap::new(),
            indexes: Vec::new(),
            comment: None,
        }
    }

    /// Add a column to the table
    pub fn add_column(mut self, column: Column) -> Self {
        self.columns.insert(column.name.clone(), column);
        self
    }

    /// Add an index to the table
    pub fn add_index(mut self, index: Index) -> Self {
        self.indexes.push(index);
        self
    }

    /// Add a comment to the table
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Generate SQL for creating this table
    pub fn to_sql(&self) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.name);

        let column_defs: Vec<String> = self.columns.values().map(|col| col.to_sql()).collect();
        sql.push_str(&format!("  {}\n", column_defs.join(",\n  ")));

        sql.push(')');

        if let Some(comment) = &self.comment {
            sql.push_str(&format!(" COMMENT '{}'", comment));
        }

        sql
    }

    /// Get a column by name
    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.columns.get(name)
    }

    /// Check if table has a column
    pub fn has_column(&self, name: &str) -> bool {
        self.columns.contains_key(name)
    }

    /// Get all column names
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.keys().map(|s| s.as_str()).collect()
    }
}

/// Schema builder for creating database schemas
pub struct Schema {
    tables: HashMap<String, Table>,
}

impl Schema {
    /// Create a new schema
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Create a new table
    pub fn create_table(
        &mut self,
        name: impl Into<String>,
        builder: impl FnOnce(&mut TableBuilder),
    ) -> &mut Self {
        let mut table_builder = TableBuilder::new(name.into());
        builder(&mut table_builder);
        let table = table_builder.build();
        self.tables.insert(table.name.clone(), table);
        self
    }

    /// Drop a table
    pub fn drop_table(&mut self, name: &str) -> &mut Self {
        self.tables.remove(name);
        self
    }

    /// Get a table by name
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    /// Get all table names
    pub fn table_names(&self) -> Vec<&str> {
        self.tables.keys().map(|s| s.as_str()).collect()
    }

    /// Generate SQL for the entire schema
    pub fn to_sql(&self) -> Vec<String> {
        let mut statements = Vec::new();

        // Create tables
        for table in self.tables.values() {
            statements.push(table.to_sql());
        }

        // Create indexes
        for table in self.tables.values() {
            for index in &table.indexes {
                statements.push(index.to_sql(&table.name));
            }
        }

        statements
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

/// Table builder for fluent table creation
pub struct TableBuilder {
    table: Table,
}

impl TableBuilder {
    /// Create a new table builder
    pub fn new(name: String) -> Self {
        Self {
            table: Table::new(name),
        }
    }

    /// Add an integer column
    pub fn integer(&mut self, name: &str) -> &mut Self {
        self.add_column(Column::new(name, ColumnType::Integer));
        self
    }

    /// Add a big integer column
    pub fn big_integer(&mut self, name: &str) -> &mut Self {
        self.add_column(Column::new(name, ColumnType::BigInteger));
        self
    }

    /// Add a string column
    pub fn string(&mut self, name: &str, length: usize) -> &mut Self {
        self.add_column(Column::new(name, ColumnType::String(length)));
        self
    }

    /// Add a text column
    pub fn text(&mut self, name: &str) -> &mut Self {
        self.add_column(Column::new(name, ColumnType::Text));
        self
    }

    /// Add a boolean column
    pub fn boolean(&mut self, name: &str) -> &mut Self {
        self.add_column(Column::new(name, ColumnType::Boolean));
        self
    }

    /// Add a datetime column
    pub fn datetime(&mut self, name: &str) -> &mut Self {
        self.add_column(Column::new(name, ColumnType::DateTime));
        self
    }

    /// Add a timestamp column
    pub fn timestamp(&mut self, name: &str) -> &mut Self {
        self.add_column(Column::new(name, ColumnType::Timestamp));
        self
    }

    /// Add a JSON column
    pub fn json(&mut self, name: &str) -> &mut Self {
        self.add_column(Column::new(name, ColumnType::Json));
        self
    }

    /// Add a UUID column
    pub fn uuid(&mut self, name: &str) -> &mut Self {
        self.add_column(Column::new(name, ColumnType::Uuid));
        self
    }

    /// Add a column
    fn add_column(&mut self, column: Column) {
        let name = column.name.clone();
        self.table.columns.insert(name, column);
    }

    /// Mark the last added column as primary key
    pub fn primary_key(&mut self) -> &mut Self {
        if let Some((_, column)) = self.table.columns.iter_mut().last() {
            column.constraints.push(Constraint::PrimaryKey);
        }
        self
    }

    /// Mark the last added column as not null
    pub fn not_null(&mut self) -> &mut Self {
        if let Some((_, column)) = self.table.columns.iter_mut().last() {
            column.constraints.push(Constraint::NotNull);
        }
        self
    }

    /// Mark the last added column as unique
    pub fn unique(&mut self) -> &mut Self {
        if let Some((_, column)) = self.table.columns.iter_mut().last() {
            column.constraints.push(Constraint::Unique);
        }
        self
    }

    /// Mark the last added column as auto increment
    pub fn auto_increment(&mut self) -> &mut Self {
        if let Some((_, column)) = self.table.columns.iter_mut().last() {
            column.constraints.push(Constraint::AutoIncrement);
        }
        self
    }

    /// Set default value for the last added column
    pub fn default(&mut self, value: &str) -> &mut Self {
        if let Some((_, column)) = self.table.columns.iter_mut().last() {
            column.constraints.push(Constraint::Default(value.to_string()));
        }
        self
    }

    /// Add an index
    pub fn index(&mut self, name: &str, columns: Vec<&str>) -> &mut Self {
        let index = Index::new(
            name,
            columns.iter().map(|s| s.to_string()).collect(),
        );
        self.table.indexes.push(index);
        self
    }

    /// Add a unique index
    pub fn unique_index(&mut self, name: &str, columns: Vec<&str>) -> &mut Self {
        let index = Index::unique(
            name,
            columns.iter().map(|s| s.to_string()).collect(),
        );
        self.table.indexes.push(index);
        self
    }

    /// Build the table
    pub fn build(self) -> Table {
        self.table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_type_to_sql() {
        assert_eq!(ColumnType::Integer.to_sql(), "INTEGER");
        assert_eq!(ColumnType::String(255).to_sql(), "VARCHAR(255)");
        assert_eq!(ColumnType::Text.to_sql(), "TEXT");
        assert_eq!(ColumnType::Boolean.to_sql(), "BOOLEAN");
        assert_eq!(ColumnType::DateTime.to_sql(), "DATETIME");
    }

    #[test]
    fn test_column_creation() {
        let column = Column::new("id", ColumnType::Integer)
            .primary_key()
            .auto_increment()
            .not_null();

        assert_eq!(column.name, "id");
        assert_eq!(column.column_type, ColumnType::Integer);
        assert_eq!(column.constraints.len(), 3);
    }

    #[test]
    fn test_column_to_sql() {
        let column = Column::new("name", ColumnType::String(255))
            .not_null()
            .default("''");

        let sql = column.to_sql();
        assert!(sql.contains("name VARCHAR(255)"));
        assert!(sql.contains("NOT NULL"));
        assert!(sql.contains("DEFAULT ''"));
    }

    #[test]
    fn test_foreign_key() {
        let column = Column::new("user_id", ColumnType::Integer)
            .foreign_key("users", "id", Some(ForeignKeyAction::Cascade), None);

        let sql = column.to_sql();
        assert!(sql.contains("REFERENCES users(id)"));
        assert!(sql.contains("ON DELETE CASCADE"));
    }

    #[test]
    fn test_table_creation() {
        let table = Table::new("users")
            .add_column(
                Column::new("id", ColumnType::Integer)
                    .primary_key()
                    .auto_increment(),
            )
            .add_column(Column::new("name", ColumnType::String(255)).not_null())
            .add_column(Column::new("email", ColumnType::String(255)).unique());

        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 3);
        assert!(table.has_column("id"));
        assert!(table.has_column("name"));
        assert!(table.has_column("email"));
    }

    #[test]
    fn test_index_creation() {
        let index = Index::new("idx_email", vec!["email".to_string()]);
        let sql = index.to_sql("users");
        assert_eq!(sql, "CREATE INDEX idx_email ON users (email)");

        let unique_index = Index::unique("idx_email_unique", vec!["email".to_string()]);
        let sql = unique_index.to_sql("users");
        assert_eq!(sql, "CREATE UNIQUE INDEX idx_email_unique ON users (email)");
    }

    #[test]
    fn test_schema_builder() {
        let mut schema = Schema::new();
        schema.create_table("users", |t| {
            t.integer("id").primary_key().auto_increment();
            t.string("name", 255).not_null();
            t.string("email", 255).unique().not_null();
            t.timestamp("created_at").not_null();
        });

        assert!(schema.get_table("users").is_some());
        let table = schema.get_table("users").unwrap();
        assert_eq!(table.columns.len(), 4);
    }

    #[test]
    fn test_table_to_sql() {
        let table = Table::new("users")
            .add_column(
                Column::new("id", ColumnType::Integer)
                    .primary_key()
                    .auto_increment(),
            )
            .add_column(Column::new("name", ColumnType::String(255)).not_null());

        let sql = table.to_sql();
        assert!(sql.contains("CREATE TABLE users"));
        assert!(sql.contains("id INTEGER PRIMARY KEY AUTO_INCREMENT"));
        assert!(sql.contains("name VARCHAR(255) NOT NULL"));
    }
}
