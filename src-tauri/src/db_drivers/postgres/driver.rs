// PostgreSQL driver implementation using tokio-postgres

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::{Client, Config, NoTls};

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    traits::{ConnectionStatus, DatabaseDriver},
    types::{
        ColumnMetadata, ConnectionConfig, DataValue, DatabaseType, QueryResult, Schema,
        TransactionHandle,
    },
};

use super::schema::PostgresSchemaIntrospector;
use super::types::PostgresTypeConverter;

pub struct PostgresDriver {
    client: Option<Arc<Mutex<Client>>>,
    config: ConnectionConfig,
    transaction_active: bool,
}

impl PostgresDriver {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            client: None,
            config,
            transaction_active: false,
        }
    }

    async fn ensure_connected(&self) -> DriverResult<Arc<Mutex<Client>>> {
        self.client
            .as_ref()
            .cloned()
            .ok_or(DriverError::NotConnected)
    }
}

#[async_trait]
impl DatabaseDriver for PostgresDriver {
    async fn connect(&mut self, config: &ConnectionConfig) -> DriverResult<()> {
        if self.client.is_some() {
            return Err(DriverError::AlreadyConnected);
        }

        let mut pg_config = Config::new();
        pg_config
            .host(&config.host)
            .port(config.port)
            .dbname(config.database.as_deref().unwrap_or("postgres"))
            .user(&config.username)
            .password(&config.password);

        // Apply additional options
        for (key, value) in &config.options {
            pg_config.options(format!("-c {}={}", key, value));
        }

        tracing::debug!(
            host = %config.host,
            port = %config.port,
            database = ?config.database.as_deref(),
            user = %config.username,
            "Attempting PostgreSQL connection"
        );

        let (client, connection) = pg_config.connect(NoTls).await.map_err(|e| {
            let detail = if let Some(db_err) = e.as_db_error() {
                format!(
                    "{}: {} (code: {}{})",
                    db_err.severity(),
                    db_err.message(),
                    db_err.code().code(),
                    db_err
                        .hint()
                        .map(|h| format!(", hint: {}", h))
                        .unwrap_or_default()
                )
            } else {
                e.to_string()
            };
            tracing::error!(
                host = %config.host,
                port = %config.port,
                database = ?config.database.as_deref(),
                user = %config.username,
                error = %detail,
                "PostgreSQL connection failed"
            );
            DriverError::ConnectionFailed(detail)
        })?;

        // Spawn connection handler with proper tracing
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!(
                    error = %e,
                    "PostgreSQL background connection handler failed"
                );
            }
        });

        self.client = Some(Arc::new(Mutex::new(client)));
        self.config = config.clone();

        tracing::debug!("PostgreSQL connection established successfully");

        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        if self.client.is_none() {
            return Err(DriverError::NotConnected);
        }

        self.client = None;
        self.transaction_active = false;

        Ok(())
    }

    async fn test_connection(&self) -> DriverResult<ConnectionStatus> {
        let client_arc = self.ensure_connected().await?;
        let client = client_arc.lock().await;

        let start = std::time::Instant::now();

        let version_result = client.query_one("SELECT version()", &[]).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match version_result {
            Ok(row) => {
                let version: String = row.get(0);
                Ok(ConnectionStatus {
                    is_connected: true,
                    message: "Connection successful".to_string(),
                    server_version: Some(version),
                    latency_ms: Some(latency_ms),
                })
            }
            Err(e) => {
                let detail = if let Some(db_err) = e.as_db_error() {
                    format!(
                        "{}: {} (code: {}{})",
                        db_err.severity(),
                        db_err.message(),
                        db_err.code().code(),
                        db_err
                            .hint()
                            .map(|h| format!(", hint: {}", h))
                            .unwrap_or_default()
                    )
                } else {
                    e.to_string()
                };
                Ok(ConnectionStatus {
                    is_connected: false,
                    message: format!("Connection test failed: {}", detail),
                    server_version: None,
                    latency_ms: Some(latency_ms),
                })
            }
        }
    }

    async fn execute_query(
        &self,
        query: &str,
        _params: Vec<DataValue>,
    ) -> DriverResult<QueryResult> {
        let client_arc = self.ensure_connected().await?;
        let client = client_arc.lock().await;

        let start = std::time::Instant::now();

        let rows = client
            .query(query, &[])
            .await
            .map_err(|e| DriverError::QueryExecutionFailed(e.to_string()))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Extract column metadata
        let columns = if let Some(first_row) = rows.first() {
            first_row
                .columns()
                .iter()
                .map(|col| ColumnMetadata {
                    name: col.name().to_string(),
                    data_type: format!("{:?}", col.type_()),
                    nullable: true, // PostgreSQL doesn't provide this in query results
                    primary_key: false,
                })
                .collect()
        } else {
            Vec::new()
        };

        // Convert rows to DataValue vectors
        let converter = PostgresTypeConverter;
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
        let result = self
            .execute_query(
                "SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname",
                vec![],
            )
            .await?;

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
        let introspector = PostgresSchemaIntrospector::new(self);
        introspector.get_schema(database).await
    }

    async fn begin_transaction(&mut self) -> DriverResult<TransactionHandle> {
        if self.transaction_active {
            return Err(DriverError::TransactionFailed(
                "Transaction already active".to_string(),
            ));
        }

        self.execute_query("BEGIN", vec![]).await?;

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
        DatabaseType::PostgreSQL
    }

    fn supports_transactions(&self) -> bool {
        true
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_driver_creation() {
        let config = ConnectionConfig {
            database_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database: Some("test".to_string()),
            username: "postgres".to_string(),
            password: "password".to_string(),
            ssl_config: None,
            options: std::collections::HashMap::new(),
        };

        let driver = PostgresDriver::new(config);
        assert!(!driver.is_connected());
        assert_eq!(driver.database_type(), DatabaseType::PostgreSQL);
        assert!(driver.supports_transactions());
    }
}
