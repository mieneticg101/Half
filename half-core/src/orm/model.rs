//! ORM Model trait and Entity definition
//!
//! Provides the core Model trait that database entities implement,
//! enabling automatic CRUD operations and type-safe queries.

use crate::orm::schema::Table;
use std::collections::HashMap;

/// Core trait for database models
///
/// Implement this trait on your structs to enable ORM functionality.
///
/// # Example
/// ```ignore
/// #[derive(Model)]
/// struct User {
///     id: i32,
///     name: String,
///     email: String,
/// }
/// ```
pub trait Model: Sized {
    /// Get the table name for this model
    fn table_name() -> &'static str;

    /// Get the primary key column name
    fn primary_key() -> &'static str {
        "id"
    }

    /// Get the table schema definition
    fn schema() -> Table;

    /// Convert the model to a HashMap of column names to values
    fn to_values(&self) -> HashMap<String, Value>;

    /// Create a model instance from a HashMap of values
    fn from_values(values: HashMap<String, Value>) -> Result<Self, ModelError>;

    /// Get column names for this model
    fn columns() -> Vec<&'static str>;

    /// Check if a column exists
    fn has_column(name: &str) -> bool {
        Self::columns().contains(&name)
    }
}

/// Entity trait for models with create/update/delete operations
pub trait Entity: Model {
    /// Get the primary key value
    fn id(&self) -> Option<i64>;

    /// Set the primary key value
    fn set_id(&mut self, id: i64);

    /// Check if this is a new record (not yet saved)
    fn is_new(&self) -> bool {
        self.id().is_none()
    }

    /// Check if this record has been persisted
    fn is_persisted(&self) -> bool {
        self.id().is_some()
    }
}

/// Value types that can be stored in database columns
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null value
    Null,
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Binary data
    Binary(Vec<u8>),
    /// JSON value
    Json(String),
}

impl Value {
    /// Convert value to i64
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert value to f64
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Convert value to String
    #[inline]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert value to bool
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if value is null
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Convert to SQL string representation
    #[inline]
    pub fn to_sql_string(&self) -> String {
        match self {
            Value::Null => String::from("NULL"),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => {
                if s.contains('\'') {
                    format!("'{}'", s.replace('\'', "''"))
                } else {
                    format!("'{}'", s)
                }
            }
            Value::Boolean(b) => String::from(if *b { "TRUE" } else { "FALSE" }),
            Value::Binary(_) => String::from("BLOB"),
            Value::Json(j) => {
                if j.contains('\'') {
                    format!("'{}'", j.replace('\'', "''"))
                } else {
                    format!("'{}'", j)
                }
            }
        }
    }
}

/// Convert from various Rust types to Value
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Integer(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v as f64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Boolean(v)
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Binary(v)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(val) => val.into(),
            None => Value::Null,
        }
    }
}

/// Model-related errors
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid field type
    #[error("Invalid type for field {field}: expected {expected}, got {actual}")]
    InvalidType {
        field: String,
        expected: String,
        actual: String,
    },

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Builder for creating models programmatically
pub struct ModelBuilder {
    values: HashMap<String, Value>,
}

impl ModelBuilder {
    /// Create a new model builder
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Set a field value
    pub fn set(mut self, field: impl Into<String>, value: impl Into<Value>) -> Self {
        self.values.insert(field.into(), value.into());
        self
    }

    /// Get a field value
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.values.get(field)
    }

    /// Build a model from the values
    pub fn build<T: Model>(self) -> Result<T, ModelError> {
        T::from_values(self.values)
    }

    /// Get all values
    pub fn values(&self) -> &HashMap<String, Value> {
        &self.values
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribute metadata for model fields
#[derive(Debug, Clone)]
pub struct FieldMetadata {
    /// Field name
    pub name: String,
    /// Whether field is required (not null)
    pub required: bool,
    /// Default value
    pub default: Option<Value>,
    /// Validation rules
    pub validators: Vec<String>,
}

impl FieldMetadata {
    /// Create new field metadata
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: false,
            default: None,
            validators: Vec::new(),
        }
    }

    /// Mark field as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set default value
    pub fn default(mut self, value: impl Into<Value>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Add a validator
    pub fn validator(mut self, rule: impl Into<String>) -> Self {
        self.validators.push(rule.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orm::schema::{Column, ColumnType, Table};

    // Test model implementation
    struct TestUser {
        id: Option<i64>,
        name: String,
        email: String,
        active: bool,
    }

    impl Model for TestUser {
        fn table_name() -> &'static str {
            "users"
        }

        fn schema() -> Table {
            Table::new("users")
                .add_column(
                    Column::new("id", ColumnType::BigInteger)
                        .primary_key()
                        .auto_increment(),
                )
                .add_column(Column::new("name", ColumnType::String(255)).not_null())
                .add_column(Column::new("email", ColumnType::String(255)).unique())
                .add_column(Column::new("active", ColumnType::Boolean).default("TRUE"))
        }

        fn to_values(&self) -> HashMap<String, Value> {
            let mut values = HashMap::new();
            if let Some(id) = self.id {
                values.insert("id".to_string(), Value::Integer(id));
            }
            values.insert("name".to_string(), Value::String(self.name.clone()));
            values.insert("email".to_string(), Value::String(self.email.clone()));
            values.insert("active".to_string(), Value::Boolean(self.active));
            values
        }

        fn from_values(values: HashMap<String, Value>) -> Result<Self, ModelError> {
            let id = values.get("id").and_then(|v| v.as_i64());
            let name = values
                .get("name")
                .and_then(|v| v.as_string())
                .ok_or_else(|| ModelError::MissingField("name".to_string()))?
                .to_string();
            let email = values
                .get("email")
                .and_then(|v| v.as_string())
                .ok_or_else(|| ModelError::MissingField("email".to_string()))?
                .to_string();
            let active = values
                .get("active")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            Ok(Self {
                id,
                name,
                email,
                active,
            })
        }

        fn columns() -> Vec<&'static str> {
            vec!["id", "name", "email", "active"]
        }
    }

    impl Entity for TestUser {
        fn id(&self) -> Option<i64> {
            self.id
        }

        fn set_id(&mut self, id: i64) {
            self.id = Some(id);
        }
    }

    #[test]
    fn test_model_table_name() {
        assert_eq!(TestUser::table_name(), "users");
    }

    #[test]
    fn test_model_primary_key() {
        assert_eq!(TestUser::primary_key(), "id");
    }

    #[test]
    fn test_model_to_values() {
        let user = TestUser {
            id: Some(1),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            active: true,
        };

        let values = user.to_values();
        assert_eq!(values.get("id").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(
            values.get("name").and_then(|v| v.as_string()),
            Some("John Doe")
        );
        assert_eq!(
            values.get("email").and_then(|v| v.as_string()),
            Some("john@example.com")
        );
        assert_eq!(values.get("active").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn test_model_from_values() {
        let mut values = HashMap::new();
        values.insert("id".to_string(), Value::Integer(1));
        values.insert("name".to_string(), Value::String("John Doe".to_string()));
        values.insert(
            "email".to_string(),
            Value::String("john@example.com".to_string()),
        );
        values.insert("active".to_string(), Value::Boolean(true));

        let user = TestUser::from_values(values).unwrap();
        assert_eq!(user.id, Some(1));
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert!(user.active);
    }

    #[test]
    fn test_entity_is_new() {
        let user = TestUser {
            id: None,
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            active: true,
        };
        assert!(user.is_new());
        assert!(!user.is_persisted());
    }

    #[test]
    fn test_entity_is_persisted() {
        let user = TestUser {
            id: Some(1),
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            active: true,
        };
        assert!(!user.is_new());
        assert!(user.is_persisted());
    }

    #[test]
    fn test_value_conversions() {
        let int_val: Value = 42i32.into();
        assert_eq!(int_val.as_i64(), Some(42));

        let str_val: Value = "hello".into();
        assert_eq!(str_val.as_string(), Some("hello"));

        let bool_val: Value = true.into();
        assert_eq!(bool_val.as_bool(), Some(true));

        let null_val: Value = Option::<i32>::None.into();
        assert!(null_val.is_null());
    }

    #[test]
    fn test_value_to_sql_string() {
        assert_eq!(Value::Integer(42).to_sql_string(), "42");
        assert_eq!(Value::String("test".to_string()).to_sql_string(), "'test'");
        assert_eq!(Value::Boolean(true).to_sql_string(), "TRUE");
        assert_eq!(Value::Null.to_sql_string(), "NULL");
    }

    #[test]
    fn test_model_builder() {
        let builder = ModelBuilder::new()
            .set("id", 1i64)
            .set("name", "John Doe")
            .set("email", "john@example.com")
            .set("active", true);

        let user: TestUser = builder.build().unwrap();
        assert_eq!(user.id, Some(1));
        assert_eq!(user.name, "John Doe");
    }

    #[test]
    fn test_field_metadata() {
        let field = FieldMetadata::new("email").required().validator("email");

        assert_eq!(field.name, "email");
        assert!(field.required);
        assert_eq!(field.validators.len(), 1);
    }
}
