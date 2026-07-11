// Core trait definition for database drivers

use crate::db_drivers::{
    error::DriverResult,
    types::{ConnectionConfig, DataValue, DatabaseType, QueryResult, Schema, TransactionHandle},
};
use async_trait::async_trait;

/// Connection status information
#[derive(Debug, Clone)]
pub struct ConnectionStatus {
    pub is_connected: bool,
    pub message: String,
    pub server_version: Option<String>,
    pub latency_ms: Option<u64>,
}

/// Core database driver trait
/// All database implementations must implement this trait
#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    /// Establish connection to the database
    async fn connect(&mut self, config: &ConnectionConfig) -> DriverResult<()>;

    /// Close the database connection
    async fn disconnect(&mut self) -> DriverResult<()>;

    /// Test if the connection is alive and working
    async fn test_connection(&self) -> DriverResult<ConnectionStatus>;

    /// Execute a query and return results
    /// For SQL databases, this is a SQL query
    /// For NoSQL databases, this could be native query format
    async fn execute_query(&self, query: &str, params: Vec<DataValue>)
        -> DriverResult<QueryResult>;

    /// Get list of available databases
    async fn get_databases(&self) -> DriverResult<Vec<String>>;

    /// Get complete schema for a database (tables, columns, indexes, foreign keys)
    async fn get_schema(&self, database: &str) -> DriverResult<Schema>;

    /// Begin a new transaction (if supported)
    async fn begin_transaction(&mut self) -> DriverResult<TransactionHandle>;

    /// Commit the active transaction
    async fn commit_transaction(&mut self, handle: &TransactionHandle) -> DriverResult<()>;

    /// Rollback the active transaction
    async fn rollback_transaction(&mut self, handle: &TransactionHandle) -> DriverResult<()>;

    /// Get the database type
    fn database_type(&self) -> DatabaseType;

    /// Check if this database supports transactions
    fn supports_transactions(&self) -> bool;

    /// Check if currently connected
    fn is_connected(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock driver for testing
    struct MockDriver {
        connected: bool,
    }

    #[async_trait]
    impl DatabaseDriver for MockDriver {
        async fn connect(&mut self, _config: &ConnectionConfig) -> DriverResult<()> {
            self.connected = true;
            Ok(())
        }

        async fn disconnect(&mut self) -> DriverResult<()> {
            self.connected = false;
            Ok(())
        }

        async fn test_connection(&self) -> DriverResult<ConnectionStatus> {
            Ok(ConnectionStatus {
                is_connected: self.connected,
                message: "OK".to_string(),
                server_version: Some("1.0.0".to_string()),
                latency_ms: Some(10),
            })
        }

        async fn execute_query(
            &self,
            _query: &str,
            _params: Vec<DataValue>,
        ) -> DriverResult<QueryResult> {
            Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time_ms: 0,
            })
        }

        async fn get_databases(&self) -> DriverResult<Vec<String>> {
            Ok(vec!["test_db".to_string()])
        }

        async fn get_schema(&self, _database: &str) -> DriverResult<Schema> {
            Ok(Schema {
                database_name: "test_db".to_string(),
                tables: vec![],
            })
        }

        async fn begin_transaction(&mut self) -> DriverResult<TransactionHandle> {
            Ok(TransactionHandle {
                id: "tx_123".to_string(),
                active: true,
            })
        }

        async fn commit_transaction(&mut self, _handle: &TransactionHandle) -> DriverResult<()> {
            Ok(())
        }

        async fn rollback_transaction(&mut self, _handle: &TransactionHandle) -> DriverResult<()> {
            Ok(())
        }

        fn database_type(&self) -> DatabaseType {
            DatabaseType::PostgreSQL
        }

        fn supports_transactions(&self) -> bool {
            true
        }

        fn is_connected(&self) -> bool {
            self.connected
        }
    }

    #[tokio::test]
    async fn test_mock_driver_connect() {
        let mut driver = MockDriver { connected: false };
        assert!(!driver.is_connected());

        let config = ConnectionConfig {
            database_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database: Some("test".to_string()),
            username: "postgres".to_string(),
            password: "password".to_string(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: std::collections::HashMap::new(),
        };

        driver.connect(&config).await.unwrap();
        assert!(driver.is_connected());
    }

    #[tokio::test]
    async fn test_mock_driver_disconnect() {
        let mut driver = MockDriver { connected: true };
        assert!(driver.is_connected());

        driver.disconnect().await.unwrap();
        assert!(!driver.is_connected());
    }
}
