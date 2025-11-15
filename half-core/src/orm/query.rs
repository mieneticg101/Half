//! Type-safe query interface for ORM
//!
//! Provides a fluent API for building and executing database queries
//! with compile-time type safety.

use crate::database::{QueryBuilder, Order as SqlOrder};
use crate::orm::connection::{Connection, ConnectionError};
use crate::orm::model::{Model, Value};
use std::collections::HashMap;
use std::marker::PhantomData;

/// Query executor for running ORM queries
pub trait QueryExecutor {
    /// Execute a query and return the number of affected rows
    fn execute(&mut self, query: &str, params: &[Value]) -> Result<u64, ConnectionError>;

    /// Execute a query and return results
    fn query(
        &mut self,
        query: &str,
        params: &[Value],
    ) -> Result<Vec<HashMap<String, Value>>, ConnectionError>;
}

impl<T: Connection> QueryExecutor for T {
    fn execute(&mut self, query: &str, params: &[Value]) -> Result<u64, ConnectionError> {
        Connection::execute(self, query, params)
    }

    fn query(
        &mut self,
        query: &str,
        params: &[Value],
    ) -> Result<Vec<HashMap<String, Value>>, ConnectionError> {
        Connection::query(self, query, params)
    }
}

/// Type-safe query builder for ORM
pub struct Query<T: Model> {
    query_builder: QueryBuilder,
    conditions: Vec<Condition>,
    params: Vec<Value>,
    _phantom: PhantomData<T>,
}

impl<T: Model> Query<T> {
    /// Create a new query for a model
    #[inline]
    pub fn new() -> Self {
        Self {
            query_builder: QueryBuilder::new().from(T::table_name()),
            conditions: Vec::new(),
            params: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Select specific columns
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.query_builder = self.query_builder.select(columns);
        self
    }

    /// Select all columns
    pub fn select_all(mut self) -> Self {
        self.query_builder = self.query_builder.select_all();
        self
    }

    /// Add a WHERE condition
    pub fn where_eq(mut self, column: &str, value: impl Into<Value>) -> Self {
        let condition = Condition::Equals(column.to_string(), value.into());
        self.conditions.push(condition);
        self
    }

    /// Add a WHERE NOT condition
    pub fn where_not_eq(mut self, column: &str, value: impl Into<Value>) -> Self {
        let condition = Condition::NotEquals(column.to_string(), value.into());
        self.conditions.push(condition);
        self
    }

    /// Add a WHERE > condition
    pub fn where_gt(mut self, column: &str, value: impl Into<Value>) -> Self {
        let condition = Condition::GreaterThan(column.to_string(), value.into());
        self.conditions.push(condition);
        self
    }

    /// Add a WHERE >= condition
    pub fn where_gte(mut self, column: &str, value: impl Into<Value>) -> Self {
        let condition = Condition::GreaterThanOrEqual(column.to_string(), value.into());
        self.conditions.push(condition);
        self
    }

    /// Add a WHERE < condition
    pub fn where_lt(mut self, column: &str, value: impl Into<Value>) -> Self {
        let condition = Condition::LessThan(column.to_string(), value.into());
        self.conditions.push(condition);
        self
    }

    /// Add a WHERE <= condition
    pub fn where_lte(mut self, column: &str, value: impl Into<Value>) -> Self {
        let condition = Condition::LessThanOrEqual(column.to_string(), value.into());
        self.conditions.push(condition);
        self
    }

    /// Add a WHERE LIKE condition
    pub fn where_like(mut self, column: &str, pattern: impl Into<String>) -> Self {
        let condition = Condition::Like(column.to_string(), Value::String(pattern.into()));
        self.conditions.push(condition);
        self
    }

    /// Add a WHERE IN condition
    pub fn where_in(mut self, column: &str, values: Vec<Value>) -> Self {
        let condition = Condition::In(column.to_string(), values);
        self.conditions.push(condition);
        self
    }

    /// Add a WHERE IS NULL condition
    pub fn where_null(mut self, column: &str) -> Self {
        let condition = Condition::IsNull(column.to_string());
        self.conditions.push(condition);
        self
    }

    /// Add a WHERE IS NOT NULL condition
    pub fn where_not_null(mut self, column: &str) -> Self {
        let condition = Condition::IsNotNull(column.to_string());
        self.conditions.push(condition);
        self
    }

    /// Add ORDER BY clause
    pub fn order_by(mut self, column: &str, order: Order) -> Self {
        let sql_order = match order {
            Order::Asc => SqlOrder::Asc,
            Order::Desc => SqlOrder::Desc,
        };
        self.query_builder = self.query_builder.order_by(column, sql_order);
        self
    }

    /// Add LIMIT clause
    pub fn limit(mut self, limit: usize) -> Self {
        self.query_builder = self.query_builder.limit(limit);
        self
    }

    /// Add OFFSET clause
    pub fn offset(mut self, offset: usize) -> Self {
        self.query_builder = self.query_builder.offset(offset);
        self
    }

    /// Build the SQL query
    fn build_query(&mut self) -> String {
        // Apply conditions to query builder
        for condition in &self.conditions {
            let where_clause = condition.to_sql();
            self.query_builder = self.query_builder.clone().where_clause(&where_clause);

            // Collect parameters
            if let Some(value) = condition.value() {
                self.params.push(value.clone());
            }
        }

        self.query_builder.build()
    }

    /// Execute the query and return all results
    pub fn get(mut self, executor: &mut dyn QueryExecutor) -> Result<Vec<T>, ConnectionError> {
        self.query_builder = self.query_builder.select_all();
        let sql = self.build_query();
        let results = executor.query(&sql, &self.params)?;

        results
            .into_iter()
            .map(|row| T::from_values(row).map_err(ConnectionError::from))
            .collect()
    }

    /// Execute the query and return the first result
    pub fn first(mut self, executor: &mut dyn QueryExecutor) -> Result<Option<T>, ConnectionError> {
        self.query_builder = self.query_builder.select_all().limit(1);
        let sql = self.build_query();
        let results = executor.query(&sql, &self.params)?;

        if let Some(row) = results.into_iter().next() {
            T::from_values(row)
                .map(Some)
                .map_err(ConnectionError::from)
        } else {
            Ok(None)
        }
    }

    /// Count the number of matching rows
    pub fn count(mut self, executor: &mut dyn QueryExecutor) -> Result<u64, ConnectionError> {
        self.query_builder = self.query_builder.select(&["COUNT(*) as count"]);
        let sql = self.build_query();
        let results = executor.query(&sql, &self.params)?;

        if let Some(row) = results.into_iter().next() {
            if let Some(count) = row.get("count").and_then(|v| v.as_i64()) {
                return Ok(count as u64);
            }
        }

        Ok(0)
    }

    /// Check if any rows match the query
    pub fn exists(self, executor: &mut dyn QueryExecutor) -> Result<bool, ConnectionError> {
        Ok(self.count(executor)? > 0)
    }
}

impl<T: Model> Default for Query<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Query condition types
#[derive(Debug, Clone)]
enum Condition {
    Equals(String, Value),
    NotEquals(String, Value),
    GreaterThan(String, Value),
    GreaterThanOrEqual(String, Value),
    LessThan(String, Value),
    LessThanOrEqual(String, Value),
    Like(String, Value),
    In(String, Vec<Value>),
    IsNull(String),
    IsNotNull(String),
}

impl Condition {
    /// Convert condition to SQL string
    fn to_sql(&self) -> String {
        match self {
            Condition::Equals(col, _) => format!("{} = ?", col),
            Condition::NotEquals(col, _) => format!("{} != ?", col),
            Condition::GreaterThan(col, _) => format!("{} > ?", col),
            Condition::GreaterThanOrEqual(col, _) => format!("{} >= ?", col),
            Condition::LessThan(col, _) => format!("{} < ?", col),
            Condition::LessThanOrEqual(col, _) => format!("{} <= ?", col),
            Condition::Like(col, _) => format!("{} LIKE ?", col),
            Condition::In(col, values) => {
                let placeholders = vec!["?"; values.len()].join(", ");
                format!("{} IN ({})", col, placeholders)
            }
            Condition::IsNull(col) => format!("{} IS NULL", col),
            Condition::IsNotNull(col) => format!("{} IS NOT NULL", col),
        }
    }

    /// Get the value for this condition (if any)
    fn value(&self) -> Option<&Value> {
        match self {
            Condition::Equals(_, v)
            | Condition::NotEquals(_, v)
            | Condition::GreaterThan(_, v)
            | Condition::GreaterThanOrEqual(_, v)
            | Condition::LessThan(_, v)
            | Condition::LessThanOrEqual(_, v)
            | Condition::Like(_, v) => Some(v),
            Condition::IsNull(_) | Condition::IsNotNull(_) => None,
            Condition::In(_, _) => None, // Handled separately
        }
    }
}

/// Sort order for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

/// Insert builder for creating new records
pub struct Insert<T: Model> {
    values: HashMap<String, Value>,
    _phantom: PhantomData<T>,
}

impl<T: Model> Insert<T> {
    /// Create a new insert builder
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    /// Set a column value
    pub fn set(mut self, column: impl Into<String>, value: impl Into<Value>) -> Self {
        self.values.insert(column.into(), value.into());
        self
    }

    /// Set values from a model instance
    pub fn from_model(model: &T) -> Self {
        Self {
            values: model.to_values(),
            _phantom: PhantomData,
        }
    }

    /// Execute the insert
    pub fn execute(self, executor: &mut dyn QueryExecutor) -> Result<u64, ConnectionError> {
        let columns: Vec<(&str, &str)> = self
            .values.keys().map(|k| (k.as_str(), "?"))
            .collect();

        let sql = QueryBuilder::insert(T::table_name(), &columns);
        let params: Vec<Value> = self.values.into_values().collect();

        executor.execute(&sql, &params)
    }
}

impl<T: Model> Default for Insert<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Update builder for modifying existing records
pub struct Update<T: Model> {
    values: HashMap<String, Value>,
    conditions: Vec<Condition>,
    _phantom: PhantomData<T>,
}

impl<T: Model> Update<T> {
    /// Create a new update builder
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            conditions: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Set a column value
    pub fn set(mut self, column: impl Into<String>, value: impl Into<Value>) -> Self {
        self.values.insert(column.into(), value.into());
        self
    }

    /// Add a WHERE condition
    pub fn where_eq(mut self, column: &str, value: impl Into<Value>) -> Self {
        self.conditions
            .push(Condition::Equals(column.to_string(), value.into()));
        self
    }

    /// Execute the update
    pub fn execute(self, executor: &mut dyn QueryExecutor) -> Result<u64, ConnectionError> {
        let updates: Vec<(&str, &str)> = self
            .values.keys().map(|k| (k.as_str(), "?"))
            .collect();

        // Build WHERE clause
        let where_parts: Vec<String> = self.conditions.iter().map(|c| c.to_sql()).collect();
        let where_clause = where_parts.join(" AND ");

        let sql = QueryBuilder::update(T::table_name(), &updates, &where_clause);

        // Collect parameters (update values first, then condition values)
        let mut params: Vec<Value> = self.values.into_values().collect();
        for condition in &self.conditions {
            if let Some(value) = condition.value() {
                params.push(value.clone());
            }
        }

        executor.execute(&sql, &params)
    }
}

impl<T: Model> Default for Update<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Delete builder for removing records
pub struct Delete<T: Model> {
    conditions: Vec<Condition>,
    _phantom: PhantomData<T>,
}

impl<T: Model> Delete<T> {
    /// Create a new delete builder
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Add a WHERE condition
    pub fn where_eq(mut self, column: &str, value: impl Into<Value>) -> Self {
        self.conditions
            .push(Condition::Equals(column.to_string(), value.into()));
        self
    }

    /// Execute the delete
    pub fn execute(self, executor: &mut dyn QueryExecutor) -> Result<u64, ConnectionError> {
        // Build WHERE clause
        let where_parts: Vec<String> = self.conditions.iter().map(|c| c.to_sql()).collect();
        let where_clause = where_parts.join(" AND ");

        let sql = QueryBuilder::delete(T::table_name(), &where_clause);

        // Collect parameters
        let params: Vec<Value> = self
            .conditions
            .iter()
            .filter_map(|c| c.value().cloned())
            .collect();

        executor.execute(&sql, &params)
    }
}

impl<T: Model> Default for Delete<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orm::schema::{ColumnType, Table};

    // Test model
    struct TestUser {
        id: Option<i64>,
        name: String,
        email: String,
    }

    impl Model for TestUser {
        fn table_name() -> &'static str {
            "users"
        }

        fn schema() -> Table {
            Table::new("users")
                .add_column(
                    crate::orm::schema::Column::new("id", ColumnType::BigInteger)
                        .primary_key()
                        .auto_increment(),
                )
                .add_column(
                    crate::orm::schema::Column::new("name", ColumnType::String(255)).not_null(),
                )
                .add_column(
                    crate::orm::schema::Column::new("email", ColumnType::String(255)).unique(),
                )
        }

        fn to_values(&self) -> HashMap<String, Value> {
            let mut values = HashMap::new();
            if let Some(id) = self.id {
                values.insert("id".to_string(), Value::Integer(id));
            }
            values.insert("name".to_string(), Value::String(self.name.clone()));
            values.insert("email".to_string(), Value::String(self.email.clone()));
            values
        }

        fn from_values(
            values: HashMap<String, Value>,
        ) -> Result<Self, crate::orm::model::ModelError> {
            let id = values.get("id").and_then(|v| v.as_i64());
            let name = values
                .get("name")
                .and_then(|v| v.as_string())
                .ok_or_else(|| crate::orm::model::ModelError::MissingField("name".to_string()))?
                .to_string();
            let email = values
                .get("email")
                .and_then(|v| v.as_string())
                .ok_or_else(|| crate::orm::model::ModelError::MissingField("email".to_string()))?
                .to_string();

            Ok(Self { id, name, email })
        }

        fn columns() -> Vec<&'static str> {
            vec!["id", "name", "email"]
        }
    }

    #[test]
    fn test_query_builder_creation() {
        let query: Query<TestUser> = Query::new();
        let _sql = query.query_builder.build();
        // Just ensure it compiles and doesn't panic
    }

    #[test]
    fn test_condition_to_sql() {
        let cond = Condition::Equals("name".to_string(), Value::String("John".to_string()));
        assert_eq!(cond.to_sql(), "name = ?");

        let cond = Condition::GreaterThan("age".to_string(), Value::Integer(18));
        assert_eq!(cond.to_sql(), "age > ?");

        let cond = Condition::Like("email".to_string(), Value::String("%@example.com".to_string()));
        assert_eq!(cond.to_sql(), "email LIKE ?");

        let cond = Condition::IsNull("deleted_at".to_string());
        assert_eq!(cond.to_sql(), "deleted_at IS NULL");
    }

    #[test]
    fn test_insert_builder() {
        let insert: Insert<TestUser> = Insert::new()
            .set("name", "John Doe")
            .set("email", "john@example.com");

        assert_eq!(insert.values.len(), 2);
    }

    #[test]
    fn test_update_builder() {
        let update: Update<TestUser> = Update::new()
            .set("name", "Jane Doe")
            .where_eq("id", 1i64);

        assert_eq!(update.values.len(), 1);
        assert_eq!(update.conditions.len(), 1);
    }

    #[test]
    fn test_delete_builder() {
        let delete: Delete<TestUser> = Delete::new().where_eq("id", 1i64);

        assert_eq!(delete.conditions.len(), 1);
    }

    #[test]
    fn test_query_where_conditions() {
        let query: Query<TestUser> = Query::new()
            .where_eq("name", "John")
            .where_gt("age", 18i64);

        assert_eq!(query.conditions.len(), 2);
    }

    #[test]
    fn test_query_order_and_limit() {
        let mut query: Query<TestUser> = Query::new()
            .order_by("created_at", Order::Desc)
            .limit(10)
            .offset(20);

        let sql = query.build_query();
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 20"));
        assert!(sql.contains("ORDER BY created_at DESC"));
    }
}
