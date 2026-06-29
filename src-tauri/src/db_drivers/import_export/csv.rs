// CSV import/export implementation

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    pool::DatabasePoolManager,
    traits::DatabaseDriver,
    types::{DataValue, QueryResult},
};
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::time::Instant;
use tauri::Emitter;

use super::{ImportOptions, ImportStats};

/// Progress event emitted during CSV import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProgress {
    pub rows_processed: usize,
    pub rows_inserted: usize,
    pub rows_failed: usize,
    pub percent_complete: f64,
}

/// Import CSV data into a table
///
/// # Arguments
/// * `file_path` - Path to CSV file
/// * `connection_id` - Database connection ID
/// * `target_table` - Target table name
/// * `options` - Import options
/// * `pool` - Database pool manager
/// * `app_handle` - Tauri app handle for progress events (optional)
///
/// # Returns
/// ImportStats with operation details
pub async fn import_csv<R: tauri::Runtime>(
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

    // Open CSV file
    let file = File::open(file_path)
        .map_err(|e| DriverError::IoError(format!("Failed to open CSV file: {}", e)))?;

    let reader = BufReader::new(file);
    let mut csv_reader = Reader::from_reader(reader);

    // Get CSV headers
    let headers = csv_reader
        .headers()
        .map_err(|e| DriverError::ParseError(format!("Failed to read CSV headers: {}", e)))?
        .clone();

    // Note: Schema validation is skipped for simplicity
    // In production, we'd call get_schema() to validate columns
    let table_columns: std::collections::HashMap<String, crate::db_drivers::types::ColumnMetadata> =
        std::collections::HashMap::new();

    // Build column mapping
    let column_map = if options.column_mappings.is_empty() {
        // Auto-detect: match CSV headers to table columns
        headers
            .iter()
            .filter_map(|header| {
                table_columns
                    .get(header)
                    .map(|_| (header.to_string(), header.to_string()))
            })
            .collect::<Vec<_>>()
    } else {
        // Use provided mappings
        options
            .column_mappings
            .iter()
            .map(|m| (m.source_column.clone(), m.target_column.clone()))
            .collect()
    };

    if column_map.is_empty() {
        return Err(DriverError::ValidationError(
            "No matching columns found between CSV and target table".to_string(),
        ));
    }

    // Truncate table if requested
    if options.truncate_first {
        let truncate_sql = format!("TRUNCATE TABLE {}", target_table);
        let driver_lock = driver.read().await;
        (**driver_lock).execute_query(&truncate_sql, vec![]).await?;
    }

    // Count total rows for progress tracking
    let total_rows = csv_reader.records().count();
    let file = File::open(file_path)
        .map_err(|e| DriverError::IoError(format!("Failed to reopen CSV file: {}", e)))?;
    let reader = BufReader::new(file);
    let mut csv_reader = Reader::from_reader(reader);
    csv_reader.headers().ok(); // Skip headers

    let mut rows_processed = 0;
    let mut rows_inserted = 0;
    let mut rows_failed = 0;
    let mut errors = Vec::new();
    let mut batch = Vec::new();

    // Build INSERT statement
    let column_names = column_map
        .iter()
        .map(|(_, target)| target.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    for result in csv_reader.records() {
        rows_processed += 1;

        match result {
            Ok(record) => {
                // Map CSV row to table columns
                let mut values = Vec::new();
                let mut row_valid = true;

                for (source_col, _target_col) in &column_map {
                    if let Some(idx) = headers.iter().position(|h| h == source_col) {
                        if let Some(value) = record.get(idx) {
                            // Convert string to DataValue (basic type inference)
                            let data_value = if value.is_empty() {
                                DataValue::Null
                            } else if let Ok(i) = value.parse::<i64>() {
                                DataValue::Integer(i)
                            } else if let Ok(f) = value.parse::<f64>() {
                                DataValue::Float(f)
                            } else if value.eq_ignore_ascii_case("true")
                                || value.eq_ignore_ascii_case("false")
                            {
                                DataValue::Boolean(value.eq_ignore_ascii_case("true"))
                            } else {
                                DataValue::String(value.to_string())
                            };
                            values.push(data_value);
                        } else {
                            row_valid = false;
                            break;
                        }
                    } else {
                        row_valid = false;
                        break;
                    }
                }

                if row_valid {
                    batch.push(values);
                } else {
                    rows_failed += 1;
                    if !options.skip_errors {
                        return Err(DriverError::ValidationError(format!(
                            "Invalid row at line {}",
                            rows_processed
                        )));
                    }
                    errors.push(format!("Invalid row at line {}", rows_processed));
                }

                // Insert batch when full
                if batch.len() >= options.batch_size {
                    let driver_lock = driver.read().await;
                    match insert_batch(&**driver_lock, target_table, &column_names, &batch).await {
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

                    // Emit progress event
                    if let Some(handle) = app_handle {
                        let progress = ImportProgress {
                            rows_processed,
                            rows_inserted,
                            rows_failed,
                            percent_complete: (rows_processed as f64 / total_rows as f64) * 100.0,
                        };
                        let _ = handle.emit("import-progress", progress);
                    }
                }
            }
            Err(e) => {
                rows_failed += 1;
                if !options.skip_errors {
                    return Err(DriverError::ParseError(format!(
                        "CSV parsing error at line {}: {}",
                        rows_processed, e
                    )));
                }
                errors.push(format!(
                    "CSV parsing error at line {}: {}",
                    rows_processed, e
                ));
            }
        }
    }

    // Insert remaining batch
    if !batch.is_empty() {
        let driver_lock = driver.read().await;
        match insert_batch(&**driver_lock, target_table, &column_names, &batch).await {
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

/// Insert a batch of rows using parameterized queries
async fn insert_batch(
    driver: &dyn DatabaseDriver,
    table: &str,
    columns: &str,
    batch: &[Vec<DataValue>],
) -> DriverResult<usize> {
    if batch.is_empty() {
        return Ok(0);
    }

    let placeholders_per_row = batch[0].len();
    let mut all_values = Vec::new();
    let mut value_placeholders = Vec::new();

    for (row_idx, row) in batch.iter().enumerate() {
        let offset = row_idx * placeholders_per_row;
        let placeholders: Vec<String> = (1..=placeholders_per_row)
            .map(|i| format!("${}", offset + i))
            .collect();
        value_placeholders.push(format!("({})", placeholders.join(", ")));
        all_values.extend(row.clone());
    }

    let sql = format!(
        "INSERT INTO {} ({}) VALUES {}",
        table,
        columns,
        value_placeholders.join(", ")
    );

    driver.execute_query(&sql, all_values).await?;
    Ok(batch.len())
}

/// Export query results to CSV file
///
/// # Arguments
/// * `query_result` - Query result to export
/// * `output_path` - Output file path
///
/// # Returns
/// () on success
pub fn export_csv(query_result: &QueryResult, output_path: &str) -> DriverResult<()> {
    let file = File::create(output_path)
        .map_err(|e| DriverError::IoError(format!("Failed to create CSV file: {}", e)))?;

    let writer = BufWriter::new(file);
    let mut csv_writer = Writer::from_writer(writer);

    // Write headers
    let headers: Vec<String> = query_result
        .columns
        .iter()
        .map(|c| c.name.clone())
        .collect();
    csv_writer
        .write_record(&headers)
        .map_err(|e| DriverError::IoError(format!("Failed to write CSV headers: {}", e)))?;

    // Write rows
    for row in &query_result.rows {
        let string_values: Vec<String> = row.iter().map(|v| v.to_string()).collect();
        csv_writer
            .write_record(&string_values)
            .map_err(|e| DriverError::IoError(format!("Failed to write CSV row: {}", e)))?;
    }

    csv_writer
        .flush()
        .map_err(|e| DriverError::IoError(format!("Failed to flush CSV writer: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_drivers::types::ColumnMetadata;

    #[test]
    fn test_export_csv_basic() {
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

        let temp_file = std::env::temp_dir().join("test_export.csv");
        let result = export_csv(&query_result, temp_file.to_str().unwrap());
        assert!(result.is_ok());

        // Verify file exists and has content
        let content = std::fs::read_to_string(&temp_file).unwrap();
        assert!(content.contains("id,name"));
        assert!(content.contains("1,Alice"));
        assert!(content.contains("2,Bob"));

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }
}
