//! Database relationship definitions for ORM
//!
//! Provides support for defining and querying relationships between models:
//! - HasOne: One-to-one relationships
//! - HasMany: One-to-many relationships
//! - BelongsTo: Inverse of HasOne/HasMany
//! - BelongsToMany: Many-to-many relationships

use crate::orm::connection::{Connection, ConnectionError};
use crate::orm::model::{Model, Value};
use crate::orm::query::{Query, QueryExecutor};
use std::marker::PhantomData;

/// Relation type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    /// One-to-one relationship
    HasOne,
    /// One-to-many relationship
    HasMany,
    /// Inverse relationship (belongs to parent)
    BelongsTo,
    /// Many-to-many relationship
    BelongsToMany,
}

/// Base trait for all relationships
pub trait Relation<Parent: Model, Related: Model> {
    /// Get the relation type
    fn relation_type(&self) -> RelationType;

    /// Get the foreign key column name
    fn foreign_key(&self) -> &str;

    /// Get the local key column name
    fn local_key(&self) -> &str;

    /// Build a query for this relation
    fn query(&self, parent_id: Value) -> Query<Related>;
}

/// HasOne relationship (one-to-one)
///
/// Example: User HasOne Profile
/// - User table has `id`
/// - Profile table has `user_id` (foreign key)
pub struct HasOne<Parent: Model, Related: Model> {
    foreign_key: String,
    local_key: String,
    _phantom_parent: PhantomData<Parent>,
    _phantom_related: PhantomData<Related>,
}

impl<Parent: Model, Related: Model> HasOne<Parent, Related> {
    /// Create a new HasOne relationship
    ///
    /// # Arguments
    /// * `foreign_key` - Column in the related table that references the parent
    /// * `local_key` - Column in the parent table (usually the primary key)
    pub fn new(foreign_key: impl Into<String>, local_key: impl Into<String>) -> Self {
        Self {
            foreign_key: foreign_key.into(),
            local_key: local_key.into(),
            _phantom_parent: PhantomData,
            _phantom_related: PhantomData,
        }
    }

    /// Get the related model for a parent
    pub fn get(
        &self,
        parent_id: Value,
        executor: &mut dyn QueryExecutor,
    ) -> Result<Option<Related>, ConnectionError> {
        self.query(parent_id).first(executor)
    }
}

impl<Parent: Model, Related: Model> Relation<Parent, Related> for HasOne<Parent, Related> {
    fn relation_type(&self) -> RelationType {
        RelationType::HasOne
    }

    fn foreign_key(&self) -> &str {
        &self.foreign_key
    }

    fn local_key(&self) -> &str {
        &self.local_key
    }

    fn query(&self, parent_id: Value) -> Query<Related> {
        Query::new().where_eq(&self.foreign_key, parent_id)
    }
}

/// HasMany relationship (one-to-many)
///
/// Example: User HasMany Posts
/// - User table has `id`
/// - Posts table has `user_id` (foreign key)
pub struct HasMany<Parent: Model, Related: Model> {
    foreign_key: String,
    local_key: String,
    _phantom_parent: PhantomData<Parent>,
    _phantom_related: PhantomData<Related>,
}

impl<Parent: Model, Related: Model> HasMany<Parent, Related> {
    /// Create a new HasMany relationship
    pub fn new(foreign_key: impl Into<String>, local_key: impl Into<String>) -> Self {
        Self {
            foreign_key: foreign_key.into(),
            local_key: local_key.into(),
            _phantom_parent: PhantomData,
            _phantom_related: PhantomData,
        }
    }

    /// Get all related models for a parent
    pub fn get(
        &self,
        parent_id: Value,
        executor: &mut dyn QueryExecutor,
    ) -> Result<Vec<Related>, ConnectionError> {
        self.query(parent_id).get(executor)
    }

    /// Count the number of related models
    pub fn count(
        &self,
        parent_id: Value,
        executor: &mut dyn QueryExecutor,
    ) -> Result<u64, ConnectionError> {
        self.query(parent_id).count(executor)
    }
}

impl<Parent: Model, Related: Model> Relation<Parent, Related> for HasMany<Parent, Related> {
    fn relation_type(&self) -> RelationType {
        RelationType::HasMany
    }

    fn foreign_key(&self) -> &str {
        &self.foreign_key
    }

    fn local_key(&self) -> &str {
        &self.local_key
    }

    fn query(&self, parent_id: Value) -> Query<Related> {
        Query::new().where_eq(&self.foreign_key, parent_id)
    }
}

/// BelongsTo relationship (inverse of HasOne/HasMany)
///
/// Example: Post BelongsTo User
/// - Post table has `user_id` (foreign key)
/// - User table has `id`
pub struct BelongsTo<Parent: Model, Related: Model> {
    foreign_key: String,
    owner_key: String,
    _phantom_parent: PhantomData<Parent>,
    _phantom_related: PhantomData<Related>,
}

impl<Parent: Model, Related: Model> BelongsTo<Parent, Related> {
    /// Create a new BelongsTo relationship
    ///
    /// # Arguments
    /// * `foreign_key` - Column in the parent table that references the related model
    /// * `owner_key` - Column in the related table (usually the primary key)
    pub fn new(foreign_key: impl Into<String>, owner_key: impl Into<String>) -> Self {
        Self {
            foreign_key: foreign_key.into(),
            owner_key: owner_key.into(),
            _phantom_parent: PhantomData,
            _phantom_related: PhantomData,
        }
    }

    /// Get the related (parent) model
    pub fn get(
        &self,
        foreign_id: Value,
        executor: &mut dyn QueryExecutor,
    ) -> Result<Option<Related>, ConnectionError> {
        self.query(foreign_id).first(executor)
    }
}

impl<Parent: Model, Related: Model> Relation<Parent, Related> for BelongsTo<Parent, Related> {
    fn relation_type(&self) -> RelationType {
        RelationType::BelongsTo
    }

    fn foreign_key(&self) -> &str {
        &self.foreign_key
    }

    fn local_key(&self) -> &str {
        &self.owner_key
    }

    fn query(&self, foreign_id: Value) -> Query<Related> {
        Query::new().where_eq(&self.owner_key, foreign_id)
    }
}

/// BelongsToMany relationship (many-to-many)
///
/// Example: User BelongsToMany Roles (through user_roles pivot table)
/// - User table has `id`
/// - Roles table has `id`
/// - user_roles table has `user_id` and `role_id`
pub struct BelongsToMany<Parent: Model, Related: Model> {
    pivot_table: String,
    foreign_pivot_key: String,
    related_pivot_key: String,
    parent_key: String,
    related_key: String,
    _phantom_parent: PhantomData<Parent>,
    _phantom_related: PhantomData<Related>,
}

impl<Parent: Model, Related: Model> BelongsToMany<Parent, Related> {
    /// Create a new BelongsToMany relationship
    ///
    /// # Arguments
    /// * `pivot_table` - Name of the pivot/junction table
    /// * `foreign_pivot_key` - Column in pivot table referencing parent
    /// * `related_pivot_key` - Column in pivot table referencing related model
    /// * `parent_key` - Column in parent table (usually id)
    /// * `related_key` - Column in related table (usually id)
    pub fn new(
        pivot_table: impl Into<String>,
        foreign_pivot_key: impl Into<String>,
        related_pivot_key: impl Into<String>,
        parent_key: impl Into<String>,
        related_key: impl Into<String>,
    ) -> Self {
        Self {
            pivot_table: pivot_table.into(),
            foreign_pivot_key: foreign_pivot_key.into(),
            related_pivot_key: related_pivot_key.into(),
            parent_key: parent_key.into(),
            related_key: related_key.into(),
            _phantom_parent: PhantomData,
            _phantom_related: PhantomData,
        }
    }

    /// Get the pivot table name
    pub fn pivot_table(&self) -> &str {
        &self.pivot_table
    }

    /// Get the foreign pivot key
    pub fn foreign_pivot_key(&self) -> &str {
        &self.foreign_pivot_key
    }

    /// Get the related pivot key
    pub fn related_pivot_key(&self) -> &str {
        &self.related_pivot_key
    }

    /// Attach a related model (create pivot record)
    pub fn attach(
        &self,
        parent_id: Value,
        related_id: Value,
        conn: &mut dyn Connection,
    ) -> Result<u64, ConnectionError> {
        let sql = format!(
            "INSERT INTO {} ({}, {}) VALUES (?, ?)",
            self.pivot_table, self.foreign_pivot_key, self.related_pivot_key
        );

        conn.execute(&sql, &[parent_id, related_id])
    }

    /// Detach a related model (delete pivot record)
    pub fn detach(
        &self,
        parent_id: Value,
        related_id: Value,
        conn: &mut dyn Connection,
    ) -> Result<u64, ConnectionError> {
        let sql = format!(
            "DELETE FROM {} WHERE {} = ? AND {} = ?",
            self.pivot_table, self.foreign_pivot_key, self.related_pivot_key
        );

        conn.execute(&sql, &[parent_id, related_id])
    }

    /// Sync related models (replace all relationships)
    pub fn sync(
        &self,
        parent_id: Value,
        related_ids: Vec<Value>,
        conn: &mut dyn Connection,
    ) -> Result<(), ConnectionError> {
        // Delete all existing relationships
        let delete_sql = format!(
            "DELETE FROM {} WHERE {} = ?",
            self.pivot_table, self.foreign_pivot_key
        );
        conn.execute(&delete_sql, std::slice::from_ref(&parent_id))?;

        // Insert new relationships
        for related_id in related_ids {
            self.attach(parent_id.clone(), related_id, conn)?;
        }

        Ok(())
    }
}

impl<Parent: Model, Related: Model> Relation<Parent, Related> for BelongsToMany<Parent, Related> {
    fn relation_type(&self) -> RelationType {
        RelationType::BelongsToMany
    }

    fn foreign_key(&self) -> &str {
        &self.foreign_pivot_key
    }

    fn local_key(&self) -> &str {
        &self.parent_key
    }

    fn query(&self, parent_id: Value) -> Query<Related> {
        // For many-to-many, we need a join query
        // This is a simplified version - a real implementation would use JOINs
        Query::new().where_eq(&self.related_key, parent_id)
    }
}

/// Eager loading support for reducing N+1 queries
pub struct EagerLoader<T: Model> {
    models: Vec<T>,
}

impl<T: Model> EagerLoader<T> {
    /// Create a new eager loader
    pub fn new(models: Vec<T>) -> Self {
        Self { models }
    }

    /// Get the loaded models
    pub fn models(&self) -> &[T] {
        &self.models
    }

    /// Get the number of loaded models
    pub fn count(&self) -> usize {
        self.models.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orm::schema::Table;
    use std::collections::HashMap;

    // Test models
    struct User {
        id: Option<i64>,
        name: String,
    }

    struct Post {
        id: Option<i64>,
        user_id: i64,
        title: String,
    }

    struct Profile {
        id: Option<i64>,
        user_id: i64,
        bio: String,
    }

    impl Model for User {
        fn table_name() -> &'static str {
            "users"
        }

        fn schema() -> Table {
            Table::new("users")
        }

        fn to_values(&self) -> HashMap<String, Value> {
            let mut values = HashMap::new();
            if let Some(id) = self.id {
                values.insert("id".to_string(), Value::Integer(id));
            }
            values.insert("name".to_string(), Value::String(self.name.clone()));
            values
        }

        fn from_values(
            values: HashMap<String, Value>,
        ) -> Result<Self, crate::orm::model::ModelError> {
            Ok(Self {
                id: values.get("id").and_then(|v| v.as_i64()),
                name: values
                    .get("name")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string(),
            })
        }

        fn columns() -> Vec<&'static str> {
            vec!["id", "name"]
        }
    }

    impl Model for Post {
        fn table_name() -> &'static str {
            "posts"
        }

        fn schema() -> Table {
            Table::new("posts")
        }

        fn to_values(&self) -> HashMap<String, Value> {
            let mut values = HashMap::new();
            if let Some(id) = self.id {
                values.insert("id".to_string(), Value::Integer(id));
            }
            values.insert("user_id".to_string(), Value::Integer(self.user_id));
            values.insert("title".to_string(), Value::String(self.title.clone()));
            values
        }

        fn from_values(
            values: HashMap<String, Value>,
        ) -> Result<Self, crate::orm::model::ModelError> {
            Ok(Self {
                id: values.get("id").and_then(|v| v.as_i64()),
                user_id: values.get("user_id").and_then(|v| v.as_i64()).unwrap_or(0),
                title: values
                    .get("title")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string(),
            })
        }

        fn columns() -> Vec<&'static str> {
            vec!["id", "user_id", "title"]
        }
    }

    impl Model for Profile {
        fn table_name() -> &'static str {
            "profiles"
        }

        fn schema() -> Table {
            Table::new("profiles")
        }

        fn to_values(&self) -> HashMap<String, Value> {
            let mut values = HashMap::new();
            if let Some(id) = self.id {
                values.insert("id".to_string(), Value::Integer(id));
            }
            values.insert("user_id".to_string(), Value::Integer(self.user_id));
            values.insert("bio".to_string(), Value::String(self.bio.clone()));
            values
        }

        fn from_values(
            values: HashMap<String, Value>,
        ) -> Result<Self, crate::orm::model::ModelError> {
            Ok(Self {
                id: values.get("id").and_then(|v| v.as_i64()),
                user_id: values.get("user_id").and_then(|v| v.as_i64()).unwrap_or(0),
                bio: values
                    .get("bio")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string(),
            })
        }

        fn columns() -> Vec<&'static str> {
            vec!["id", "user_id", "bio"]
        }
    }

    #[test]
    fn test_has_one_relation() {
        let relation: HasOne<User, Profile> = HasOne::new("user_id", "id");

        assert_eq!(relation.relation_type(), RelationType::HasOne);
        assert_eq!(relation.foreign_key(), "user_id");
        assert_eq!(relation.local_key(), "id");
    }

    #[test]
    fn test_has_many_relation() {
        let relation: HasMany<User, Post> = HasMany::new("user_id", "id");

        assert_eq!(relation.relation_type(), RelationType::HasMany);
        assert_eq!(relation.foreign_key(), "user_id");
        assert_eq!(relation.local_key(), "id");
    }

    #[test]
    fn test_belongs_to_relation() {
        let relation: BelongsTo<Post, User> = BelongsTo::new("user_id", "id");

        assert_eq!(relation.relation_type(), RelationType::BelongsTo);
        assert_eq!(relation.foreign_key(), "user_id");
        assert_eq!(relation.local_key(), "id");
    }

    #[test]
    fn test_belongs_to_many_relation() {
        let relation: BelongsToMany<User, Profile> =
            BelongsToMany::new("user_profiles", "user_id", "profile_id", "id", "id");

        assert_eq!(relation.relation_type(), RelationType::BelongsToMany);
        assert_eq!(relation.pivot_table(), "user_profiles");
        assert_eq!(relation.foreign_pivot_key(), "user_id");
        assert_eq!(relation.related_pivot_key(), "profile_id");
    }

    #[test]
    fn test_eager_loader() {
        let users = vec![
            User {
                id: Some(1),
                name: "User 1".to_string(),
            },
            User {
                id: Some(2),
                name: "User 2".to_string(),
            },
        ];

        let loader = EagerLoader::new(users);
        assert_eq!(loader.count(), 2);
        assert_eq!(loader.models().len(), 2);
    }
}
