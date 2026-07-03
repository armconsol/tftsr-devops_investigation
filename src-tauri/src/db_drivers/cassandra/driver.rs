// Cassandra driver implementation using Scylla driver (pure Rust)

use async_trait::async_trait;
use scylla::{Session, SessionBuilder};
use std::sync::Arc;

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    traits::{ConnectionStatus, DatabaseDriver},
    types::{
        ColumnMetadata, ConnectionConfig, DataValue, DatabaseType, QueryResult, Schema,
        TransactionHandle,
    },
};

use super::schema::CassandraSchemaIntrospector;
use super::types::cql_value_to_data_value;

pub struct CassandraDriver {
    session: Option<Arc<Session>>,
    config: ConnectionConfig,
}

impl CassandraDriver {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            session: None,
            config,
        }
    }

    fn ensure_connected(&self) -> DriverResult<Arc<Session>> {
        self.session
            .as_ref()
            .cloned()
            .ok_or(DriverError::NotConnected)
    }
}

#[async_trait]
impl DatabaseDriver for CassandraDriver {
    async fn connect(&mut self, config: &ConnectionConfig) -> DriverResult<()> {
        if self.session.is_some() {
            return Err(DriverError::AlreadyConnected);
        }

        // Build contact point
        let contact_point = format!("{}:{}", config.host, config.port);

        let mut builder = SessionBuilder::new().known_node(&contact_point);

        // Add authentication if provided
        if !config.username.is_empty() && !config.password.is_empty() {
            builder = builder.user(&config.username, &config.password);
        }

        // Build session
        let session = builder
            .build()
            .await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        // If a keyspace is specified, use it
        if let Some(keyspace) = &config.database {
            session.use_keyspace(keyspace, false).await.map_err(|e| {
                DriverError::ConnectionFailed(format!(
                    "Failed to use keyspace '{}': {}",
                    keyspace, e
                ))
            })?;
        }

        self.session = Some(Arc::new(session));
        self.config = config.clone();

        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        if self.session.is_none() {
            return Err(DriverError::NotConnected);
        }

        self.session = None;
        Ok(())
    }

    async fn test_connection(&self) -> DriverResult<ConnectionStatus> {
        let session = self.ensure_connected()?;

        let start = std::time::Instant::now();

        // Execute a simple query to test connection
        let result = session
            .query("SELECT release_version FROM system.local", &[])
            .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(query_result) => {
                let version = query_result
                    .rows
                    .as_ref()
                    .and_then(|rows| rows.first())
                    .and_then(|row| row.columns.first())
                    .and_then(|col_opt| col_opt.as_ref())
                    .and_then(|val| match val {
                        scylla::frame::response::result::CqlValue::Text(s) => Some(s.clone()),
                        _ => None,
                    })
                    .map(|v| format!("Cassandra {}", v))
                    .unwrap_or_else(|| "Cassandra (unknown version)".to_string());

                Ok(ConnectionStatus {
                    is_connected: true,
                    message: "Connection successful".to_string(),
                    server_version: Some(version),
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
        params: Vec<DataValue>,
    ) -> DriverResult<QueryResult> {
        let session = self.ensure_connected()?;

        if !params.is_empty() {
            return Err(DriverError::UnsupportedOperation(
                "Parameterized CQL queries are not supported by this driver path yet".to_string(),
            ));
        }

        let start = std::time::Instant::now();

        // Execute CQL query
        let query_result = session
            .query(query, &[])
            .await
            .map_err(|e| DriverError::QueryExecutionFailed(e.to_string()))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Extract column metadata
        let columns: Vec<ColumnMetadata> = query_result
            .col_specs
            .iter()
            .map(|col_spec| ColumnMetadata {
                name: col_spec.name.clone(),
                data_type: format!("{:?}", col_spec.typ), // Use Debug format instead of Display
                nullable: true, // CQL doesn't provide nullable info in query results
                primary_key: false,
            })
            .collect();

        // Convert rows
        let rows = if let Some(result_rows) = query_result.rows {
            let mut converted_rows = Vec::new();

            for row in result_rows {
                let mut converted_row = Vec::new();

                for column_value in row.columns {
                    let data_value = if let Some(cql_val) = column_value {
                        cql_value_to_data_value(&cql_val)?
                    } else {
                        DataValue::Null
                    };
                    converted_row.push(data_value);
                }

                converted_rows.push(converted_row);
            }

            converted_rows
        } else {
            Vec::new()
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
        let session = self.ensure_connected()?;
        let introspector = CassandraSchemaIntrospector::new(session);
        introspector.get_keyspaces().await
    }

    async fn get_schema(&self, database: &str) -> DriverResult<Schema> {
        let session = self.ensure_connected()?;
        let introspector = CassandraSchemaIntrospector::new(session);
        introspector.get_schema(database).await
    }

    async fn begin_transaction(&mut self) -> DriverResult<TransactionHandle> {
        // Cassandra does not support traditional ACID transactions
        Err(DriverError::UnsupportedOperation(
            "Cassandra does not support traditional ACID transactions. Use BATCH statements for atomic operations on a single partition.".to_string(),
        ))
    }

    async fn commit_transaction(&mut self, _handle: &TransactionHandle) -> DriverResult<()> {
        Err(DriverError::UnsupportedOperation(
            "Cassandra transactions not supported".to_string(),
        ))
    }

    async fn rollback_transaction(&mut self, _handle: &TransactionHandle) -> DriverResult<()> {
        Err(DriverError::UnsupportedOperation(
            "Cassandra transactions not supported".to_string(),
        ))
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::Cassandra
    }

    fn supports_transactions(&self) -> bool {
        false
    }

    fn is_connected(&self) -> bool {
        self.session.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cassandra_driver_creation() {
        let config = ConnectionConfig {
            database_type: DatabaseType::Cassandra,
            host: "localhost".to_string(),
            port: 9042,
            database: Some("test_keyspace".to_string()),
            username: "cassandra".to_string(),
            password: "cassandra".to_string(),
            ssl_config: None,
            options: std::collections::HashMap::new(),
        };

        let driver = CassandraDriver::new(config);
        assert!(!driver.is_connected());
        assert_eq!(driver.database_type(), DatabaseType::Cassandra);
        assert!(!driver.supports_transactions());
    }

    #[test]
    fn test_cassandra_driver_properties() {
        let config = ConnectionConfig {
            database_type: DatabaseType::Cassandra,
            host: "127.0.0.1".to_string(),
            port: 9042,
            database: None,
            username: String::new(),
            password: String::new(),
            ssl_config: None,
            options: std::collections::HashMap::new(),
        };

        let driver = CassandraDriver::new(config);
        assert!(!driver.is_connected());
        assert_eq!(driver.database_type(), DatabaseType::Cassandra);
        assert!(!driver.supports_transactions());
    }
}
