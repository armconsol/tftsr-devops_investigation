// Redis driver implementation using redis crate with async support

use async_trait::async_trait;
use redis::{aio::ConnectionManager, Client, RedisError};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    traits::{ConnectionStatus, DatabaseDriver},
    types::{
        ColumnMetadata, ConnectionConfig, DataValue, DatabaseType, QueryResult, Schema,
        TransactionHandle,
    },
};

use super::types::{parse_redis_command, redis_value_to_data_value};

pub struct RedisDriver {
    connection: Option<Arc<Mutex<ConnectionManager>>>,
    config: ConnectionConfig,
}

impl RedisDriver {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            connection: None,
            config,
        }
    }

    async fn ensure_connected(&self) -> DriverResult<Arc<Mutex<ConnectionManager>>> {
        self.connection
            .as_ref()
            .cloned()
            .ok_or(DriverError::NotConnected)
    }

    /// Build Redis connection URL from config
    fn build_connection_url(config: &ConnectionConfig) -> String {
        let auth = if !config.password.is_empty() {
            format!("{}:{}@", config.username, config.password)
        } else if !config.username.is_empty() {
            format!("{}@", config.username)
        } else {
            String::new()
        };

        // Redis database number (0-15)
        let db = config
            .database
            .as_ref()
            .and_then(|d| d.parse::<u8>().ok())
            .unwrap_or(0);

        format!("redis://{}{}:{}/{}", auth, config.host, config.port, db)
    }

    /// Execute Redis command and return result
    async fn execute_redis_command(&self, parts: Vec<String>) -> DriverResult<redis::Value> {
        if parts.is_empty() {
            return Err(DriverError::QueryExecutionFailed(
                "Empty command".to_string(),
            ));
        }

        let conn_arc = self.ensure_connected().await?;
        let mut conn = conn_arc.lock().await;

        // Convert string parts to redis::Arg compatible types
        let args: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();

        // Execute command using redis pipeline
        redis::cmd(args[0])
            .arg(args[1..].to_vec())
            .query_async::<_, redis::Value>(&mut *conn)
            .await
            .map_err(|e: RedisError| DriverError::QueryExecutionFailed(e.to_string()))
    }
}

#[async_trait]
impl DatabaseDriver for RedisDriver {
    async fn connect(&mut self, config: &ConnectionConfig) -> DriverResult<()> {
        if self.connection.is_some() {
            return Err(DriverError::AlreadyConnected);
        }

        let url = Self::build_connection_url(config);

        let client = Client::open(url)
            .map_err(|e: RedisError| DriverError::ConnectionFailed(e.to_string()))?;

        let connection_manager = client
            .get_connection_manager()
            .await
            .map_err(|e: RedisError| DriverError::ConnectionFailed(e.to_string()))?;

        self.connection = Some(Arc::new(Mutex::new(connection_manager)));
        self.config = config.clone();

        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        if self.connection.is_none() {
            return Err(DriverError::NotConnected);
        }

        self.connection = None;
        Ok(())
    }

    async fn test_connection(&self) -> DriverResult<ConnectionStatus> {
        let conn_arc = self.ensure_connected().await?;
        let mut conn = conn_arc.lock().await;

        let start = std::time::Instant::now();

        let ping_result: Result<String, RedisError> =
            redis::cmd("PING").query_async(&mut *conn).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match ping_result {
            Ok(_) => {
                // Get server info for version
                let info_result: Result<String, RedisError> = redis::cmd("INFO")
                    .arg("server")
                    .query_async(&mut *conn)
                    .await;

                let server_version = if let Ok(info) = info_result {
                    // Parse Redis version from INFO output
                    info.lines()
                        .find(|line| line.starts_with("redis_version:"))
                        .and_then(|line| line.split(':').nth(1))
                        .map(|v| format!("Redis {}", v.trim()))
                } else {
                    Some("Redis (unknown version)".to_string())
                };

                Ok(ConnectionStatus {
                    is_connected: true,
                    message: "Connection successful".to_string(),
                    server_version,
                    latency_ms: Some(latency_ms),
                })
            }
            Err(e) => Ok(ConnectionStatus {
                is_connected: false,
                message: format!("Connection test failed: {}", e),
                server_version: None,
                latency_ms: Some(latency_ms),
            }),
        }
    }

    async fn execute_query(
        &self,
        query: &str,
        _params: Vec<DataValue>,
    ) -> DriverResult<QueryResult> {
        let start = std::time::Instant::now();

        // Parse Redis command
        let parts = parse_redis_command(query);
        if parts.is_empty() {
            return Err(DriverError::QueryExecutionFailed("Empty query".to_string()));
        }

        // Execute command
        let redis_value = self.execute_redis_command(parts).await?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Convert result to QueryResult
        let data_value = redis_value_to_data_value(&redis_value)?;

        // Format result based on value type
        let (columns, rows) = match data_value {
            DataValue::Array(arr) => {
                // Multi-value response (e.g., KEYS, LRANGE)
                let cols = vec![ColumnMetadata {
                    name: "value".to_string(),
                    data_type: "string".to_string(),
                    nullable: false,
                    primary_key: false,
                }];

                let result_rows: Vec<Vec<DataValue>> =
                    arr.iter().map(|v| vec![v.clone()]).collect();

                (cols, result_rows)
            }
            _ => {
                // Single value response (e.g., GET, SET)
                let cols = vec![ColumnMetadata {
                    name: "result".to_string(),
                    data_type: "string".to_string(),
                    nullable: false,
                    primary_key: false,
                }];

                (cols, vec![vec![data_value]])
            }
        };

        let row_count = rows.len();

        Ok(QueryResult {
            columns,
            rows,
            row_count,
            execution_time_ms,
        })
    }

    async fn get_databases(&self) -> DriverResult<Vec<String>> {
        // Redis supports database numbers 0-15 by default
        // Return them as strings
        Ok((0..16).map(|i| i.to_string()).collect())
    }

    async fn get_schema(&self, _database: &str) -> DriverResult<Schema> {
        // Redis doesn't have traditional schema
        // Return empty schema with database name
        Ok(Schema {
            database_name: _database.to_string(),
            tables: vec![],
        })
    }

    async fn begin_transaction(&mut self) -> DriverResult<TransactionHandle> {
        // Redis doesn't support traditional transactions in the same way as SQL databases
        // Redis has MULTI/EXEC but that's different from SQL transactions
        Err(DriverError::UnsupportedOperation(
            "Redis does not support traditional ACID transactions. Use MULTI/EXEC for atomic command execution.".to_string(),
        ))
    }

    async fn commit_transaction(&mut self, _handle: &TransactionHandle) -> DriverResult<()> {
        Err(DriverError::UnsupportedOperation(
            "Redis transactions not supported via this interface".to_string(),
        ))
    }

    async fn rollback_transaction(&mut self, _handle: &TransactionHandle) -> DriverResult<()> {
        Err(DriverError::UnsupportedOperation(
            "Redis transactions not supported via this interface".to_string(),
        ))
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::Redis
    }

    fn supports_transactions(&self) -> bool {
        false
    }

    fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_driver_creation() {
        let config = ConnectionConfig {
            database_type: DatabaseType::Redis,
            host: "localhost".to_string(),
            port: 6379,
            database: Some("0".to_string()),
            username: String::new(),
            password: String::new(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: std::collections::HashMap::new(),
        };

        let driver = RedisDriver::new(config);
        assert!(!driver.is_connected());
        assert_eq!(driver.database_type(), DatabaseType::Redis);
        assert!(!driver.supports_transactions());
    }

    #[test]
    fn test_build_connection_url_no_auth() {
        let config = ConnectionConfig {
            database_type: DatabaseType::Redis,
            host: "localhost".to_string(),
            port: 6379,
            database: Some("0".to_string()),
            username: String::new(),
            password: String::new(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: std::collections::HashMap::new(),
        };

        let url = RedisDriver::build_connection_url(&config);
        assert_eq!(url, "redis://localhost:6379/0");
    }

    #[test]
    fn test_build_connection_url_with_password() {
        let config = ConnectionConfig {
            database_type: DatabaseType::Redis,
            host: "localhost".to_string(),
            port: 6379,
            database: Some("1".to_string()),
            username: "default".to_string(),
            password: "secret".to_string(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: std::collections::HashMap::new(),
        };

        let url = RedisDriver::build_connection_url(&config);
        assert_eq!(url, "redis://default:secret@localhost:6379/1");
    }

    #[test]
    fn test_build_connection_url_default_db() {
        let config = ConnectionConfig {
            database_type: DatabaseType::Redis,
            host: "localhost".to_string(),
            port: 6379,
            database: None,
            username: String::new(),
            password: String::new(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: std::collections::HashMap::new(),
        };

        let url = RedisDriver::build_connection_url(&config);
        assert_eq!(url, "redis://localhost:6379/0");
    }
}
