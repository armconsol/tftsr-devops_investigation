// JSON import/export implementation

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    pool::DatabasePoolManager,
    traits::DatabaseDriver,
    types::{DataValue, QueryResult},
};
use serde_json::{Map, Value as JsonValue};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::time::Instant;
use tauri::Emitter;

use super::{ImportOptions, ImportStats};

/// Import JSON data into a table
///
/// Supports two JSON formats:
/// 1. Array of objects: `[{"col1": "val1", "col2": "val2"}, ...]`
/// 2. Object with data array: `{"data": [{"col1": "val1"}, ...]}`
///
/// For MongoDB, supports nested documents
pub async fn import_json<R: tauri::Runtime>(
    file_path: &str,
    connection_id: &str,
    target_table: &str,
    options: ImportOptions,
    pool: &DatabasePoolManager,
    app_handle: Option<&tauri::AppHandle<R>>,
) -> DriverResult<ImportStats> {
    let start_time = Instant::now();

    // Get driver from pool
    let driver = pool.get_driver(connection_id).await?;

    // Read JSON file
    let file = File::open(file_path)
        .map_err(|e| DriverError::IoError(format!("Failed to open JSON file: {}", e)))?;

    let reader = BufReader::new(file);
    let json: JsonValue = serde_json::from_reader(reader)
        .map_err(|e| DriverError::ParseError(format!("Failed to parse JSON: {}", e)))?;

    // Extract array of records
    let records = match json {
        JsonValue::Array(arr) => arr,
        JsonValue::Object(obj) => {
            if let Some(JsonValue::Array(arr)) = obj.get("data") {
                arr.clone()
            } else {
                return Err(DriverError::ParseError(
                    "JSON must be array or object with 'data' array".to_string(),
                ));
            }
        }
        _ => {
            return Err(DriverError::ParseError(
                "JSON must be array or object".to_string(),
            ))
        }
    };

    let total_rows = records.len();
    let mut rows_processed = 0;
    let mut rows_inserted = 0;
    let mut rows_failed = 0;
    let mut errors = Vec::new();

    // Check if this is MongoDB (supports nested documents)
    let is_mongodb = {
        let driver_lock = driver.read().await;
        driver_lock.database_type().to_string().to_lowercase() == "mongodb"
    };

    // Note: Schema validation is skipped for simplicity
    // In production, we'd call get_schema() to validate columns
    let table_columns: std::collections::HashMap<String, crate::db_drivers::types::ColumnMetadata> =
        std::collections::HashMap::new();

    // Truncate table if requested
    if options.truncate_first {
        if is_mongodb {
            let delete_sql = format!("db.{}.deleteMany({{}})", target_table);
            let driver_lock = driver.read().await;
            (**driver_lock).execute_query(&delete_sql, vec![]).await?;
        } else {
            let truncate_sql = format!("TRUNCATE TABLE {}", target_table);
            let driver_lock = driver.read().await;
            (**driver_lock).execute_query(&truncate_sql, vec![]).await?;
        }
    }

    // Process records in batches
    let mut batch = Vec::new();

    for (idx, record) in records.iter().enumerate() {
        rows_processed += 1;

        if let JsonValue::Object(obj) = record {
            if is_mongodb {
                // MongoDB: insert document as-is
                batch.push(DataValue::Json(JsonValue::Object(obj.clone())));
            } else {
                // SQL: convert object to row values
                let mut row_values = Vec::new();
                let mut row_valid = true;

                // Apply column mappings or auto-detect
                let columns = if options.column_mappings.is_empty() {
                    obj.keys().cloned().collect::<Vec<_>>()
                } else {
                    options
                        .column_mappings
                        .iter()
                        .map(|m| m.target_column.clone())
                        .collect()
                };

                for col in &columns {
                    if let Some(value) = obj.get(col) {
                        let data_value = json_to_data_value(value);
                        row_values.push(data_value);
                    } else if table_columns.get(col).is_some_and(|c| c.nullable) {
                        row_values.push(DataValue::Null);
                    } else {
                        row_valid = false;
                        errors.push(format!(
                            "Missing required column '{}' at record {}",
                            col, idx
                        ));
                        break;
                    }
                }

                if row_valid {
                    batch.push(DataValue::Array(row_values));
                } else {
                    rows_failed += 1;
                    if !options.skip_errors {
                        return Err(DriverError::ValidationError(format!(
                            "Invalid record at index {}",
                            idx
                        )));
                    }
                }
            }

            // Insert batch when full
            if batch.len() >= options.batch_size {
                let driver_lock = driver.read().await;
                match insert_json_batch(&**driver_lock, target_table, &batch, is_mongodb).await {
                    Ok(count) => {
                        rows_inserted += count;
                        batch.clear();
                    }
                    Err(e) => {
                        rows_failed += batch.len();
                        if !options.skip_errors {
                            return Err(e);
                        }
                        errors.push(format!("Batch insert failed: {}", e));
                        batch.clear();
                    }
                }
                drop(driver_lock);

                // Emit progress
                if let Some(handle) = app_handle {
                    let progress = super::csv::ImportProgress {
                        rows_processed,
                        rows_inserted,
                        rows_failed,
                        percent_complete: (rows_processed as f64 / total_rows as f64) * 100.0,
                    };
                    let _ = handle.emit("import-progress", progress);
                }
            }
        } else {
            rows_failed += 1;
            if !options.skip_errors {
                return Err(DriverError::ValidationError(format!(
                    "Record at index {} is not an object",
                    idx
                )));
            }
            errors.push(format!("Record at index {} is not an object", idx));
        }
    }

    // Insert remaining batch
    if !batch.is_empty() {
        let driver_lock = driver.read().await;
        match insert_json_batch(&**driver_lock, target_table, &batch, is_mongodb).await {
            Ok(count) => rows_inserted += count,
            Err(e) => {
                rows_failed += batch.len();
                if !options.skip_errors {
                    return Err(e);
                }
                errors.push(format!("Final batch insert failed: {}", e));
            }
        }
    }

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(ImportStats {
        rows_processed,
        rows_inserted,
        rows_failed,
        errors,
        execution_time_ms,
    })
}

/// Convert JSON value to DataValue
fn json_to_data_value(value: &JsonValue) -> DataValue {
    match value {
        JsonValue::Null => DataValue::Null,
        JsonValue::Bool(b) => DataValue::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                DataValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                DataValue::Float(f)
            } else {
                DataValue::String(n.to_string())
            }
        }
        JsonValue::String(s) => DataValue::String(s.clone()),
        JsonValue::Array(arr) => DataValue::Array(arr.iter().map(json_to_data_value).collect()),
        JsonValue::Object(_) => DataValue::Json(value.clone()),
    }
}

/// Insert batch of JSON records
async fn insert_json_batch(
    driver: &dyn DatabaseDriver,
    table: &str,
    batch: &[DataValue],
    is_mongodb: bool,
) -> DriverResult<usize> {
    if batch.is_empty() {
        return Ok(0);
    }

    if is_mongodb {
        // MongoDB: use insertMany
        let docs: Vec<JsonValue> = batch
            .iter()
            .filter_map(|v| {
                if let DataValue::Json(json) = v {
                    Some(json.clone())
                } else {
                    None
                }
            })
            .collect();

        let docs_json = serde_json::to_string(&docs)
            .map_err(|e| DriverError::SerializationError(e.to_string()))?;

        let sql = format!("db.{}.insertMany({})", table, docs_json);
        driver.execute_query(&sql, vec![]).await?;
    } else {
        // SQL: build multi-row INSERT
        // Extract column count from first row
        let first_row = match &batch[0] {
            DataValue::Array(arr) => arr,
            _ => {
                return Err(DriverError::ValidationError(
                    "Invalid batch format".to_string(),
                ))
            }
        };

        let col_count = first_row.len();
        let mut all_values = Vec::new();
        let mut value_placeholders = Vec::new();

        for (row_idx, row_data) in batch.iter().enumerate() {
            if let DataValue::Array(row) = row_data {
                let offset = row_idx * col_count;
                let placeholders: Vec<String> = (1..=col_count)
                    .map(|i| format!("${}", offset + i))
                    .collect();
                value_placeholders.push(format!("({})", placeholders.join(", ")));
                all_values.extend(row.clone());
            }
        }

        // Note: Column names should be extracted from schema or provided
        // For now, assuming auto-generated columns
        let sql = format!(
            "INSERT INTO {} VALUES {}",
            table,
            value_placeholders.join(", ")
        );

        driver.execute_query(&sql, all_values).await?;
    }

    Ok(batch.len())
}

/// Export query results to JSON file
///
/// Exports as array of objects: `[{"col1": "val1", "col2": "val2"}, ...]`
pub fn export_json(query_result: &QueryResult, output_path: &str) -> DriverResult<()> {
    let file = File::create(output_path)
        .map_err(|e| DriverError::IoError(format!("Failed to create JSON file: {}", e)))?;

    let mut writer = BufWriter::new(file);

    // Convert rows to JSON array
    let mut json_rows = Vec::new();

    for row in &query_result.rows {
        let mut obj = Map::new();
        for (idx, value) in row.iter().enumerate() {
            if let Some(col) = query_result.columns.get(idx) {
                let json_value = data_value_to_json(value);
                obj.insert(col.name.clone(), json_value);
            }
        }
        json_rows.push(JsonValue::Object(obj));
    }

    let json = JsonValue::Array(json_rows);
    let json_string = serde_json::to_string_pretty(&json)
        .map_err(|e| DriverError::SerializationError(e.to_string()))?;

    writer
        .write_all(json_string.as_bytes())
        .map_err(|e| DriverError::IoError(format!("Failed to write JSON: {}", e)))?;

    writer
        .flush()
        .map_err(|e| DriverError::IoError(format!("Failed to flush JSON writer: {}", e)))?;

    Ok(())
}

/// Convert DataValue to JSON Value
fn data_value_to_json(value: &DataValue) -> JsonValue {
    match value {
        DataValue::Null => JsonValue::Null,
        DataValue::Boolean(b) => JsonValue::Bool(*b),
        DataValue::Integer(i) => JsonValue::Number((*i).into()),
        DataValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        DataValue::String(s) => JsonValue::String(s.clone()),
        DataValue::Bytes(b) => {
            use base64::Engine;
            JsonValue::String(base64::engine::general_purpose::STANDARD.encode(b))
        }
        DataValue::Date(d) => JsonValue::String(d.clone()),
        DataValue::DateTime(dt) => JsonValue::String(dt.clone()),
        DataValue::Json(j) => j.clone(),
        DataValue::Array(arr) => JsonValue::Array(arr.iter().map(data_value_to_json).collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_drivers::types::ColumnMetadata;

    #[test]
    fn test_export_json_basic() {
        let query_result = QueryResult {
            columns: vec![
                ColumnMetadata {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    primary_key: true,
                },
                ColumnMetadata {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    primary_key: false,
                },
            ],
            rows: vec![
                vec![
                    DataValue::Integer(1),
                    DataValue::String("Alice".to_string()),
                ],
                vec![DataValue::Integer(2), DataValue::String("Bob".to_string())],
            ],
            row_count: 2,
            execution_time_ms: 10,
        };

        let temp_file = std::env::temp_dir().join("test_export.json");
        let result = export_json(&query_result, temp_file.to_str().unwrap());
        assert!(result.is_ok());

        // Verify file exists and has valid JSON
        let content = std::fs::read_to_string(&temp_file).unwrap();
        let parsed: JsonValue = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_json_to_data_value_conversions() {
        assert_eq!(json_to_data_value(&JsonValue::Null), DataValue::Null);
        assert_eq!(
            json_to_data_value(&JsonValue::Bool(true)),
            DataValue::Boolean(true)
        );
        assert_eq!(
            json_to_data_value(&JsonValue::Number(42.into())),
            DataValue::Integer(42)
        );
        assert_eq!(
            json_to_data_value(&JsonValue::String("test".to_string())),
            DataValue::String("test".to_string())
        );
    }
}
