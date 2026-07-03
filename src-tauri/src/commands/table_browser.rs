// Table browser commands for viewing and modifying database table data.
// Uses the configured database driver (PostgreSQL/MySQL) instead of local SQLite metadata.

use crate::commands::database::load_connection_config;
use crate::db_drivers::types::{ColumnMetadata, DataValue, DatabaseType};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

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
    pub value: serde_json::Value,
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
    let table_ident = sanitize_identifier(&table)?;

    let config = load_connection_config(&connection_id, &state).await?;
    ensure_sql_table_browser_supported(config.database_type)?;

    let where_clause = build_where_clause(filters)?;
    let order_clause = build_order_clause(sort)?;

    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;
    let driver = driver_arc.read().await;

    let count_query = format!("SELECT COUNT(*) AS count FROM {table_ident}{where_clause}");
    let count_result = driver
        .execute_query(&count_query, Vec::new())
        .await
        .map_err(|e| format!("Failed to count rows: {}", e))?;
    let total_count = count_result
        .rows
        .first()
        .and_then(|r| r.first())
        .and_then(data_value_as_i64)
        .unwrap_or(0);
    let total_pages = if pagination.limit == 0 {
        0
    } else {
        (total_count as usize).div_ceil(pagination.limit)
    };
    let page_number = pagination.offset.checked_div(pagination.limit).unwrap_or(0);

    let data_query = format!(
        "SELECT * FROM {table_ident}{where_clause}{order_clause} LIMIT {} OFFSET {}",
        pagination.limit, pagination.offset
    );
    let result = driver
        .execute_query(&data_query, Vec::new())
        .await
        .map_err(|e| format!("Failed to query table data: {}", e))?;

    let rows = result
        .rows
        .into_iter()
        .map(|row_values| {
            let mut row = HashMap::new();
            for (idx, col) in result.columns.iter().enumerate() {
                if let Some(value) = row_values.get(idx) {
                    row.insert(col.name.clone(), value.clone());
                }
            }
            row
        })
        .collect::<Vec<_>>();

    Ok(BrowseTableResponse {
        rows,
        total_count,
        page_number,
        page_size: pagination.limit,
        total_pages,
    })
}

#[tauri::command]
pub async fn get_table_row_count(
    connection_id: String,
    _database: String,
    table: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let table_ident = sanitize_identifier(&table)?;
    let config = load_connection_config(&connection_id, &state).await?;
    ensure_sql_table_browser_supported(config.database_type)?;

    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;
    let driver = driver_arc.read().await;

    let query = format!("SELECT COUNT(*) AS count FROM {table_ident}");
    let result = driver
        .execute_query(&query, Vec::new())
        .await
        .map_err(|e| format!("Failed to count rows: {}", e))?;

    Ok(result
        .rows
        .first()
        .and_then(|r| r.first())
        .and_then(data_value_as_i64)
        .unwrap_or(0))
}

#[tauri::command]
pub async fn get_table_metadata(
    connection_id: String,
    database: String,
    table: String,
    state: State<'_, AppState>,
) -> Result<TableMetadata, String> {
    let config = load_connection_config(&connection_id, &state).await?;
    ensure_sql_table_browser_supported(config.database_type)?;

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
    let table_schema = schema
        .tables
        .iter()
        .find(|t| t.name == table)
        .ok_or_else(|| format!("Table '{}' not found in database '{}'", table, database))?;

    let columns = table_schema
        .columns
        .iter()
        .map(|c| ColumnMetadata {
            name: c.name.clone(),
            data_type: c.data_type.clone(),
            nullable: c.nullable,
            primary_key: c.primary_key,
        })
        .collect::<Vec<_>>();
    let primary_key = columns
        .iter()
        .find(|c| c.primary_key)
        .map(|c| c.name.clone());

    let row_count = if let Some(count) = table_schema.row_count {
        count as i64
    } else {
        let table_ident = sanitize_identifier(&table)?;
        let count_query = format!("SELECT COUNT(*) AS count FROM {table_ident}");
        let count_result = driver
            .execute_query(&count_query, Vec::new())
            .await
            .map_err(|e| format!("Failed to count rows: {}", e))?;
        count_result
            .rows
            .first()
            .and_then(|r| r.first())
            .and_then(data_value_as_i64)
            .unwrap_or(0)
    };

    Ok(TableMetadata {
        table_name: table,
        row_count,
        columns,
        primary_key,
        estimated_size_bytes: None,
    })
}

#[tauri::command]
pub async fn insert_table_row(
    connection_id: String,
    _database: String,
    table: String,
    row_data: RowData,
    state: State<'_, AppState>,
) -> Result<RowData, String> {
    if row_data.values.is_empty() {
        return Err("No values provided for insert".to_string());
    }

    let table_ident = sanitize_identifier(&table)?;
    let config = load_connection_config(&connection_id, &state).await?;
    ensure_sql_table_browser_supported(config.database_type)?;

    let mut columns = row_data.values.keys().cloned().collect::<Vec<_>>();
    columns.sort();
    let columns_ident = columns
        .iter()
        .map(|c| sanitize_identifier(c))
        .collect::<Result<Vec<_>, _>>()?;
    let values = columns
        .iter()
        .map(|c| {
            row_data
                .values
                .get(c)
                .ok_or_else(|| format!("Missing value for column '{}'", c))
                .and_then(data_value_to_sql_literal)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let query = format!(
        "INSERT INTO {table_ident} ({}) VALUES ({})",
        columns_ident.join(", "),
        values.join(", ")
    );

    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;
    let driver = driver_arc.read().await;
    driver
        .execute_query(&query, Vec::new())
        .await
        .map_err(|e| format!("Failed to insert row: {}", e))?;

    Ok(row_data)
}

#[tauri::command]
pub async fn update_table_row(
    connection_id: String,
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
    let config = load_connection_config(&connection_id, &state).await?;
    ensure_sql_table_browser_supported(config.database_type)?;

    let mut columns = row_data.values.keys().cloned().collect::<Vec<_>>();
    columns.sort();
    let set_clause = columns
        .iter()
        .map(|c| {
            let ident = sanitize_identifier(c)?;
            let literal = row_data
                .values
                .get(c)
                .ok_or_else(|| format!("Missing value for column '{}'", c))
                .and_then(data_value_to_sql_literal)?;
            Ok::<String, String>(format!("{ident} = {literal}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let pk_literal = data_value_to_sql_literal(&primary_key_value)?;

    let query = format!(
        "UPDATE {table_ident} SET {} WHERE {pk_ident} = {pk_literal}",
        set_clause.join(", ")
    );

    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;
    let driver = driver_arc.read().await;
    driver
        .execute_query(&query, Vec::new())
        .await
        .map_err(|e| format!("Failed to update row: {}", e))?;

    Ok(row_data)
}

#[tauri::command]
pub async fn delete_table_row(
    connection_id: String,
    _database: String,
    table: String,
    primary_key_col: String,
    primary_key_value: DataValue,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let table_ident = sanitize_identifier(&table)?;
    let pk_ident = sanitize_identifier(&primary_key_col)?;
    let pk_literal = data_value_to_sql_literal(&primary_key_value)?;

    let config = load_connection_config(&connection_id, &state).await?;
    ensure_sql_table_browser_supported(config.database_type)?;

    let exists_query =
        format!("SELECT COUNT(*) AS count FROM {table_ident} WHERE {pk_ident} = {pk_literal}");
    let pool = state.db_pool_manager.lock().await;
    let driver_arc = pool
        .get_or_create_driver(&connection_id, &config)
        .await
        .map_err(|e| format!("Failed to get driver: {}", e))?;
    let driver = driver_arc.read().await;
    let exists_result = driver
        .execute_query(&exists_query, Vec::new())
        .await
        .map_err(|e| format!("Failed to check row existence: {}", e))?;
    let exists = exists_result
        .rows
        .first()
        .and_then(|r| r.first())
        .and_then(data_value_as_i64)
        .unwrap_or(0)
        > 0;
    if !exists {
        return Err("Row not found for delete operation".to_string());
    }

    let query = format!("DELETE FROM {table_ident} WHERE {pk_ident} = {pk_literal}");

    driver
        .execute_query(&query, Vec::new())
        .await
        .map_err(|e| format!("Failed to delete row: {}", e))?;

    Ok(())
}

fn ensure_sql_table_browser_supported(database_type: DatabaseType) -> Result<(), String> {
    match database_type {
        DatabaseType::PostgreSQL | DatabaseType::MySQL => Ok(()),
        DatabaseType::Redis => {
            Err("Redis key browsing is not available in SQL table browser mode".to_string())
        }
        other => Err(format!(
            "Table browser is not supported for database type: {}",
            other
        )),
    }
}

fn build_where_clause(filters: Option<Vec<FilterCondition>>) -> Result<String, String> {
    let Some(filters) = filters else {
        return Ok(String::new());
    };
    if filters.is_empty() {
        return Ok(String::new());
    }

    let mut clauses = Vec::with_capacity(filters.len());
    for filter in filters {
        let column = sanitize_identifier(&filter.column)?;
        let operator = match filter.operator.as_str() {
            "=" | "!=" | ">" | "<" | ">=" | "<=" | "LIKE" => filter.operator,
            _ => return Err(format!("Unsupported filter operator: {}", filter.operator)),
        };

        let value_literal = filter_value_to_sql_literal(&filter.value, operator == "LIKE")?;

        clauses.push(format!("{column} {operator} {value_literal}"));
    }

    Ok(format!(" WHERE {}", clauses.join(" AND ")))
}

fn build_order_clause(sort: Option<SortParams>) -> Result<String, String> {
    let Some(sort) = sort else {
        return Ok(String::new());
    };

    let column = sanitize_identifier(&sort.column)?;
    let direction = match sort.direction.to_ascii_uppercase().as_str() {
        "ASC" => "ASC",
        "DESC" => "DESC",
        _ => return Err(format!("Unsupported sort direction: {}", sort.direction)),
    };

    Ok(format!(" ORDER BY {column} {direction}"))
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

    Ok(identifier.to_string())
}

fn data_value_as_i64(value: &DataValue) -> Option<i64> {
    match value {
        DataValue::Integer(v) => Some(*v),
        DataValue::Float(v) => Some(*v as i64),
        DataValue::String(v) => v.parse::<i64>().ok(),
        _ => None,
    }
}

fn data_value_to_sql_literal(value: &DataValue) -> Result<String, String> {
    match value {
        DataValue::Null => Ok("NULL".to_string()),
        DataValue::Boolean(v) => Ok(if *v { "TRUE" } else { "FALSE" }.to_string()),
        DataValue::Integer(v) => Ok(v.to_string()),
        DataValue::Float(v) => Ok(v.to_string()),
        DataValue::String(v) => Ok(format!("'{}'", v.replace('\'', "''"))),
        DataValue::Bytes(v) => {
            let hex = v
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join("");
            Ok(format!("X'{hex}'"))
        }
        DataValue::Date(v) | DataValue::DateTime(v) => Ok(format!("'{}'", v.replace('\'', "''"))),
        DataValue::Json(v) => Ok(format!("'{}'", v.to_string().replace('\'', "''"))),
        DataValue::Array(_) => Err("Array values are not supported for this operation".to_string()),
    }
}

fn filter_value_to_sql_literal(
    value: &serde_json::Value,
    for_like: bool,
) -> Result<String, String> {
    match value {
        serde_json::Value::String(v) => Ok(format!("'{}'", v.replace('\'', "''"))),
        serde_json::Value::Number(v) => Ok(v.to_string()),
        serde_json::Value::Bool(v) => Ok(if *v { "TRUE" } else { "FALSE" }.to_string()),
        serde_json::Value::Null => {
            if for_like {
                Err("LIKE filters do not support NULL values".to_string())
            } else {
                Ok("NULL".to_string())
            }
        }
        _ => Err("Filter values must be string, number, boolean, or null".to_string()),
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
            value: serde_json::Value::String("test".to_string()),
        };
        assert_eq!(filter.column, "name");
        assert_eq!(filter.operator, "=");
        assert_eq!(filter.value, serde_json::Value::String("test".to_string()));
    }

    #[test]
    fn test_row_data_empty() {
        let row = RowData {
            values: HashMap::new(),
        };
        assert!(row.values.is_empty());
    }

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("users").unwrap(), "users");
        assert!(sanitize_identifier("users-table").is_err());
    }

    #[test]
    fn test_build_where_clause_like_escapes_quotes() {
        let where_clause = build_where_clause(Some(vec![FilterCondition {
            column: "name".to_string(),
            operator: "LIKE".to_string(),
            value: serde_json::Value::String("o'hara%".to_string()),
        }]))
        .unwrap();
        assert_eq!(where_clause, " WHERE name LIKE 'o''hara%'");
    }

    #[test]
    fn test_build_order_clause_validates_direction() {
        let order_clause = build_order_clause(Some(SortParams {
            column: "created_at".to_string(),
            direction: "DESC".to_string(),
        }))
        .unwrap();
        assert_eq!(order_clause, " ORDER BY created_at DESC");
        assert!(build_order_clause(Some(SortParams {
            column: "created_at".to_string(),
            direction: "INVALID".to_string(),
        }))
        .is_err());
    }
}
