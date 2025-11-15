//! Database connection and connection pooling for ORM
//!
//! Provides abstractions for database connections, connection pooling,
//! and transaction management.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::orm::model::{Value, ModelError};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database URL or connection string
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Idle timeout in seconds (connections idle for this duration are closed)
    pub idle_timeout: u64,
    /// Maximum lifetime for a connection in seconds
    pub max_lifetime: u64,
}

impl DatabaseConfig {
    /// Create a new database configuration
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
        }
    }

    /// Set maximum connections
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set minimum connections
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    /// Set connection timeout
    pub fn connect_timeout(mut self, timeout: u64) -> Self {
        self.connect_timeout = timeout;
        self
    }
}

/// Database connection trait
pub trait Connection: Send + Sync {
    /// Execute a query and return affected rows
    fn execute(&mut self, query: &str, params: &[Value]) -> Result<u64, ConnectionError>;

    /// Execute a query and return results
    fn query(
        &mut self,
        query: &str,
        params: &[Value],
    ) -> Result<Vec<HashMap<String, Value>>, ConnectionError>;

    /// Begin a transaction
    fn begin_transaction(&mut self) -> Result<(), ConnectionError>;

    /// Commit a transaction
    fn commit(&mut self) -> Result<(), ConnectionError>;

    /// Rollback a transaction
    fn rollback(&mut self) -> Result<(), ConnectionError>;

    /// Check if connection is valid
    fn ping(&mut self) -> Result<(), ConnectionError>;

    /// Get the last inserted ID
    fn last_insert_id(&self) -> Option<i64>;
}

/// Connection pool for managing database connections
pub struct ConnectionPool {
    config: DatabaseConfig,
    connections: Arc<RwLock<Vec<PooledConnection>>>,
    stats: Arc<RwLock<PoolStats>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(PoolStats::default())),
        }
    }

    /// Get a connection from the pool
    pub fn get(&self) -> Result<PooledConnection, ConnectionError> {
        let mut connections = self.connections.write();

        // Try to get an available connection
        if let Some(conn) = connections.iter_mut().find(|c| !c.in_use) {
            conn.in_use = true;
            self.stats.write().active_connections += 1;
            return Ok(conn.clone());
        }

        // Create a new connection if pool is not at max capacity
        let total_connections = connections.len();
        if total_connections < self.config.max_connections as usize {
            let conn = self.create_connection()?;
            connections.push(conn.clone());
            self.stats.write().total_connections += 1;
            self.stats.write().active_connections += 1;
            return Ok(conn);
        }

        Err(ConnectionError::PoolExhausted)
    }

    /// Return a connection to the pool
    pub fn release(&self, conn: &PooledConnection) {
        let mut connections = self.connections.write();
        if let Some(pool_conn) = connections.iter_mut().find(|c| c.id == conn.id) {
            pool_conn.in_use = false;
            self.stats.write().active_connections -= 1;
        }
    }

    /// Create a new connection
    fn create_connection(&self) -> Result<PooledConnection, ConnectionError> {
        // In a real implementation, this would create an actual database connection
        // For now, we create a mock connection
        Ok(PooledConnection {
            id: uuid::Uuid::new_v4().to_string(),
            in_use: true,
            created_at: std::time::SystemTime::now(),
            last_used: std::time::SystemTime::now(),
        })
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        *self.stats.read()
    }

    /// Close all connections in the pool
    pub fn close(&self) {
        let mut connections = self.connections.write();
        connections.clear();
        let mut stats = self.stats.write();
        stats.total_connections = 0;
        stats.active_connections = 0;
    }

    /// Get pool configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }
}

/// A pooled database connection
#[derive(Debug, Clone)]
pub struct PooledConnection {
    id: String,
    in_use: bool,
    created_at: std::time::SystemTime,
    last_used: std::time::SystemTime,
}

impl PooledConnection {
    /// Get connection ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Check if connection is in use
    pub fn is_in_use(&self) -> bool {
        self.in_use
    }

    /// Get connection age in seconds
    pub fn age(&self) -> u64 {
        self.created_at
            .elapsed()
            .unwrap_or_default()
            .as_secs()
    }

    /// Get time since last use in seconds
    pub fn idle_time(&self) -> u64 {
        self.last_used
            .elapsed()
            .unwrap_or_default()
            .as_secs()
    }
}

/// Connection pool statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct PoolStats {
    /// Total number of connections
    pub total_connections: u32,
    /// Number of active connections
    pub active_connections: u32,
    /// Number of idle connections
    pub idle_connections: u32,
    /// Total queries executed
    pub total_queries: u64,
    /// Number of connection errors
    pub connection_errors: u64,
}

impl PoolStats {
    /// Get utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            (self.active_connections as f64 / self.total_connections as f64) * 100.0
        }
    }
}

/// Transaction wrapper for managing database transactions
pub struct Transaction<'a> {
    connection: &'a mut dyn Connection,
    committed: bool,
    rolled_back: bool,
}

impl<'a> Transaction<'a> {
    /// Create a new transaction
    pub fn new(connection: &'a mut dyn Connection) -> Result<Self, ConnectionError> {
        connection.begin_transaction()?;
        Ok(Self {
            connection,
            committed: false,
            rolled_back: false,
        })
    }

    /// Commit the transaction
    pub fn commit(mut self) -> Result<(), ConnectionError> {
        self.connection.commit()?;
        self.committed = true;
        Ok(())
    }

    /// Rollback the transaction
    pub fn rollback(mut self) -> Result<(), ConnectionError> {
        self.connection.rollback()?;
        self.rolled_back = true;
        Ok(())
    }

    /// Execute a query within the transaction
    pub fn execute(&mut self, query: &str, params: &[Value]) -> Result<u64, ConnectionError> {
        self.connection.execute(query, params)
    }

    /// Query within the transaction
    pub fn query(
        &mut self,
        query: &str,
        params: &[Value],
    ) -> Result<Vec<HashMap<String, Value>>, ConnectionError> {
        self.connection.query(query, params)
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        // Auto-rollback if not committed or rolled back
        if !self.committed && !self.rolled_back {
            let _ = self.connection.rollback();
        }
    }
}

/// Connection-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Query execution failed
    #[error("Query execution failed: {0}")]
    QueryFailed(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Pool exhausted (no available connections)
    #[error("Connection pool exhausted")]
    PoolExhausted,

    /// Connection timeout
    #[error("Connection timeout")]
    Timeout,

    /// Invalid connection state
    #[error("Invalid connection state: {0}")]
    InvalidState(String),

    /// Model error
    #[error("Model error: {0}")]
    ModelError(#[from] ModelError),
}

/// Mock connection implementation for testing
#[cfg(test)]
pub struct MockConnection {
    in_transaction: bool,
    last_insert_id: Option<i64>,
    query_results: Vec<HashMap<String, Value>>,
}

#[cfg(test)]
impl MockConnection {
    pub fn new() -> Self {
        Self {
            in_transaction: false,
            last_insert_id: None,
            query_results: Vec::new(),
        }
    }

    pub fn set_query_results(&mut self, results: Vec<HashMap<String, Value>>) {
        self.query_results = results;
    }
}

#[cfg(test)]
impl Default for MockConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl Connection for MockConnection {
    fn execute(&mut self, _query: &str, _params: &[Value]) -> Result<u64, ConnectionError> {
        self.last_insert_id = Some(1);
        Ok(1)
    }

    fn query(
        &mut self,
        _query: &str,
        _params: &[Value],
    ) -> Result<Vec<HashMap<String, Value>>, ConnectionError> {
        Ok(self.query_results.clone())
    }

    fn begin_transaction(&mut self) -> Result<(), ConnectionError> {
        if self.in_transaction {
            return Err(ConnectionError::TransactionError(
                "Already in transaction".to_string(),
            ));
        }
        self.in_transaction = true;
        Ok(())
    }

    fn commit(&mut self) -> Result<(), ConnectionError> {
        if !self.in_transaction {
            return Err(ConnectionError::TransactionError(
                "Not in transaction".to_string(),
            ));
        }
        self.in_transaction = false;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), ConnectionError> {
        if !self.in_transaction {
            return Err(ConnectionError::TransactionError(
                "Not in transaction".to_string(),
            ));
        }
        self.in_transaction = false;
        Ok(())
    }

    fn ping(&mut self) -> Result<(), ConnectionError> {
        Ok(())
    }

    fn last_insert_id(&self) -> Option<i64> {
        self.last_insert_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config() {
        let config = DatabaseConfig::new("sqlite::memory:")
            .max_connections(20)
            .min_connections(5)
            .connect_timeout(60);

        assert_eq!(config.url, "sqlite::memory:");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.connect_timeout, 60);
    }

    #[test]
    fn test_connection_pool_creation() {
        let config = DatabaseConfig::new("sqlite::memory:");
        let pool = ConnectionPool::new(config);

        let stats = pool.stats();
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[test]
    fn test_connection_pool_get() {
        let config = DatabaseConfig::new("sqlite::memory:");
        let pool = ConnectionPool::new(config);

        let conn = pool.get().unwrap();
        assert!(conn.is_in_use());

        let stats = pool.stats();
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 1);
    }

    #[test]
    fn test_connection_pool_release() {
        let config = DatabaseConfig::new("sqlite::memory:");
        let pool = ConnectionPool::new(config);

        let conn = pool.get().unwrap();
        pool.release(&conn);

        let stats = pool.stats();
        assert_eq!(stats.active_connections, 0);
    }

    #[test]
    fn test_pool_stats_utilization() {
        let mut stats = PoolStats {
            total_connections: 10,
            active_connections: 5,
            idle_connections: 5,
            total_queries: 100,
            connection_errors: 0,
        };

        assert_eq!(stats.utilization(), 50.0);

        stats.active_connections = 10;
        assert_eq!(stats.utilization(), 100.0);

        stats.active_connections = 0;
        assert_eq!(stats.utilization(), 0.0);
    }

    #[test]
    fn test_mock_connection() {
        let mut conn = MockConnection::new();

        assert!(!conn.in_transaction);
        conn.begin_transaction().unwrap();
        assert!(conn.in_transaction);

        let result = conn.execute("INSERT INTO test VALUES (1)", &[]).unwrap();
        assert_eq!(result, 1);
        assert_eq!(conn.last_insert_id(), Some(1));

        conn.commit().unwrap();
        assert!(!conn.in_transaction);
    }

    #[test]
    fn test_transaction_auto_rollback() {
        let mut conn = MockConnection::new();

        {
            let _tx = Transaction::new(&mut conn).unwrap();
            // Transaction not committed, should auto-rollback on drop
        }

        assert!(!conn.in_transaction);
    }

    #[test]
    fn test_transaction_commit() {
        let mut conn = MockConnection::new();

        let tx = Transaction::new(&mut conn).unwrap();
        tx.commit().unwrap();

        assert!(!conn.in_transaction);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut conn = MockConnection::new();

        let tx = Transaction::new(&mut conn).unwrap();
        tx.rollback().unwrap();

        assert!(!conn.in_transaction);
    }

    #[test]
    fn test_pooled_connection_age() {
        let conn = PooledConnection {
            id: "test-1".to_string(),
            in_use: false,
            created_at: std::time::SystemTime::now(),
            last_used: std::time::SystemTime::now(),
        };

        assert_eq!(conn.age(), 0);
        assert_eq!(conn.idle_time(), 0);
    }
}
