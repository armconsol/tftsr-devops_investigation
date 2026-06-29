// SQL export implementation

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    types::{DataValue, QueryResult},
};
use std::fs::File;
use std::io::{BufWriter, Write};

/// Export query results as SQL INSERT statements
///
/// Generates batched INSERT statements with proper SQL escaping
/// for easy re-import into the same or different database.
///
/// # Arguments
/// * `query_result` - Query result to export
/// * `table_name` - Target table name for INSERT statements
/// * `output_path` - Output file path
///
/// # Returns
/// () on success
pub fn export_sql_inserts(
    query_result: &QueryResult,
    table_name: &str,
    output_path: &str,
) -> DriverResult<()> {
    let file = File::create(output_path)
        .map_err(|e| DriverError::Other(format!("Failed to create SQL file: {}", e)))?;

    let mut writer = BufWriter::new(file);

    // Write header comment
    writeln!(
        writer,
        "-- SQL Export\n-- Table: {}\n-- Rows: {}\n",
        table_name, query_result.row_count
    )
    .map_err(|e| DriverError::Other(format!("Failed to write SQL header: {}", e)))?;

    // Get column names
    let column_names: Vec<String> = query_result
        .columns
        .iter()
        .map(|c| c.name.clone())
        .collect();

    // Process rows in batches of 1000
    const BATCH_SIZE: usize = 1000;
    let mut batch_values = Vec::new();

    for (idx, row) in query_result.rows.iter().enumerate() {
        // Convert row values to SQL strings
        let value_strings: Vec<String> = row.iter().map(data_value_to_sql).collect();
        batch_values.push(format!("({})", value_strings.join(", ")));

        // Write batch when full or at end
        if batch_values.len() >= BATCH_SIZE || idx == query_result.rows.len() - 1 {
            let insert_stmt = format!(
                "INSERT INTO {} ({}) VALUES\n{};",
                table_name,
                column_names.join(", "),
                batch_values.join(",\n")
            );

            writeln!(writer, "{}\n", insert_stmt)
                .map_err(|e| DriverError::Other(format!("Failed to write SQL batch: {}", e)))?;

            batch_values.clear();
        }
    }

    writer
        .flush()
        .map_err(|e| DriverError::Other(format!("Failed to flush SQL writer: {}", e)))?;

    Ok(())
}

/// Convert DataValue to SQL literal string with proper escaping
fn data_value_to_sql(value: &DataValue) -> String {
    match value {
        DataValue::Null => "NULL".to_string(),
        DataValue::Boolean(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        DataValue::Integer(i) => i.to_string(),
        DataValue::Float(f) => {
            if f.is_nan() {
                "'NaN'".to_string()
            } else if f.is_infinite() {
                if *f > 0.0 {
                    "'Infinity'".to_string()
                } else {
                    "'-Infinity'".to_string()
                }
            } else {
                f.to_string()
            }
        }
        DataValue::String(s) => format!("'{}'", escape_sql_string(s)),
        DataValue::Bytes(b) => {
            // Export as hex string
            format!("'\\x{}'", hex::encode(b))
        }
        DataValue::Date(d) => format!("'{}'", escape_sql_string(d)),
        DataValue::DateTime(dt) => format!("'{}'", escape_sql_string(dt)),
        DataValue::Json(j) => {
            // Export JSON as escaped string
            let json_str = serde_json::to_string(j).unwrap_or_else(|_| "{}".to_string());
            format!("'{}'", escape_sql_string(&json_str))
        }
        DataValue::Array(arr) => {
            // PostgreSQL array syntax
            let elements: Vec<String> = arr.iter().map(data_value_to_sql).collect();
            format!("ARRAY[{}]", elements.join(", "))
        }
    }
}

/// Escape single quotes and backslashes in SQL strings
fn escape_sql_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "''")
}

/// Export query results as CREATE TABLE + INSERT statements
///
/// Includes table schema definition for complete database export
pub fn export_sql_with_schema(
    query_result: &QueryResult,
    table_name: &str,
    output_path: &str,
) -> DriverResult<()> {
    let file = File::create(output_path)
        .map_err(|e| DriverError::Other(format!("Failed to create SQL file: {}", e)))?;

    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(
        writer,
        "-- SQL Export with Schema\n-- Table: {}\n-- Rows: {}\n",
        table_name, query_result.row_count
    )
    .map_err(|e| DriverError::Other(format!("Failed to write SQL header: {}", e)))?;

    // Generate CREATE TABLE statement
    writeln!(writer, "DROP TABLE IF EXISTS {};", table_name)
        .map_err(|e| DriverError::Other(format!("Failed to write DROP TABLE: {}", e)))?;

    let mut column_defs = Vec::new();
    for col in &query_result.columns {
        let mut def = format!("{} {}", col.name, col.data_type);
        if !col.nullable {
            def.push_str(" NOT NULL");
        }
        if col.primary_key {
            def.push_str(" PRIMARY KEY");
        }
        column_defs.push(def);
    }

    writeln!(
        writer,
        "CREATE TABLE {} (\n  {}\n);\n",
        table_name,
        column_defs.join(",\n  ")
    )
    .map_err(|e| DriverError::Other(format!("Failed to write CREATE TABLE: {}", e)))?;

    writer
        .flush()
        .map_err(|e| DriverError::Other(format!("Failed to flush after schema: {}", e)))?;

    // Now write INSERTs
    export_sql_inserts(query_result, table_name, output_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_drivers::types::ColumnMetadata;

    #[test]
    fn test_escape_sql_string() {
        assert_eq!(escape_sql_string("hello"), "hello");
        assert_eq!(escape_sql_string("it's"), "it''s");
        assert_eq!(escape_sql_string("path\\to\\file"), "path\\\\to\\\\file");
        assert_eq!(escape_sql_string("it's a\\test"), "it''s a\\\\test");
    }

    #[test]
    fn test_data_value_to_sql() {
        assert_eq!(data_value_to_sql(&DataValue::Null), "NULL");
        assert_eq!(data_value_to_sql(&DataValue::Boolean(true)), "TRUE");
        assert_eq!(data_value_to_sql(&DataValue::Integer(42)), "42");
        assert_eq!(data_value_to_sql(&DataValue::Float(3.14)), "3.14");
        assert_eq!(
            data_value_to_sql(&DataValue::String("test".to_string())),
            "'test'"
        );
        assert_eq!(
            data_value_to_sql(&DataValue::String("it's".to_string())),
            "'it''s'"
        );
    }

    #[test]
    fn test_export_sql_inserts() {
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

        let temp_file = std::env::temp_dir().join("test_export.sql");
        let result = export_sql_inserts(&query_result, "users", temp_file.to_str().unwrap());
        assert!(result.is_ok());

        // Verify file content
        let content = std::fs::read_to_string(&temp_file).unwrap();
        assert!(content.contains("INSERT INTO users"));
        assert!(content.contains("id, name"));
        assert!(content.contains("(1, 'Alice')"));
        assert!(content.contains("(2, 'Bob')"));

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }
}
