// MySQL driver implementation using mysql_async

use async_trait::async_trait;
use mysql_async::{prelude::*, Conn, Opts, OptsBuilder, Pool, Row};
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

use super::schema::MySQLSchemaIntrospector;
use super::types::MySQLTypeConverter;

pub struct MySQLDriver {
    pool: Option<Pool>,
    conn: Option<Arc<Mutex<Conn>>>,
    config: ConnectionConfig,
    transaction_active: bool,
}

impl MySQLDriver {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            pool: None,
            conn: None,
            config,
            transaction_active: false,
        }
    }

    async fn ensure_connected(&self) -> DriverResult<Arc<Mutex<Conn>>> {
        self.conn.as_ref().cloned().ok_or(DriverError::NotConnected)
    }

    fn build_opts(&self, config: &ConnectionConfig) -> Opts {
        let mut builder = OptsBuilder::default()
            .ip_or_hostname(&config.host)
            .tcp_port(config.port)
            .user(Some(&config.username))
            .pass(Some(&config.password));

        if let Some(ref db) = config.database {
            builder = builder.db_name(Some(db));
        }

        // Apply additional options
        for (key, value) in &config.options {
            match key.as_str() {
                "max_connections" => {
                    if let Ok(max) = value.parse::<usize>() {
                        builder =
                            builder.pool_opts(mysql_async::PoolOpts::default().with_constraints(
                                mysql_async::PoolConstraints::new(1, max).unwrap(),
                            ));
                    }
                }
                _ => {
                    // Additional options can be added here
                }
            }
        }

        builder.into()
    }
}

#[async_trait]
impl DatabaseDriver for MySQLDriver {
    async fn connect(&mut self, config: &ConnectionConfig) -> DriverResult<()> {
        if self.conn.is_some() {
            return Err(DriverError::AlreadyConnected);
        }

        let opts = self.build_opts(config);
        let pool = Pool::new(opts);

        // Get a connection from the pool to verify connectivity
        let conn = pool
            .get_conn()
            .await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        self.pool = Some(pool);
        self.conn = Some(Arc::new(Mutex::new(conn)));
        self.config = config.clone();

        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        if self.conn.is_none() {
            return Err(DriverError::NotConnected);
        }

        // Drop connection first
        self.conn = None;

        // Then disconnect the pool
        if let Some(pool) = self.pool.take() {
            pool.disconnect()
                .await
                .map_err(|e| DriverError::Other(format!("Failed to disconnect pool: {}", e)))?;
        }

        self.transaction_active = false;

        Ok(())
    }

    async fn test_connection(&self) -> DriverResult<ConnectionStatus> {
        let conn_arc = self.ensure_connected().await?;
        let mut conn = conn_arc.lock().await;

        let start = std::time::Instant::now();

        let version_result = conn.query_first::<String, _>("SELECT VERSION()").await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match version_result {
            Ok(Some(version)) => Ok(ConnectionStatus {
                is_connected: true,
                message: "Connection successful".to_string(),
                server_version: Some(version),
                latency_ms: Some(latency_ms),
            }),
            Ok(None) => Ok(ConnectionStatus {
                is_connected: false,
                message: "Connection test returned no version".to_string(),
                server_version: None,
                latency_ms: Some(latency_ms),
            }),
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
        let conn_arc = self.ensure_connected().await?;
        let mut conn = conn_arc.lock().await;

        let start = std::time::Instant::now();

        let rows: Vec<Row> = conn
            .query(query)
            .await
            .map_err(|e| DriverError::QueryExecutionFailed(e.to_string()))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Extract column metadata
        let columns = if let Some(first_row) = rows.first() {
            first_row
                .columns_ref()
                .iter()
                .map(|col| ColumnMetadata {
                    name: col.name_str().to_string(),
                    data_type: format!("{:?}", col.column_type()),
                    nullable: (col.flags() & mysql_async::consts::ColumnFlags::NOT_NULL_FLAG)
                        .is_empty(),
                    primary_key: (col.flags() & mysql_async::consts::ColumnFlags::PRI_KEY_FLAG)
                        .bits()
                        != 0,
                })
                .collect()
        } else {
            Vec::new()
        };

        // Convert rows to DataValue vectors
        let converter = MySQLTypeConverter;
        let result_rows: Vec<Vec<DataValue>> = rows
            .iter()
            .map(|row| converter.row_to_data_values(row))
            .collect();

        let row_count = result_rows.len();

        Ok(QueryResult {
            columns,
            rows: result_rows,
            row_count,
            execution_time_ms,
        })
    }

    async fn get_databases(&self) -> DriverResult<Vec<String>> {
        let result = self.execute_query("SHOW DATABASES", vec![]).await?;

        Ok(result
            .rows
            .into_iter()
            .filter_map(|row| {
                if let Some(DataValue::String(name)) = row.first() {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect())
    }

    async fn get_schema(&self, database: &str) -> DriverResult<Schema> {
        let introspector = MySQLSchemaIntrospector::new(self);
        introspector.get_schema(database).await
    }

    async fn begin_transaction(&mut self) -> DriverResult<TransactionHandle> {
        if self.transaction_active {
            return Err(DriverError::TransactionFailed(
                "Transaction already active".to_string(),
            ));
        }

        self.execute_query("START TRANSACTION", vec![]).await?;

        self.transaction_active = true;

        Ok(TransactionHandle {
            id: uuid::Uuid::new_v4().to_string(),
            active: true,
        })
    }

    async fn commit_transaction(&mut self, _handle: &TransactionHandle) -> DriverResult<()> {
        if !self.transaction_active {
            return Err(DriverError::TransactionFailed(
                "No active transaction".to_string(),
            ));
        }

        self.execute_query("COMMIT", vec![]).await?;
        self.transaction_active = false;

        Ok(())
    }

    async fn rollback_transaction(&mut self, _handle: &TransactionHandle) -> DriverResult<()> {
        if !self.transaction_active {
            return Err(DriverError::TransactionFailed(
                "No active transaction".to_string(),
            ));
        }

        self.execute_query("ROLLBACK", vec![]).await?;
        self.transaction_active = false;

        Ok(())
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::MySQL
    }

    fn supports_transactions(&self) -> bool {
        true
    }

    fn is_connected(&self) -> bool {
        self.conn.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysql_driver_creation() {
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

        let driver = MySQLDriver::new(config);
        assert!(!driver.is_connected());
        assert_eq!(driver.database_type(), DatabaseType::MySQL);
        assert!(driver.supports_transactions());
    }
}
