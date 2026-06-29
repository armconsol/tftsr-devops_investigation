// Database management commands for import/export and visualization

use crate::db_drivers::{
    import_export::{csv, json, sql, ImportOptions, ImportStats},
    types::QueryResult,
    visualization,
};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{AppHandle, State};

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
