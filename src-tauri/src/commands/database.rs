// Database management commands for import/export, connection management, and visualization

use crate::db::models::{
    ConnectionTestResult, DatabaseConnection, QueryBookmark, QueryExecutionResult, QueryHistory,
};
use crate::db_drivers::{
    import_export::{csv, json, sql, ImportOptions, ImportStats},
    types::{ConnectionConfig, DatabaseType, QueryResult, SslConfig},
    visualization,
};
use crate::state::AppState;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tauri::{AppHandle, State};
use uuid::Uuid;

/// Statistics returned after export operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    pub rows_exported: usize,
    pub file_size_bytes: u64,
    pub execution_time_ms: u64,
}

/// Import CSV data into a table
///
/// # Arguments
/// * `file_path` - Path to CSV file
/// * `connection_id` - Database connection ID
/// * `target_table` - Target table name
/// * `options` - Import options (optional)
#[tauri::command]
pub async fn import_csv_data<R: tauri::Runtime>(
    file_path: String,
    connection_id: String,
    target_table: String,
    options: Option<ImportOptions>,
    app_handle: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<ImportStats, String> {
    let options = options.unwrap_or_default();

    // Get pool manager from app state
    let pool = state.db_pool_manager.lock().await;

    csv::import_csv(
        &file_path,
        &connection_id,
        &target_table,
        options,
        &pool,
        Some(&app_handle),
    )
    .await
    .map_err(|e| format!("CSV import failed: {}", e))
}

/// Import JSON data into a table
///
/// # Arguments
/// * `file_path` - Path to JSON file
/// * `connection_id` - Database connection ID
/// * `target_table` - Target table name
/// * `options` - Import options (optional)
#[tauri::command]
pub async fn import_json_data<R: tauri::Runtime>(
    file_path: String,
    connection_id: String,
    target_table: String,
    options: Option<ImportOptions>,
    app_handle: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<ImportStats, String> {
    let options = options.unwrap_or_default();

    // Get pool manager from app state
    let pool = state.db_pool_manager.lock().await;

    json::import_json(
        &file_path,
        &connection_id,
        &target_table,
        options,
        &pool,
        Some(&app_handle),
    )
    .await
    .map_err(|e| format!("JSON import failed: {}", e))
}

/// Export query results to file
///
/// # Arguments
/// * `query_result` - Query result to export
/// * `format` - Export format: "csv", "json", or "sql"
/// * `output_path` - Output file path
/// * `table_name` - Table name for SQL exports (optional)
#[tauri::command]
pub async fn export_query_results(
    query_result: QueryResult,
    format: String,
    output_path: String,
    table_name: Option<String>,
    _state: State<'_, AppState>,
) -> Result<ExportStats, String> {
    let start_time = std::time::Instant::now();
    let row_count = query_result.rows.len();

    match format.to_lowercase().as_str() {
        "csv" => {
            csv::export_csv(&query_result, &output_path)
                .map_err(|e| format!("CSV export failed: {}", e))?;
        }
        "json" => {
            json::export_json(&query_result, &output_path)
                .map_err(|e| format!("JSON export failed: {}", e))?;
        }
        "sql" => {
            let table =
                table_name.ok_or_else(|| "Table name required for SQL export".to_string())?;
            sql::export_sql_inserts(&query_result, &table, &output_path)
                .map_err(|e| format!("SQL export failed: {}", e))?;
        }
        _ => {
            return Err(format!("Unsupported export format: {}", format));
        }
    }

    // Get file size
    let file_size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(ExportStats {
        rows_exported: row_count,
        file_size_bytes: file_size,
        execution_time_ms,
    })
}

/// Generate ER diagram for a database
///
/// # Arguments
/// * `connection_id` - Database connection ID
/// * `database` - Database name (optional)
#[tauri::command]
pub async fn generate_er_diagram(
    connection_id: String,
    database: Option<String>,
    state: State<'_, AppState>,
) -> Result<visualization::ERDiagramData, String> {
    // Get pool manager from app state
    let pool = state.db_pool_manager.lock().await;

    visualization::generate_er_diagram(&connection_id, database.as_deref(), &pool)
        .await
        .map_err(|e| format!("ER diagram generation failed: {}", e))
}

/// Preview CSV file (first N rows)
///
/// # Arguments
/// * `file_path` - Path to CSV file
/// * `max_rows` - Maximum rows to preview (default: 100)
#[tauri::command]
pub async fn preview_csv_file(
    file_path: String,
    max_rows: Option<usize>,
) -> Result<PreviewData, String> {
    use std::fs::File;
    use std::io::BufReader;

    let max_rows = max_rows.unwrap_or(100);

    let file = File::open(&file_path).map_err(|e| format!("Failed to open CSV file: {}", e))?;

    let reader = BufReader::new(file);
    let mut csv_reader = ::csv::ReaderBuilder::new().from_reader(reader);

    // Get headers
    let headers = csv_reader
        .headers()
        .map_err(|e| format!("Failed to read CSV headers: {}", e))?
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Read preview rows
    let mut rows = Vec::new();
    for (idx, result) in csv_reader.records().enumerate() {
        if idx >= max_rows {
            break;
        }

        match result {
            Ok(record) => {
                let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
                rows.push(row);
            }
            Err(e) => {
                return Err(format!("CSV parsing error at row {}: {}", idx + 1, e));
            }
        }
    }

    Ok(PreviewData { headers, rows })
}

/// Preview JSON file (first N records)
///
/// # Arguments
/// * `file_path` - Path to JSON file
/// * `max_records` - Maximum records to preview (default: 100)
#[tauri::command]
pub async fn preview_json_file(
    file_path: String,
    max_records: Option<usize>,
) -> Result<serde_json::Value, String> {
    use std::fs;

    let max_records = max_records.unwrap_or(100);

    let content =
        fs::read_to_string(&file_path).map_err(|e| format!("Failed to read JSON file: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Extract preview
    let preview = match json {
        serde_json::Value::Array(arr) => {
            let preview_arr: Vec<_> = arr.into_iter().take(max_records).collect();
            serde_json::Value::Array(preview_arr)
        }
        serde_json::Value::Object(obj) => {
            if let Some(serde_json::Value::Array(arr)) = obj.get("data") {
                let preview_arr: Vec<_> = arr.iter().take(max_records).cloned().collect();
                serde_json::json!({ "data": preview_arr })
            } else {
                serde_json::Value::Object(obj)
            }
        }
        other => other,
    };

    Ok(preview)
}

/// Preview data returned by preview commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_preview_csv_basic() {
        // Create temporary CSV file
        let temp_file = std::env::temp_dir().join("test_preview.csv");
        std::fs::write(
            &temp_file,
            "id,name,email\n1,Alice,alice@example.com\n2,Bob,bob@example.com\n",
        )
        .unwrap();

        let result = preview_csv_file(temp_file.to_str().unwrap().to_string(), Some(10))
            .await
            .unwrap();

        assert_eq!(result.headers, vec!["id", "name", "email"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0], vec!["1", "Alice", "alice@example.com"]);

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[tokio::test]
    async fn test_preview_json_array() {
        // Create temporary JSON file
        let temp_file = std::env::temp_dir().join("test_preview.json");
        std::fs::write(
            &temp_file,
            r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#,
        )
        .unwrap();

        let result = preview_json_file(temp_file.to_str().unwrap().to_string(), Some(10))
            .await
            .unwrap();

        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }
}

// ─── Database Connection Management ─────────────────────────────────────────

/// Create a new database connection
#[tauri::command]
pub async fn create_database_connection(
    name: String,
    db_type: String,
    host: String,
    port: u16,
    database_name: Option<String>,
    username: String,
    password: String,
    ssl_enabled: bool,
    ssl_ca_cert_path: Option<String>,
    ssl_client_cert_path: Option<String>,
    ssl_client_key_path: Option<String>,
    connection_options: Option<String>,
    state: State<'_, AppState>,
) -> Result<DatabaseConnection, String> {
    // Encrypt password
    let encrypted_password = crate::integrations::auth::encrypt_token(&password)
        .map_err(|e| format!("Failed to encrypt password: {}", e))?;

    let connection = DatabaseConnection {
        id: Uuid::now_v7().to_string(),
        name: name.clone(),
        db_type: db_type.clone(),
        host: host.clone(),
        port,
        database_name: database_name.clone(),
        username: username.clone(),
        ssl_enabled,
        ssl_ca_cert_path: ssl_ca_cert_path.clone(),
        ssl_client_cert_path: ssl_client_cert_path.clone(),
        ssl_client_key_path: ssl_client_key_path.clone(),
        connection_options: connection_options.clone(),
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    // Store in database
    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        db.execute(
            "INSERT INTO database_connections (id, name, db_type, host, port, database_name, username, encrypted_password, ssl_enabled, ssl_ca_cert_path, ssl_client_cert_path, ssl_client_key_path, connection_options, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                connection.id,
                connection.name,
                connection.db_type,
                connection.host,
                connection.port,
                connection.database_name,
                connection.username,
                encrypted_password,
                if ssl_enabled { 1 } else { 0 },
                ssl_ca_cert_path,
                ssl_client_cert_path,
                ssl_client_key_path,
                connection_options,
                connection.created_at,
                connection.updated_at,
            ],
        )
        .map_err(|e| format!("Failed to store connection: {}", e))?;
    }

    // Audit log
    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        let details = serde_json::json!({
            "name": name,
            "db_type": db_type,
            "host": host,
            "port": port
        })
        .to_string();

        crate::audit::log::write_audit_event(
            &db,
            "database_connection_created",
            "database_connection",
            &connection.id,
            &details,
        )
        .map_err(|e| format!("Failed to write audit log: {}", e))?;
    }

    Ok(connection)
}

/// Update an existing database connection
#[tauri::command]
pub async fn update_database_connection(
    id: String,
    name: Option<String>,
    password: Option<String>,
    ssl_enabled: Option<bool>,
    ssl_ca_cert_path: Option<String>,
    ssl_client_cert_path: Option<String>,
    ssl_client_key_path: Option<String>,
    connection_options: Option<String>,
    state: State<'_, AppState>,
) -> Result<DatabaseConnection, String> {
    // Encrypt new password if provided
    let encrypted_password = if let Some(pwd) = password {
        Some(
            crate::integrations::auth::encrypt_token(&pwd)
                .map_err(|e| format!("Failed to encrypt password: {}", e))?,
        )
    } else {
        None
    };

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Update database
    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        // Build dynamic UPDATE statement
        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(n) = &name {
            updates.push("name = ?");
            params.push(Box::new(n.clone()));
        }
        if let Some(p) = &encrypted_password {
            updates.push("encrypted_password = ?");
            params.push(Box::new(p.clone()));
        }
        if let Some(ssl) = ssl_enabled {
            updates.push("ssl_enabled = ?");
            params.push(Box::new(if ssl { 1 } else { 0 }));
        }
        if let Some(ca) = &ssl_ca_cert_path {
            updates.push("ssl_ca_cert_path = ?");
            params.push(Box::new(ca.clone()));
        }
        if let Some(cert) = &ssl_client_cert_path {
            updates.push("ssl_client_cert_path = ?");
            params.push(Box::new(cert.clone()));
        }
        if let Some(key) = &ssl_client_key_path {
            updates.push("ssl_client_key_path = ?");
            params.push(Box::new(key.clone()));
        }
        if let Some(opts) = &connection_options {
            updates.push("connection_options = ?");
            params.push(Box::new(opts.clone()));
        }

        if updates.is_empty() {
            return Err("No fields to update".to_string());
        }

        updates.push("updated_at = ?");
        params.push(Box::new(now.clone()));
        params.push(Box::new(id.clone()));

        let query = format!(
            "UPDATE database_connections SET {} WHERE id = ?",
            updates.join(", ")
        );

        db.execute(&query, rusqlite::params_from_iter(params.iter()))
            .map_err(|e| format!("Failed to update connection: {}", e))?;

        // Audit log (capture updates before dropping db lock)
        let updated_field_names: Vec<String> = updates
            .iter()
            .map(|u| u.split(" = ").next().unwrap_or("").to_string())
            .collect();
        let details = serde_json::json!({ "updated_fields": updated_field_names }).to_string();

        crate::audit::log::write_audit_event(
            &db,
            "database_connection_updated",
            "database_connection",
            &id,
            &details,
        )
        .map_err(|e| format!("Failed to write audit log: {}", e))?;
    }

    // Remove from pool if connected (force reconnection with new credentials)
    {
        let pool = state.db_pool_manager.lock().await;
        let _ = pool.remove_driver(&id).await;
    }

    // Load updated connection
    list_database_connections(state)
        .await?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| "Connection not found after update".to_string())
}

/// Delete a database connection
#[tauri::command]
pub async fn delete_database_connection(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Remove from database
    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        db.execute("DELETE FROM database_connections WHERE id = ?1", [&id])
            .map_err(|e| format!("Failed to delete connection: {}", e))?;
    }

    // Remove from pool
    {
        let pool = state.db_pool_manager.lock().await;
        let _ = pool.remove_driver(&id).await;
    }

    // Audit log
    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        crate::audit::log::write_audit_event(
            &db,
            "database_connection_deleted",
            "database_connection",
            &id,
            "{}",
        )
        .map_err(|e| format!("Failed to write audit log: {}", e))?;
    }

    Ok(())
}

/// List all database connections
#[tauri::command]
pub async fn list_database_connections(
    state: State<'_, AppState>,
) -> Result<Vec<DatabaseConnection>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let mut stmt = db
        .prepare(
            "SELECT id, name, db_type, host, port, database_name, username, ssl_enabled, ssl_ca_cert_path, ssl_client_cert_path, ssl_client_key_path, connection_options, created_at, updated_at
             FROM database_connections",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let connections = stmt
        .query_map([], |row| {
            Ok(DatabaseConnection {
                id: row.get(0)?,
                name: row.get(1)?,
                db_type: row.get(2)?,
                host: row.get(3)?,
                port: row.get::<_, i64>(4)? as u16,
                database_name: row.get(5)?,
                username: row.get(6)?,
                ssl_enabled: row.get::<_, i64>(7)? != 0,
                ssl_ca_cert_path: row.get(8)?,
                ssl_client_cert_path: row.get(9)?,
                ssl_client_key_path: row.get(10)?,
                connection_options: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(|e| format!("Failed to query connections: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect connections: {}", e))?;

    Ok(connections)
}

/// Test database connection
#[tauri::command]
pub async fn test_database_connection(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<ConnectionTestResult, String> {
    let start = std::time::Instant::now();

    // Load connection config
    let config = load_connection_config(&connection_id, &state).await?;

    // Create driver and test connection
    let mut driver = crate::db_drivers::create_driver(&config)
        .map_err(|e| format!("Failed to create driver: {}", e))?;

    match driver.connect(&config).await {
        Ok(_) => {
            let latency = start.elapsed().as_millis() as u64;
            driver
                .disconnect()
                .await
                .map_err(|e| format!("Failed to disconnect: {}", e))?;

            Ok(ConnectionTestResult {
                success: true,
                message: "Connection successful".to_string(),
                latency_ms: Some(latency),
            })
        }
        Err(e) => Ok(ConnectionTestResult {
            success: false,
            message: format!("Connection failed: {}", e),
            latency_ms: None,
        }),
    }
}

/// Execute a database query
#[tauri::command]
pub async fn execute_database_query(
    connection_id: String,
    query: String,
    page: usize,
    page_size: usize,
    state: State<'_, AppState>,
) -> Result<QueryExecutionResult, String> {
    let start = std::time::Instant::now();

    // Get or create driver
    let config = load_connection_config(&connection_id, &state).await?;
    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;

    let driver = driver_arc.read().await;

    // Execute query (with empty params - we don't support parameterized queries yet)
    let result = driver
        .execute_query(&query, Vec::new())
        .await
        .map_err(|e| format!("Query execution failed: {}", e))?;

    let execution_time = start.elapsed().as_millis() as u64;
    let row_count = result.rows.len();

    // Paginate results
    let offset = page * page_size;
    let paginated_rows: Vec<_> = result
        .rows
        .into_iter()
        .skip(offset)
        .take(page_size)
        .collect();

    let paginated_result = QueryResult {
        columns: result.columns,
        rows: paginated_rows,
        row_count,
        execution_time_ms: execution_time,
    };

    // Save to history
    save_query_history(&connection_id, &query, row_count, execution_time, "success", None, &state)
        .await?;

    Ok(QueryExecutionResult {
        query_result: paginated_result,
        execution_time_ms: execution_time,
        row_count,
    })
}

/// Get list of databases
#[tauri::command]
pub async fn get_databases(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let config = load_connection_config(&connection_id, &state).await?;
    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;

    let driver = driver_arc.read().await;
    driver
        .get_databases()
        .await
        .map_err(|e| format!("Failed to get databases: {}", e))
}

/// Get schema for a database
#[tauri::command]
pub async fn get_schema(
    connection_id: String,
    database: String,
    state: State<'_, AppState>,
) -> Result<crate::db_drivers::types::Schema, String> {
    let config = load_connection_config(&connection_id, &state).await?;
    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;

    let driver = driver_arc.read().await;
    driver
        .get_schema(&database)
        .await
        .map_err(|e| format!("Failed to get schema: {}", e))
}

/// Get tables in a database
#[tauri::command]
pub async fn get_tables(
    connection_id: String,
    database: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let config = load_connection_config(&connection_id, &state).await?;
    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;

    let driver = driver_arc.read().await;
    let schema = driver
        .get_schema(&database)
        .await
        .map_err(|e| format!("Failed to get schema: {}", e))?;

    Ok(schema.tables.iter().map(|t| t.name.clone()).collect())
}

/// Get table schema
#[tauri::command]
pub async fn get_table_schema(
    connection_id: String,
    database: String,
    table: String,
    state: State<'_, AppState>,
) -> Result<crate::db_drivers::types::Table, String> {
    let config = load_connection_config(&connection_id, &state).await?;
    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;

    let driver = driver_arc.read().await;
    let schema = driver
        .get_schema(&database)
        .await
        .map_err(|e| format!("Failed to get schema: {}", e))?;

    schema
        .tables
        .into_iter()
        .find(|t| t.name == table)
        .ok_or_else(|| format!("Table {} not found", table))
}

/// Begin transaction
#[tauri::command]
pub async fn begin_transaction(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = load_connection_config(&connection_id, &state).await?;
    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;

    let mut driver = driver_arc.write().await;
    let handle = driver
        .begin_transaction()
        .await
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    Ok(handle.id)
}

/// Commit transaction
#[tauri::command]
pub async fn commit_transaction(
    connection_id: String,
    transaction_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = load_connection_config(&connection_id, &state).await?;
    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;

    let mut driver = driver_arc.write().await;
    let handle = crate::db_drivers::types::TransactionHandle {
        id: transaction_id,
        active: true,
    };
    driver
        .commit_transaction(&handle)
        .await
        .map_err(|e| format!("Failed to commit transaction: {}", e))
}

/// Rollback transaction
#[tauri::command]
pub async fn rollback_transaction(
    connection_id: String,
    transaction_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = load_connection_config(&connection_id, &state).await?;
    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;

    let mut driver = driver_arc.write().await;
    let handle = crate::db_drivers::types::TransactionHandle {
        id: transaction_id,
        active: true,
    };
    driver
        .rollback_transaction(&handle)
        .await
        .map_err(|e| format!("Failed to rollback transaction: {}", e))
}

/// Get query history
#[tauri::command]
pub async fn get_query_history(
    connection_id: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<QueryHistory>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let limit_val = limit.unwrap_or(100);
    let mut stmt = db
        .prepare(
            "SELECT id, connection_id, query_text, row_count, execution_time_ms, status, error_message, executed_at
             FROM query_history
             WHERE connection_id = ?1
             ORDER BY executed_at DESC
             LIMIT ?2",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let history = stmt
        .query_map(rusqlite::params![connection_id, limit_val], |row| {
            Ok(QueryHistory {
                id: row.get(0)?,
                connection_id: row.get(1)?,
                query_text: row.get(2)?,
                row_count: row.get(3)?,
                execution_time_ms: row.get(4)?,
                status: row.get(5)?,
                error_message: row.get(6)?,
                executed_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query history: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect history: {}", e))?;

    Ok(history)
}

/// Search query history
#[tauri::command]
pub async fn search_query_history(
    search: String,
    connection_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<QueryHistory>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let search_pattern = format!("%{}%", search);

    let history: Vec<QueryHistory> = if let Some(conn_id) = connection_id {
        let mut stmt = db
            .prepare(
                "SELECT id, connection_id, query_text, row_count, execution_time_ms, status, error_message, executed_at
                 FROM query_history
                 WHERE connection_id = ?1 AND query_text LIKE ?2
                 ORDER BY executed_at DESC
                 LIMIT 100",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map(rusqlite::params![conn_id, search_pattern], |row| {
                Ok(QueryHistory {
                    id: row.get(0)?,
                    connection_id: row.get(1)?,
                    query_text: row.get(2)?,
                    row_count: row.get(3)?,
                    execution_time_ms: row.get(4)?,
                    status: row.get(5)?,
                    error_message: row.get(6)?,
                    executed_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query history: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect history: {}", e))?
    } else {
        let mut stmt = db
            .prepare(
                "SELECT id, connection_id, query_text, row_count, execution_time_ms, status, error_message, executed_at
                 FROM query_history
                 WHERE query_text LIKE ?1
                 ORDER BY executed_at DESC
                 LIMIT 100",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map(rusqlite::params![search_pattern], |row| {
                Ok(QueryHistory {
                    id: row.get(0)?,
                    connection_id: row.get(1)?,
                    query_text: row.get(2)?,
                    row_count: row.get(3)?,
                    execution_time_ms: row.get(4)?,
                    status: row.get(5)?,
                    error_message: row.get(6)?,
                    executed_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query history: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect history: {}", e))?
    };

    Ok(history)
}

/// Create query bookmark
#[tauri::command]
pub async fn create_query_bookmark(
    name: String,
    query_text: String,
    connection_id: Option<String>,
    tags: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<QueryBookmark, String> {
    let bookmark = QueryBookmark {
        id: Uuid::now_v7().to_string(),
        name: name.clone(),
        query_text: query_text.clone(),
        connection_id: connection_id.clone(),
        tags: tags.clone(),
        description: description.clone(),
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.execute(
        "INSERT INTO query_bookmarks (id, name, query_text, connection_id, tags, description, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            bookmark.id,
            bookmark.name,
            bookmark.query_text,
            bookmark.connection_id,
            bookmark.tags,
            bookmark.description,
            bookmark.created_at,
        ],
    )
    .map_err(|e| format!("Failed to create bookmark: {}", e))?;

    Ok(bookmark)
}

/// List query bookmarks
#[tauri::command]
pub async fn list_query_bookmarks(
    connection_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<QueryBookmark>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let bookmarks: Vec<QueryBookmark> = if let Some(conn_id) = connection_id {
        let mut stmt = db
            .prepare(
                "SELECT id, name, query_text, connection_id, tags, description, created_at
                 FROM query_bookmarks
                 WHERE connection_id = ?1 OR connection_id IS NULL
                 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([conn_id], |row| {
                Ok(QueryBookmark {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    query_text: row.get(2)?,
                    connection_id: row.get(3)?,
                    tags: row.get(4)?,
                    description: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query bookmarks: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect bookmarks: {}", e))?
    } else {
        let mut stmt = db
            .prepare(
                "SELECT id, name, query_text, connection_id, tags, description, created_at
                 FROM query_bookmarks
                 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(QueryBookmark {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    query_text: row.get(2)?,
                    connection_id: row.get(3)?,
                    tags: row.get(4)?,
                    description: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query bookmarks: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect bookmarks: {}", e))?
    };

    Ok(bookmarks)
}

/// Delete query bookmark
#[tauri::command]
pub async fn delete_query_bookmark(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.execute("DELETE FROM query_bookmarks WHERE id = ?1", [id])
        .map_err(|e| format!("Failed to delete bookmark: {}", e))?;

    Ok(())
}

// ─── Helper Functions ───────────────────────────────────────────────────────

/// Load connection config from database and decrypt password
async fn load_connection_config(
    connection_id: &str,
    state: &State<'_, AppState>,
) -> Result<ConnectionConfig, String> {
    let (db_type, host, port, database_name, username, encrypted_password, ssl_enabled, ssl_ca_cert_path, ssl_client_cert_path, ssl_client_key_path, connection_options) = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        let mut stmt = db
            .prepare(
                "SELECT db_type, host, port, database_name, username, encrypted_password, ssl_enabled, ssl_ca_cert_path, ssl_client_cert_path, ssl_client_key_path, connection_options
                 FROM database_connections
                 WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        stmt.query_row([connection_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        })
        .optional()
        .map_err(|e| format!("Failed to query connection: {}", e))?
        .ok_or_else(|| format!("Connection {} not found", connection_id))?
    };

    // Decrypt password
    let password = crate::integrations::auth::decrypt_token(&encrypted_password)
        .map_err(|e| format!("Failed to decrypt password: {}", e))?;

    // Parse database type
    let database_type = match db_type.as_str() {
        "postgresql" => DatabaseType::PostgreSQL,
        "mysql" => DatabaseType::MySQL,
        "mongodb" => DatabaseType::MongoDB,
        "redis" => DatabaseType::Redis,
        "cassandra" => DatabaseType::Cassandra,
        _ => return Err(format!("Unsupported database type: {}", db_type)),
    };

    // Parse SSL config
    let ssl_config = if ssl_enabled != 0 {
        Some(SslConfig {
            enabled: true,
            ca_cert_path: ssl_ca_cert_path,
            client_cert_path: ssl_client_cert_path,
            client_key_path: ssl_client_key_path,
            verify_server: true,
        })
    } else {
        None
    };

    // Parse connection options
    let mut options = HashMap::new();
    if let Some(opts_str) = connection_options {
        if let Ok(opts_json) = serde_json::from_str::<serde_json::Value>(&opts_str) {
            if let Some(obj) = opts_json.as_object() {
                for (key, value) in obj {
                    if let Some(val_str) = value.as_str() {
                        options.insert(key.clone(), val_str.to_string());
                    }
                }
            }
        }
    }

    Ok(ConnectionConfig {
        database_type,
        host,
        port: port as u16,
        database: database_name,
        username,
        password,
        ssl_config,
        options,
    })
}

/// Save query execution to history
async fn save_query_history(
    connection_id: &str,
    query_text: &str,
    row_count: usize,
    execution_time_ms: u64,
    status: &str,
    error_message: Option<String>,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let history_id = Uuid::now_v7().to_string();
    let executed_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    db.execute(
        "INSERT INTO query_history (id, connection_id, query_text, row_count, execution_time_ms, status, error_message, executed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            history_id,
            connection_id,
            query_text,
            row_count as i64,
            execution_time_ms as i64,
            status,
            error_message,
            executed_at,
        ],
    )
    .map_err(|e| format!("Failed to save query history: {}", e))?;

    Ok(())
}
