//! Database query builder for Half framework
//!
//! Provides a type-safe query builder for constructing SQL queries.

use std::collections::HashMap;

/// SQL query builder
///
/// Provides a fluent API for building SQL queries in a type-safe manner.
///
/// # Example
/// ```ignore
/// let query = QueryBuilder::new()
///     .select(&["id", "name", "email"])
///     .from("users")
///     .where_clause("age > ?")
///     .order_by("name", Order::Asc)
///     .limit(10)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    select_columns: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by_clauses: Vec<(String, Order)>,
    limit_value: Option<usize>,
    offset_value: Option<usize>,
    join_clauses: Vec<JoinClause>,
}

/// SQL join types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    /// INNER JOIN
    Inner,
    /// LEFT JOIN
    Left,
    /// RIGHT JOIN
    Right,
    /// FULL OUTER JOIN
    Full,
}

/// Join clause
#[derive(Debug, Clone)]
struct JoinClause {
    join_type: JoinType,
    table: String,
    on_condition: String,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            select_columns: Vec::new(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by_clauses: Vec::new(),
            limit_value: None,
            offset_value: None,
            join_clauses: Vec::new(),
        }
    }

    /// Set the columns to select
    ///
    /// # Example
    /// ```ignore
    /// builder.select(&["id", "name", "email"]);
    /// ```
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.select_columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Select all columns (SELECT *)
    pub fn select_all(mut self) -> Self {
        self.select_columns = vec!["*".to_string()];
        self
    }

    /// Set the table to query from
    pub fn from(mut self, table: &str) -> Self {
        self.from_table = Some(table.to_string());
        self
    }

    /// Add a WHERE clause
    ///
    /// Multiple where clauses are combined with AND.
    ///
    /// # Example
    /// ```ignore
    /// builder.where_clause("age > 18")
    ///        .where_clause("status = 'active'");
    /// // WHERE age > 18 AND status = 'active'
    /// ```
    pub fn where_clause(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    /// Add an ORDER BY clause
    pub fn order_by(mut self, column: &str, order: Order) -> Self {
        self.order_by_clauses.push((column.to_string(), order));
        self
    }

    /// Set the LIMIT clause
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit_value = Some(limit);
        self
    }

    /// Set the OFFSET clause
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset_value = Some(offset);
        self
    }

    /// Add a JOIN clause
    ///
    /// # Example
    /// ```ignore
    /// builder.join(JoinType::Inner, "orders", "users.id = orders.user_id");
    /// ```
    pub fn join(mut self, join_type: JoinType, table: &str, on_condition: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type,
            table: table.to_string(),
            on_condition: on_condition.to_string(),
        });
        self
    }

    /// Build the SQL query string
    ///
    /// # Returns
    /// The SQL query string
    ///
    /// # Panics
    /// Panics if no table is specified
    pub fn build(&self) -> String {
        let mut query = String::new();

        // SELECT clause
        if self.select_columns.is_empty() {
            query.push_str("SELECT *");
        } else {
            query.push_str("SELECT ");
            query.push_str(&self.select_columns.join(", "));
        }

        // FROM clause
        if let Some(table) = &self.from_table {
            query.push_str(" FROM ");
            query.push_str(table);
        } else {
            panic!("No table specified for query");
        }

        // JOIN clauses
        for join in &self.join_clauses {
            query.push(' ');
            query.push_str(match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
                JoinType::Full => "FULL OUTER JOIN",
            });
            query.push(' ');
            query.push_str(&join.table);
            query.push_str(" ON ");
            query.push_str(&join.on_condition);
        }

        // WHERE clauses
        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }

        // ORDER BY clauses
        if !self.order_by_clauses.is_empty() {
            query.push_str(" ORDER BY ");
            let order_parts: Vec<String> = self
                .order_by_clauses
                .iter()
                .map(|(col, order)| {
                    format!(
                        "{} {}",
                        col,
                        match order {
                            Order::Asc => "ASC",
                            Order::Desc => "DESC",
                        }
                    )
                })
                .collect();
            query.push_str(&order_parts.join(", "));
        }

        // LIMIT clause
        if let Some(limit) = self.limit_value {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET clause
        if let Some(offset) = self.offset_value {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }

    /// Build an INSERT query
    ///
    /// # Example
    /// ```ignore
    /// let values = vec![
    ///     ("name", "John"),
    ///     ("email", "john@example.com"),
    /// ];
    /// let query = QueryBuilder::insert("users", &values);
    /// ```
    pub fn insert(table: &str, values: &[(&str, &str)]) -> String {
        let columns: Vec<&str> = values.iter().map(|(col, _)| *col).collect();
        let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();

        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            placeholders.join(", ")
        )
    }

    /// Build an UPDATE query
    ///
    /// # Example
    /// ```ignore
    /// let updates = vec![
    ///     ("name", "John Updated"),
    ///     ("email", "john.updated@example.com"),
    /// ];
    /// let query = QueryBuilder::update("users", &updates, "id = ?");
    /// ```
    pub fn update(table: &str, updates: &[(&str, &str)], where_clause: &str) -> String {
        let set_parts: Vec<String> = updates
            .iter()
            .map(|(col, _)| format!("{} = ?", col))
            .collect();

        format!(
            "UPDATE {} SET {} WHERE {}",
            table,
            set_parts.join(", "),
            where_clause
        )
    }

    /// Build a DELETE query
    ///
    /// # Example
    /// ```ignore
    /// let query = QueryBuilder::delete("users", "id = ?");
    /// ```
    pub fn delete(table: &str, where_clause: &str) -> String {
        format!("DELETE FROM {} WHERE {}", table, where_clause)
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameter binding helper for prepared statements
#[derive(Debug, Clone)]
pub struct QueryParams {
    params: HashMap<String, String>,
}

impl QueryParams {
    /// Create new query parameters
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Bind a parameter
    pub fn bind(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Get a parameter value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    /// Get all parameters
    pub fn all(&self) -> &HashMap<String, String> {
        &self.params
    }
}

impl Default for QueryParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_query() {
        let query = QueryBuilder::new()
            .select(&["id", "name", "email"])
            .from("users")
            .build();

        assert_eq!(query, "SELECT id, name, email FROM users");
    }

    #[test]
    fn test_select_all() {
        let query = QueryBuilder::new().select_all().from("users").build();

        assert_eq!(query, "SELECT * FROM users");
    }

    #[test]
    fn test_where_clause() {
        let query = QueryBuilder::new()
            .select_all()
            .from("users")
            .where_clause("age > 18")
            .where_clause("status = 'active'")
            .build();

        assert_eq!(
            query,
            "SELECT * FROM users WHERE age > 18 AND status = 'active'"
        );
    }

    #[test]
    fn test_order_by() {
        let query = QueryBuilder::new()
            .select_all()
            .from("users")
            .order_by("name", Order::Asc)
            .order_by("age", Order::Desc)
            .build();

        assert_eq!(query, "SELECT * FROM users ORDER BY name ASC, age DESC");
    }

    #[test]
    fn test_limit_offset() {
        let query = QueryBuilder::new()
            .select_all()
            .from("users")
            .limit(10)
            .offset(20)
            .build();

        assert_eq!(query, "SELECT * FROM users LIMIT 10 OFFSET 20");
    }

    #[test]
    fn test_join() {
        let query = QueryBuilder::new()
            .select(&["users.name", "orders.total"])
            .from("users")
            .join(JoinType::Inner, "orders", "users.id = orders.user_id")
            .build();

        assert_eq!(
            query,
            "SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id"
        );
    }

    #[test]
    fn test_complex_query() {
        let query = QueryBuilder::new()
            .select(&["id", "name", "email"])
            .from("users")
            .where_clause("age > 18")
            .where_clause("country = 'US'")
            .order_by("name", Order::Asc)
            .limit(50)
            .offset(100)
            .build();

        assert_eq!(
            query,
            "SELECT id, name, email FROM users WHERE age > 18 AND country = 'US' ORDER BY name ASC LIMIT 50 OFFSET 100"
        );
    }

    #[test]
    fn test_insert() {
        let query =
            QueryBuilder::insert("users", &[("name", "John"), ("email", "john@example.com")]);

        assert_eq!(query, "INSERT INTO users (name, email) VALUES (?, ?)");
    }

    #[test]
    fn test_update() {
        let query = QueryBuilder::update(
            "users",
            &[
                ("name", "John Updated"),
                ("email", "john.updated@example.com"),
            ],
            "id = ?",
        );

        assert_eq!(query, "UPDATE users SET name = ?, email = ? WHERE id = ?");
    }

    #[test]
    fn test_delete() {
        let query = QueryBuilder::delete("users", "id = ?");

        assert_eq!(query, "DELETE FROM users WHERE id = ?");
    }

    #[test]
    fn test_query_params() {
        let params = QueryParams::new()
            .bind("name", "John")
            .bind("email", "john@example.com");

        assert_eq!(params.get("name"), Some(&"John".to_string()));
        assert_eq!(params.get("email"), Some(&"john@example.com".to_string()));
        assert_eq!(params.get("age"), None);
    }

    #[test]
    #[should_panic(expected = "No table specified")]
    fn test_no_table_panics() {
        QueryBuilder::new().select_all().build();
    }
}
