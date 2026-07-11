// Database driver abstraction layer
// Provides unified interface for multiple database systems

pub mod cassandra;
pub mod error;
pub mod import_export;
pub mod mongodb;
pub mod mysql;
pub mod pool;
pub mod postgres;
pub mod redis;
pub mod traits;
pub mod types;
pub mod visualization;

pub use error::{DriverError, DriverResult};
pub use pool::DatabasePoolManager;
pub use traits::{ConnectionStatus, DatabaseDriver};
pub use types::{
    Column, ColumnMetadata, ConnectionConfig, DataValue, DatabaseType, ForeignKey, Index,
    QueryResult, Schema, SslConfig, Table, TransactionHandle,
};

/// Factory function to create appropriate driver instance based on database type
pub fn create_driver(config: &ConnectionConfig) -> DriverResult<Box<dyn DatabaseDriver>> {
    match config.database_type {
        DatabaseType::PostgreSQL => {
            let driver = postgres::PostgresDriver::new(config.clone());
            Ok(Box::new(driver))
        }
        DatabaseType::MySQL => {
            let driver = mysql::MySQLDriver::new(config.clone());
            Ok(Box::new(driver))
        }
        DatabaseType::MongoDB => {
            let driver = mongodb::MongoDBDriver::new(config.clone());
            Ok(Box::new(driver))
        }
        DatabaseType::Redis => {
            let driver = redis::RedisDriver::new(config.clone());
            Ok(Box::new(driver))
        }
        DatabaseType::Cassandra => {
            let driver = cassandra::CassandraDriver::new(config.clone());
            Ok(Box::new(driver))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_postgres_driver() {
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

        let result = create_driver(&config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().database_type(), DatabaseType::PostgreSQL);
    }

    #[test]
    fn test_create_mysql_driver() {
        let config = ConnectionConfig {
            database_type: DatabaseType::MySQL,
            host: "localhost".to_string(),
            port: 3306,
            database: Some("test".to_string()),
            username: "root".to_string(),
            password: "password".to_string(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: std::collections::HashMap::new(),
        };

        let result = create_driver(&config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().database_type(), DatabaseType::MySQL);
    }
}
