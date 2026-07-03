// Table browser commands for viewing and modifying database table data
// Supports pagination, sorting, filtering, and CRUD operations

use crate::db_drivers::types::{ColumnMetadata, DataValue, DatabaseType};
use crate::state::AppState;
use rusqlite::{params_from_iter, types::Value, types::ValueRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

// ─── Types ──────────────────────────────────────────────────────────────────

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub limit: usize,
    pub offset: usize,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

/// Sorting parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortParams {
    pub column: String,
    pub direction: String, // "ASC" or "DESC"
}

/// Filter condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    pub column: String,
    pub operator: String, // "=", "!=", ">", "<", ">=", "<=", "LIKE"
    pub value: String,
}

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    pub table_name: String,
    pub row_count: i64,
    pub columns: Vec<ColumnMetadata>,
    pub primary_key: Option<String>,
    pub estimated_size_bytes: Option<i64>,
}

/// Row representation as a map of column name to value
pub type TableRow = HashMap<String, DataValue>;

/// Browse table data response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseTableResponse {
    pub rows: Vec<TableRow>,
    pub total_count: i64,
    pub page_number: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

/// Row insertion/update parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowData {
    pub values: HashMap<String, DataValue>,
}

// ─── Commands ───────────────────────────────────────────────────────────────

/// Browse table data with pagination and optional sorting/filtering
#[tauri::command]
pub async fn browse_table_data(
    connection_id: String,
    _database: String,
    table: String,
    pagination: Option<PaginationParams>,
    sort: Option<SortParams>,
    filters: Option<Vec<FilterCondition>>,
    state: State<'_, AppState>,
) -> Result<BrowseTableResponse, String> {
    let pagination = pagination.unwrap_or_default();

    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    // Load connection config
    let mut stmt = db
        .prepare("SELECT db_type FROM database_connections WHERE id = ?1")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let db_type_str: String = stmt
        .query_row(rusqlite::params![&connection_id], |row| row.get(0))
        .map_err(|e| format!("Connection not found: {}", e))?;

    let db_type = DatabaseType::parse(&db_type_str)
        .ok_or_else(|| format!("Unknown database type: {}", db_type_str))?;

    let table_ident = sanitize_identifier(&table)?;

    // Build SQL query
    let mut where_clauses = Vec::new();
    let mut filter_params: Vec<Value> = Vec::new();

    if let Some(filter_list) = &filters {
        for filter in filter_list {
            let col_ident = sanitize_identifier(&filter.column)?;
            match filter.operator.as_str() {
                "=" => {
                    where_clauses.push(format!("{col_ident} = ?"));
                    filter_params.push(Value::Text(filter.value.clone()));
                }
                "!=" => {
                    where_clauses.push(format!("{col_ident} != ?"));
                    filter_params.push(Value::Text(filter.value.clone()));
                }
                ">" => {
                    where_clauses.push(format!("{col_ident} > ?"));
                    filter_params.push(Value::Text(filter.value.clone()));
                }
                "<" => {
                    where_clauses.push(format!("{col_ident} < ?"));
                    filter_params.push(Value::Text(filter.value.clone()));
                }
                ">=" => {
                    where_clauses.push(format!("{col_ident} >= ?"));
                    filter_params.push(Value::Text(filter.value.clone()));
                }
                "<=" => {
                    where_clauses.push(format!("{col_ident} <= ?"));
                    filter_params.push(Value::Text(filter.value.clone()));
                }
                "LIKE" => {
                    where_clauses.push(format!("{col_ident} LIKE ?"));
                    filter_params.push(Value::Text(format!("%{}%", filter.value)));
                }
                _ => return Err(format!("Unsupported operator: {}", filter.operator)),
            }
        }
    }

    let where_clause = if !where_clauses.is_empty() {
        format!(" WHERE {}", where_clauses.join(" AND "))
    } else {
        String::new()
    };

    // Count total rows
    let count_query = match db_type {
        DatabaseType::PostgreSQL | DatabaseType::MySQL => {
            format!("SELECT COUNT(*) as count FROM {table_ident}{where_clause}")
        }
        DatabaseType::Redis => {
            // Redis: count keys
            format!("SELECT COUNT(*) as count FROM {table_ident}")
        }
        _ => {
            return Err(format!(
                "Unsupported database type for table browsing: {}",
                db_type
            ))
        }
    };

    // Get total count
    let total_count: i64 = db
        .query_row(
            &count_query,
            params_from_iter(filter_params.iter()),
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_pages = (total_count as usize).div_ceil(pagination.limit);

    // Build main query with pagination
    let order_by = if let Some(sort_params) = &sort {
        let sort_col = sanitize_identifier(&sort_params.column)?;
        let direction = if sort_params.direction.to_uppercase() == "DESC" {
            "DESC"
        } else {
            "ASC"
        };
        format!(" ORDER BY {sort_col} {direction}")
    } else {
        String::new()
    };

    let query = match db_type {
        DatabaseType::PostgreSQL | DatabaseType::MySQL => {
            format!("SELECT * FROM {table_ident}{where_clause}{order_by} LIMIT ? OFFSET ?")
        }
        DatabaseType::Redis => {
            format!("SELECT * FROM {table_ident} LIMIT ? OFFSET ?")
        }
        _ => {
            return Err(format!(
                "Unsupported database type for table browsing: {}",
                db_type
            ))
        }
    };

    // Execute query
    let mut stmt = db
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let column_names = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let query_params: Vec<Value> = filter_params
        .iter()
        .cloned()
        .chain([
            Value::Integer(pagination.limit as i64),
            Value::Integer(pagination.offset as i64),
        ])
        .collect();

    let rows: Vec<TableRow> = stmt
        .query_map(params_from_iter(query_params.iter()), |row| {
            let mut row_map = HashMap::new();
            for (idx, col_name) in column_names.iter().enumerate() {
                let value = match row.get_ref(idx)? {
                    ValueRef::Null => DataValue::Null,
                    ValueRef::Integer(i) => DataValue::Integer(i),
                    ValueRef::Real(f) => DataValue::Float(f),
                    ValueRef::Text(t) => DataValue::String(String::from_utf8_lossy(t).to_string()),
                    ValueRef::Blob(b) => DataValue::Bytes(b.to_vec()),
                };
                row_map.insert(col_name.clone(), value);
            }
            Ok(row_map)
        })
        .map_err(|e| format!("Failed to query table: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect rows: {}", e))?;

    let page_number = pagination.offset / pagination.limit;

    Ok(BrowseTableResponse {
        rows,
        total_count,
        page_number,
        page_size: pagination.limit,
        total_pages,
    })
}

/// Get total row count for a table
#[tauri::command]
pub async fn get_table_row_count(
    _connection_id: String,
    _database: String,
    table: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let table_ident = sanitize_identifier(&table)?;
    let query = format!("SELECT COUNT(*) as count FROM {table_ident}");
    db.query_row(&query, [], |row| row.get(0))
        .map_err(|e| format!("Failed to count rows: {}", e))
}

/// Get table metadata (columns, primary key, etc.)
#[tauri::command]
pub async fn get_table_metadata(
    _connection_id: String,
    _database: String,
    table: String,
    state: State<'_, AppState>,
) -> Result<TableMetadata, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let table_ident = sanitize_identifier(&table)?;
    // Get column information using SQLite PRAGMA
    let pragma_query = format!("PRAGMA table_info({table_ident})");
    let mut stmt = db
        .prepare(&pragma_query)
        .map_err(|e| format!("Failed to prepare pragma: {}", e))?;

    let columns: Vec<ColumnMetadata> = stmt
        .query_map([], |row| {
            Ok(ColumnMetadata {
                name: row.get(1)?,
                data_type: row.get(2)?,
                nullable: row.get::<_, i32>(3).map(|v| v == 0).unwrap_or(true),
                primary_key: row.get::<_, i32>(5).map(|v| v != 0).unwrap_or(false),
            })
        })
        .map_err(|e| format!("Failed to query columns: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect columns: {}", e))?;

    let row_count = db
        .query_row(&format!("SELECT COUNT(*) FROM {table_ident}"), [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    let primary_key = columns
        .iter()
        .find(|c| c.primary_key)
        .map(|c| c.name.clone());

    Ok(TableMetadata {
        table_name: table.clone(),
        row_count,
        columns,
        primary_key,
        estimated_size_bytes: None,
    })
}

/// Insert a new row into a table
#[tauri::command]
pub async fn insert_table_row(
    _connection_id: String,
    _database: String,
    table: String,
    row_data: RowData,
    state: State<'_, AppState>,
) -> Result<RowData, String> {
    if row_data.values.is_empty() {
        return Err("No values provided for insert".to_string());
    }

    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let table_ident = sanitize_identifier(&table)?;
    let columns: Vec<String> = row_data.values.keys().cloned().collect();
    let mut ordered_columns = columns;
    ordered_columns.sort();
    let placeholders = vec!["?"; ordered_columns.len()].join(",");
    let quoted_columns = ordered_columns
        .iter()
        .map(|c| sanitize_identifier(c))
        .collect::<Result<Vec<_>, _>>()?;

    let insert_query = format!(
        "INSERT INTO {table_ident} ({}) VALUES ({})",
        quoted_columns.join(","),
        placeholders
    );

    // Build parameters
    let mut params: Vec<Value> = Vec::new();
    for col in &ordered_columns {
        if let Some(val) = row_data.values.get(col) {
            params.push(data_value_to_sql_value(val)?);
        }
    }

    db.execute(&insert_query, params_from_iter(params.iter()))
        .map_err(|e| format!("Failed to insert row: {}", e))?;

    Ok(row_data)
}

/// Update an existing row in a table
#[tauri::command]
pub async fn update_table_row(
    _connection_id: String,
    _database: String,
    table: String,
    primary_key_col: String,
    primary_key_value: DataValue,
    row_data: RowData,
    state: State<'_, AppState>,
) -> Result<RowData, String> {
    if row_data.values.is_empty() {
        return Err("No values provided for update".to_string());
    }

    let table_ident = sanitize_identifier(&table)?;
    let pk_ident = sanitize_identifier(&primary_key_col)?;
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let mut columns: Vec<String> = row_data.values.keys().cloned().collect();
    columns.sort();
    let set_clauses: Vec<String> = columns
        .iter()
        .map(|col| sanitize_identifier(col).map(|ident| format!("{ident} = ?")))
        .collect::<Result<Vec<_>, _>>()?;

    if set_clauses.is_empty() {
        return Err("No valid columns provided for update".to_string());
    }

    let update_query = format!(
        "UPDATE {table_ident} SET {} WHERE {pk_ident} = ?",
        set_clauses.join(","),
    );

    let mut params: Vec<Value> = Vec::new();
    for col in columns {
        if let Some(val) = row_data.values.get(&col) {
            params.push(data_value_to_sql_value(val)?);
        }
    }

    params.push(data_value_to_sql_value(&primary_key_value)?);

    db.execute(&update_query, params_from_iter(params.iter()))
        .map_err(|e| format!("Failed to update row: {}", e))?;

    Ok(row_data)
}

/// Delete a row from a table
#[tauri::command]
pub async fn delete_table_row(
    _connection_id: String,
    _database: String,
    table: String,
    primary_key_col: String,
    primary_key_value: DataValue,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let table_ident = sanitize_identifier(&table)?;
    let pk_ident = sanitize_identifier(&primary_key_col)?;
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let delete_query = format!("DELETE FROM {table_ident} WHERE {pk_ident} = ?");
    let pk_value = data_value_to_sql_value(&primary_key_value)?;

    db.execute(&delete_query, params_from_iter([&pk_value]))
        .map_err(|e| format!("Failed to delete row: {}", e))?;

    Ok(())
}

fn sanitize_identifier(identifier: &str) -> Result<String, String> {
    if identifier.is_empty() {
        return Err("Identifier cannot be empty".to_string());
    }

    if !identifier
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(format!("Invalid identifier: {identifier}"));
    }

    Ok(format!("`{identifier}`"))
}

fn data_value_to_sql_value(value: &DataValue) -> Result<Value, String> {
    match value {
        DataValue::Null => Ok(Value::Null),
        DataValue::Boolean(v) => Ok(Value::Integer(if *v { 1 } else { 0 })),
        DataValue::Integer(v) => Ok(Value::Integer(*v)),
        DataValue::Float(v) => Ok(Value::Real(*v)),
        DataValue::String(v) => Ok(Value::Text(v.clone())),
        DataValue::Bytes(v) => Ok(Value::Blob(v.clone())),
        DataValue::Date(v) => Ok(Value::Text(v.clone())),
        DataValue::DateTime(v) => Ok(Value::Text(v.clone())),
        DataValue::Json(v) => Ok(Value::Text(v.to_string())),
        DataValue::Array(_) => Err("Array values are not supported for this operation".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pagination() {
        let pagination = PaginationParams::default();
        assert_eq!(pagination.limit, 50);
        assert_eq!(pagination.offset, 0);
    }

    #[test]
    fn test_filter_condition() {
        let filter = FilterCondition {
            column: "name".to_string(),
            operator: "=".to_string(),
            value: "test".to_string(),
        };
        assert_eq!(filter.column, "name");
        assert_eq!(filter.operator, "=");
        assert_eq!(filter.value, "test");
    }

    #[test]
    fn test_row_data_empty() {
        let row = RowData {
            values: HashMap::new(),
        };
        assert!(row.values.is_empty());
    }

    #[test]
    fn test_table_metadata_creation() {
        let metadata = TableMetadata {
            table_name: "users".to_string(),
            row_count: 100,
            columns: vec![],
            primary_key: Some("id".to_string()),
            estimated_size_bytes: None,
        };
        assert_eq!(metadata.table_name, "users");
        assert_eq!(metadata.row_count, 100);
        assert_eq!(metadata.primary_key, Some("id".to_string()));
    }

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("users").unwrap(), "`users`");
        assert!(sanitize_identifier("users-table").is_err());
    }

    #[test]
    fn test_data_value_to_sql_value() {
        let value = data_value_to_sql_value(&DataValue::Integer(42)).unwrap();
        assert_eq!(value, Value::Integer(42));
    }
}
