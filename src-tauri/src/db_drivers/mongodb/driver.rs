// MongoDB database driver implementation

use super::schema::MongoSchemaInspector;
use super::types::BsonConverter;
use crate::db_drivers::{
    error::{DriverError, DriverResult},
    traits::{ConnectionStatus, DatabaseDriver},
    types::{
        ColumnMetadata, ConnectionConfig, DataValue, DatabaseType, QueryResult, Schema,
        TransactionHandle,
    },
};
use async_trait::async_trait;
use bson::{doc, Document};
use mongodb::{options::ClientOptions, Client, ClientSession};
use std::time::{Duration, Instant};

/// MongoDB driver implementation
pub struct MongoDBDriver {
    config: Option<ConnectionConfig>,
    client: Option<Client>,
    session: Option<ClientSession>,
    current_database: Option<String>,
    active_transactions: std::collections::HashMap<String, ClientSession>,
}

impl MongoDBDriver {
    /// Create a new MongoDB driver instance
    pub fn new(config: ConnectionConfig) -> Self {
        let current_database = config.database.clone();
        Self {
            config: Some(config),
            client: None,
            session: None,
            current_database,
            active_transactions: std::collections::HashMap::new(),
        }
    }

    /// Build MongoDB connection string from config
    fn build_connection_string(config: &ConnectionConfig) -> String {
        let auth = if !config.username.is_empty() {
            format!(
                "{}:{}@",
                urlencoding::encode(&config.username),
                urlencoding::encode(&config.password)
            )
        } else {
            String::new()
        };

        let database = config
            .database
            .as_ref()
            .map(|db| format!("/{}", db))
            .unwrap_or_default();

        // Build options string
        let mut options = Vec::new();

        // SSL/TLS options
        if let Some(ssl) = &config.ssl_config {
            if ssl.enabled {
                options.push("tls=true".to_string());
                if !ssl.verify_server {
                    options.push("tlsAllowInvalidCertificates=true".to_string());
                }
            }
        }

        // Add custom options from config
        for (key, value) in &config.options {
            options.push(format!("{}={}", key, value));
        }

        let options_str = if options.is_empty() {
            String::new()
        } else {
            format!("?{}", options.join("&"))
        };

        format!(
            "mongodb://{}{}:{}{}{}",
            auth, config.host, config.port, database, options_str
        )
    }

    /// Parse a MongoDB query from JSON format
    /// Expected format:
    /// {
    ///   "collection": "users",
    ///   "operation": "find" | "aggregate" | "count",
    ///   "filter": { ... },
    ///   "projection": { ... },
    ///   "sort": { ... },
    ///   "limit": 100,
    ///   "skip": 0,
    ///   "pipeline": [ ... ] // for aggregation
    /// }
    fn parse_query(query_str: &str) -> DriverResult<serde_json::Value> {
        serde_json::from_str(query_str)
            .map_err(|e| DriverError::QueryExecutionFailed(format!("Invalid query JSON: {}", e)))
    }

    /// Execute a find operation
    async fn execute_find(
        &self,
        database_name: &str,
        collection_name: &str,
        query: &serde_json::Value,
    ) -> DriverResult<QueryResult> {
        let client = self.client.as_ref().ok_or(DriverError::NotConnected)?;

        let db = client.database(database_name);
        let collection = db.collection::<Document>(collection_name);

        // Parse filter
        let filter = query
            .get("filter")
            .and_then(|f| bson::to_document(f).ok())
            .unwrap_or_else(|| doc! {});

        // Build find options
        let mut find_options = mongodb::options::FindOptions::default();

        if let Some(limit) = query.get("limit").and_then(|l| l.as_i64()) {
            find_options.limit = Some(limit);
        }

        if let Some(skip) = query.get("skip").and_then(|s| s.as_u64()) {
            find_options.skip = Some(skip);
        }

        if let Some(sort) = query.get("sort").and_then(|s| bson::to_document(s).ok()) {
            find_options.sort = Some(sort);
        }

        if let Some(projection) = query
            .get("projection")
            .and_then(|p| bson::to_document(p).ok())
        {
            find_options.projection = Some(projection);
        }

        // Execute query and measure time
        let start = Instant::now();
        let mut cursor = collection
            .find(filter)
            .with_options(find_options)
            .await
            .map_err(|e| {
                DriverError::QueryExecutionFailed(format!("Find operation failed: {}", e))
            })?;

        let mut rows = Vec::new();
        let mut column_names: Option<Vec<String>> = None;

        // Fetch all documents
        use futures::stream::StreamExt;
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    // Extract column names from first document
                    if column_names.is_none() {
                        column_names = Some(BsonConverter::get_column_names(&doc));
                    }
                    rows.push(BsonConverter::document_to_row(&doc));
                }
                Err(e) => {
                    return Err(DriverError::QueryExecutionFailed(format!(
                        "Error reading document: {}",
                        e
                    )));
                }
            }
        }

        let execution_time = start.elapsed();

        // Build column metadata
        let columns = if let Some(names) = column_names {
            names
                .into_iter()
                .map(|name| ColumnMetadata {
                    name,
                    data_type: "mixed".to_string(), // MongoDB has dynamic types
                    nullable: true,
                    primary_key: false,
                })
                .collect()
        } else {
            vec![]
        };

        Ok(QueryResult {
            columns,
            row_count: rows.len(),
            rows,
            execution_time_ms: execution_time.as_millis() as u64,
        })
    }

    /// Execute an aggregation pipeline
    async fn execute_aggregate(
        &self,
        database_name: &str,
        collection_name: &str,
        query: &serde_json::Value,
    ) -> DriverResult<QueryResult> {
        let client = self.client.as_ref().ok_or(DriverError::NotConnected)?;

        let db = client.database(database_name);
        let collection = db.collection::<Document>(collection_name);

        // Parse pipeline
        let pipeline_json = query.get("pipeline").ok_or_else(|| {
            DriverError::QueryExecutionFailed("Aggregation requires 'pipeline' field".to_string())
        })?;

        let pipeline: Vec<Document> = serde_json::from_value(pipeline_json.clone())
            .map_err(|e| DriverError::QueryExecutionFailed(format!("Invalid pipeline: {}", e)))?;

        // Execute aggregation
        let start = Instant::now();
        let mut cursor = collection
            .aggregate(pipeline)
            .await
            .map_err(|e| DriverError::QueryExecutionFailed(format!("Aggregation failed: {}", e)))?;

        let mut rows = Vec::new();
        let mut column_names: Option<Vec<String>> = None;

        use futures::stream::StreamExt;
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    if column_names.is_none() {
                        column_names = Some(BsonConverter::get_column_names(&doc));
                    }
                    rows.push(BsonConverter::document_to_row(&doc));
                }
                Err(e) => {
                    return Err(DriverError::QueryExecutionFailed(format!(
                        "Error reading result: {}",
                        e
                    )));
                }
            }
        }

        let execution_time = start.elapsed();

        let columns = if let Some(names) = column_names {
            names
                .into_iter()
                .map(|name| ColumnMetadata {
                    name,
                    data_type: "mixed".to_string(),
                    nullable: true,
                    primary_key: false,
                })
                .collect()
        } else {
            vec![]
        };

        Ok(QueryResult {
            columns,
            row_count: rows.len(),
            rows,
            execution_time_ms: execution_time.as_millis() as u64,
        })
    }

    /// Execute a count operation
    async fn execute_count(
        &self,
        database_name: &str,
        collection_name: &str,
        query: &serde_json::Value,
    ) -> DriverResult<QueryResult> {
        let client = self.client.as_ref().ok_or(DriverError::NotConnected)?;

        let db = client.database(database_name);
        let collection = db.collection::<Document>(collection_name);

        // Parse filter
        let filter = query
            .get("filter")
            .and_then(|f| bson::to_document(f).ok())
            .unwrap_or_else(|| doc! {});

        let start = Instant::now();
        let count = collection
            .count_documents(filter)
            .await
            .map_err(|e| DriverError::QueryExecutionFailed(format!("Count failed: {}", e)))?;
        let execution_time = start.elapsed();

        Ok(QueryResult {
            columns: vec![ColumnMetadata {
                name: "count".to_string(),
                data_type: "long".to_string(),
                nullable: false,
                primary_key: false,
            }],
            rows: vec![vec![DataValue::Integer(count as i64)]],
            row_count: 1,
            execution_time_ms: execution_time.as_millis() as u64,
        })
    }

    /// Check if the server supports transactions (requires replica set)
    async fn check_transaction_support(&self) -> bool {
        if let Some(client) = &self.client {
            // Try to get server info
            if let Ok(db) = client
                .database("admin")
                .run_command(doc! { "isMaster": 1 })
                .await
            {
                // Check if this is a replica set or sharded cluster
                return db.get_str("setName").is_ok() || db.get_document("sharding").is_ok();
            }
        }
        false
    }
}

#[async_trait]
impl DatabaseDriver for MongoDBDriver {
    async fn connect(&mut self, config: &ConnectionConfig) -> DriverResult<()> {
        if self.client.is_some() {
            return Err(DriverError::AlreadyConnected);
        }

        // Build connection string
        let conn_str = Self::build_connection_string(config);

        // Parse client options
        let mut client_options = ClientOptions::parse(&conn_str).await.map_err(|e| {
            DriverError::ConnectionFailed(format!("Invalid connection string: {}", e))
        })?;

        // Set app name
        client_options.app_name = Some("tftsr-devops-investigation".to_string());

        // Set timeouts
        client_options.connect_timeout = Some(Duration::from_secs(10));
        client_options.server_selection_timeout = Some(Duration::from_secs(10));

        // Create client
        let client = Client::with_options(client_options).map_err(|e| {
            DriverError::ConnectionFailed(format!("Failed to create client: {}", e))
        })?;

        // Test connection by running a ping command
        client
            .database("admin")
            .run_command(doc! { "ping": 1 })
            .await
            .map_err(|e| DriverError::ConnectionFailed(format!("Ping failed: {}", e)))?;

        self.client = Some(client);
        self.current_database = config.database.clone();
        self.config = Some(config.clone());

        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        // Drop all active sessions
        self.active_transactions.clear();
        self.session = None;

        // Client will be dropped automatically
        self.client = None;
        self.current_database = None;

        Ok(())
    }

    async fn test_connection(&self) -> DriverResult<ConnectionStatus> {
        let client = self.client.as_ref().ok_or(DriverError::NotConnected)?;

        let start = Instant::now();

        // Run server status command to get version and test connection
        let status = client
            .database("admin")
            .run_command(doc! { "serverStatus": 1 })
            .await
            .map_err(|e| DriverError::ConnectionFailed(format!("Server status failed: {}", e)))?;

        let latency = start.elapsed();

        let version = status.get_str("version").unwrap_or("unknown").to_string();

        Ok(ConnectionStatus {
            is_connected: true,
            message: "Connected".to_string(),
            server_version: Some(version),
            latency_ms: Some(latency.as_millis() as u64),
        })
    }

    async fn execute_query(
        &self,
        query: &str,
        _params: Vec<DataValue>,
    ) -> DriverResult<QueryResult> {
        if self.client.is_none() {
            return Err(DriverError::NotConnected);
        }

        // Parse the query JSON
        let query_json = Self::parse_query(query)?;

        // Extract collection and database
        let collection_name = query_json
            .get("collection")
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                DriverError::QueryExecutionFailed(
                    "Query must specify 'collection' field".to_string(),
                )
            })?;

        let database_name = query_json
            .get("database")
            .and_then(|d| d.as_str())
            .or(self.current_database.as_deref())
            .ok_or_else(|| {
                DriverError::QueryExecutionFailed("No database specified".to_string())
            })?;

        // Extract operation type
        let operation = query_json
            .get("operation")
            .and_then(|o| o.as_str())
            .unwrap_or("find");

        // Execute based on operation type
        match operation {
            "find" => {
                self.execute_find(database_name, collection_name, &query_json)
                    .await
            }
            "aggregate" => {
                self.execute_aggregate(database_name, collection_name, &query_json)
                    .await
            }
            "count" => {
                self.execute_count(database_name, collection_name, &query_json)
                    .await
            }
            _ => Err(DriverError::UnsupportedOperation(format!(
                "Unsupported operation: {}. Supported: find, aggregate, count",
                operation
            ))),
        }
    }

    async fn get_databases(&self) -> DriverResult<Vec<String>> {
        let client = self.client.as_ref().ok_or(DriverError::NotConnected)?;

        MongoSchemaInspector::list_databases(client).await
    }

    async fn get_schema(&self, database: &str) -> DriverResult<Schema> {
        let client = self.client.as_ref().ok_or(DriverError::NotConnected)?;

        MongoSchemaInspector::get_schema(client, database).await
    }

    async fn begin_transaction(&mut self) -> DriverResult<TransactionHandle> {
        let client = self.client.as_ref().ok_or(DriverError::NotConnected)?;

        // Check if transactions are supported
        if !self.check_transaction_support().await {
            return Err(DriverError::UnsupportedOperation(
                "Transactions require MongoDB replica set or sharded cluster".to_string(),
            ));
        }

        // Start a new session
        let mut session = client.start_session().await.map_err(|e| {
            DriverError::TransactionFailed(format!("Failed to start session: {}", e))
        })?;

        // Start transaction
        session.start_transaction().await.map_err(|e| {
            DriverError::TransactionFailed(format!("Failed to start transaction: {}", e))
        })?;

        let tx_id = uuid::Uuid::new_v4().to_string();

        let handle = TransactionHandle {
            id: tx_id.clone(),
            active: true,
        };

        self.active_transactions.insert(tx_id, session);

        Ok(handle)
    }

    async fn commit_transaction(&mut self, handle: &TransactionHandle) -> DriverResult<()> {
        let mut session = self
            .active_transactions
            .remove(&handle.id)
            .ok_or_else(|| DriverError::TransactionFailed("Transaction not found".to_string()))?;

        session
            .commit_transaction()
            .await
            .map_err(|e| DriverError::TransactionFailed(format!("Commit failed: {}", e)))?;

        Ok(())
    }

    async fn rollback_transaction(&mut self, handle: &TransactionHandle) -> DriverResult<()> {
        let mut session = self
            .active_transactions
            .remove(&handle.id)
            .ok_or_else(|| DriverError::TransactionFailed("Transaction not found".to_string()))?;

        session
            .abort_transaction()
            .await
            .map_err(|e| DriverError::TransactionFailed(format!("Rollback failed: {}", e)))?;

        Ok(())
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::MongoDB
    }

    fn supports_transactions(&self) -> bool {
        // MongoDB supports transactions only in replica sets or sharded clusters
        // We can't determine this without connecting, so return true
        // The actual check happens in begin_transaction
        true
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_build_connection_string_basic() {
        let config = ConnectionConfig {
            database_type: DatabaseType::MongoDB,
            host: "localhost".to_string(),
            port: 27017,
            database: Some("testdb".to_string()),
            username: String::new(),
            password: String::new(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: HashMap::new(),
        };

        let conn_str = MongoDBDriver::build_connection_string(&config);
        assert_eq!(conn_str, "mongodb://localhost:27017/testdb");
    }

    #[test]
    fn test_build_connection_string_with_auth() {
        let config = ConnectionConfig {
            database_type: DatabaseType::MongoDB,
            host: "localhost".to_string(),
            port: 27017,
            database: Some("testdb".to_string()),
            username: "admin".to_string(),
            password: "secret".to_string(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: HashMap::new(),
        };

        let conn_str = MongoDBDriver::build_connection_string(&config);
        assert_eq!(conn_str, "mongodb://admin:secret@localhost:27017/testdb");
    }

    #[test]
    fn test_build_connection_string_with_special_chars() {
        let config = ConnectionConfig {
            database_type: DatabaseType::MongoDB,
            host: "localhost".to_string(),
            port: 27017,
            database: Some("testdb".to_string()),
            username: "user@domain".to_string(),
            password: "p@ss:word".to_string(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: HashMap::new(),
        };

        let conn_str = MongoDBDriver::build_connection_string(&config);
        // Special characters should be URL-encoded
        assert!(conn_str.contains("user%40domain"));
        assert!(conn_str.contains("p%40ss%3Aword"));
    }

    #[test]
    fn test_build_connection_string_with_ssl() {
        let config = ConnectionConfig {
            database_type: DatabaseType::MongoDB,
            host: "localhost".to_string(),
            port: 27017,
            database: Some("testdb".to_string()),
            username: String::new(),
            password: String::new(),
            ssl_config: Some(crate::db_drivers::types::SslConfig {
                enabled: true,
                ca_cert_path: None,
                client_cert_path: None,
                client_key_path: None,
                verify_server: false,
            }),
            ssh_tunnel_config: None,
            options: HashMap::new(),
        };

        let conn_str = MongoDBDriver::build_connection_string(&config);
        assert!(conn_str.contains("tls=true"));
        assert!(conn_str.contains("tlsAllowInvalidCertificates=true"));
    }

    #[test]
    fn test_parse_query_find() {
        let query = r#"{
            "collection": "users",
            "operation": "find",
            "filter": {"age": {"$gt": 18}},
            "limit": 10
        }"#;

        let result = MongoDBDriver::parse_query(query);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["collection"], "users");
        assert_eq!(json["operation"], "find");
        assert_eq!(json["limit"], 10);
    }

    #[test]
    fn test_parse_query_aggregate() {
        let query = r#"{
            "collection": "orders",
            "operation": "aggregate",
            "pipeline": [
                {"$match": {"status": "completed"}},
                {"$group": {"_id": "$customer", "total": {"$sum": "$amount"}}}
            ]
        }"#;

        let result = MongoDBDriver::parse_query(query);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["operation"], "aggregate");
        assert!(json["pipeline"].is_array());
    }

    #[test]
    fn test_parse_query_invalid_json() {
        let query = "not valid json";
        let result = MongoDBDriver::parse_query(query);
        assert!(result.is_err());
    }

    #[test]
    fn test_driver_not_connected() {
        let config = ConnectionConfig {
            database_type: DatabaseType::MongoDB,
            host: "localhost".to_string(),
            port: 27017,
            database: Some("test".to_string()),
            username: String::new(),
            password: String::new(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: HashMap::new(),
        };

        let driver = MongoDBDriver::new(config);
        assert!(!driver.is_connected());
        assert_eq!(driver.database_type(), DatabaseType::MongoDB);
        assert!(driver.supports_transactions());
    }
}
