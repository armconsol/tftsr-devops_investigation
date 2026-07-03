// Table browser commands for viewing and modifying database table data
// Supports pagination, sorting, filtering, and CRUD operations

use crate::db_drivers::types::{DatabaseType, DataValue, ColumnMetadata};
use crate::state::AppState;
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
        .prepare(
            "SELECT db_type FROM database_connections WHERE id = ?1",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let db_type_str: String = stmt
        .query_row(rusqlite::params![&connection_id], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Connection not found: {}", e))?;

    let db_type = DatabaseType::parse(&db_type_str)
        .ok_or_else(|| format!("Unknown database type: {}", db_type_str))?;

    // Build SQL query
    let mut where_clauses = Vec::new();

    if let Some(filter_list) = &filters {
        for filter in filter_list {
            match filter.operator.as_str() {
                "=" => where_clauses.push(format!("`{}` = '{}'", filter.column, filter.value.replace('\'', "''"))),
                "!=" => where_clauses.push(format!("`{}` != '{}'", filter.column, filter.value.replace('\'', "''"))),
                ">" => where_clauses.push(format!("`{}` > '{}'", filter.column, filter.value.replace('\'', "''"))),
                "<" => where_clauses.push(format!("`{}` < '{}'", filter.column, filter.value.replace('\'', "''"))),
                ">=" => where_clauses.push(format!("`{}` >= '{}'", filter.column, filter.value.replace('\'', "''"))),
                "<=" => where_clauses.push(format!("`{}` <= '{}'", filter.column, filter.value.replace('\'', "''"))),
                "LIKE" => where_clauses.push(format!("`{}` LIKE '%{}%'", filter.column, filter.value.replace('\'', "''"))),
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
            format!("SELECT COUNT(*) as count FROM `{}`{}", table, where_clause)
        }
        DatabaseType::Redis => {
            // Redis: count keys
            format!("SELECT COUNT(*) as count FROM `{}`", table)
        }
        _ => return Err(format!("Unsupported database type for table browsing: {}", db_type)),
    };

    // Get total count
    let total_count: i64 = db
        .query_row(&count_query, [], |row| row.get(0))
        .unwrap_or(0);

    let total_pages = (total_count as usize + pagination.limit - 1) / pagination.limit;

    // Build main query with pagination
    let order_by = if let Some(sort_params) = &sort {
        let direction = if sort_params.direction.to_uppercase() == "DESC" {
            "DESC"
        } else {
            "ASC"
        };
        format!(" ORDER BY `{}` {}", sort_params.column, direction)
    } else {
        String::new()
    };

    let query = match db_type {
        DatabaseType::PostgreSQL | DatabaseType::MySQL => {
            format!(
                "SELECT * FROM `{}`{}{} LIMIT {} OFFSET {}",
                table, where_clause, order_by, pagination.limit, pagination.offset
            )
        }
        DatabaseType::Redis => {
            format!(
                "SELECT * FROM `{}` LIMIT {} OFFSET {}",
                table, pagination.limit, pagination.offset
            )
        }
        _ => return Err(format!("Unsupported database type for table browsing: {}", db_type)),
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

    let rows: Vec<TableRow> = stmt
        .query_map([], |row| {
            let mut row_map = HashMap::new();
            for (idx, col_name) in column_names.iter().enumerate() {
                let value: String = row.get::<_, String>(idx).unwrap_or_default();
                row_map.insert(col_name.clone(), DataValue::String(value));
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

    let query = format!("SELECT COUNT(*) as count FROM `{}`", table);
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

    // Get column information using SQLite PRAGMA
    let pragma_query = format!("PRAGMA table_info(`{}`)", table);
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
        .query_row(&format!("SELECT COUNT(*) FROM `{}`", table), [], |row| {
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

    let columns: Vec<String> = row_data.values.keys().cloned().collect();
    let placeholders = vec!["?"; columns.len()].join(",");

    let insert_query = format!(
        "INSERT INTO `{}` ({}) VALUES ({})",
        table,
        columns.iter().map(|c| format!("`{}`", c)).collect::<Vec<_>>().join(","),
        placeholders
    );

    // Build parameters
    let mut params: Vec<String> = Vec::new();
    for col in &columns {
        if let Some(val) = row_data.values.get(col) {
            params.push(format!("'{}'", match val {
                DataValue::String(s) => s.replace('\'', "''"),
                DataValue::Integer(i) => i.to_string(),
                DataValue::Float(f) => f.to_string(),
                DataValue::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
                DataValue::Null => "NULL".to_string(),
                _ => return Err(format!("Unsupported value type for column: {}", col)),
            }));
        }
    }

    let final_query = insert_query.replace("?", "{}");
    let formatted_query = if !params.is_empty() {
        format_args_into_string(&final_query, &params)
    } else {
        final_query
    };

    db.execute(&formatted_query, [])
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

    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let set_clauses: Vec<String> = row_data
        .values
        .keys()
        .enumerate()
        .map(|(i, col)| format!("`{}` = ?{}", col, i + 1))
        .collect();

    let update_query = format!(
        "UPDATE `{}` SET {} WHERE `{}` = ?{}",
        table,
        set_clauses.join(","),
        primary_key_col,
        row_data.values.len() + 1
    );

    // Build parameter string - simplified for now
    let mut params: Vec<String> = Vec::new();
    for col in row_data.values.keys() {
        if let Some(val) = row_data.values.get(col) {
            params.push(match val {
                DataValue::String(s) => format!("'{}'", s.replace('\'', "''")),
                DataValue::Integer(i) => i.to_string(),
                DataValue::Float(f) => f.to_string(),
                DataValue::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
                DataValue::Null => "NULL".to_string(),
                _ => return Err(format!("Unsupported value type for column: {}", col)),
            });
        }
    }

    params.push(match primary_key_value {
        DataValue::String(s) => format!("'{}'", s.replace('\'', "''")),
        DataValue::Integer(i) => i.to_string(),
        DataValue::Float(f) => f.to_string(),
        _ => return Err("Unsupported primary key value type".to_string()),
    });

    // Simple parameter replacement - in production use proper parameterized queries
    let mut final_query = update_query.clone();
    for (i, param) in params.iter().enumerate() {
        final_query = final_query.replacen(&format!("?{}", i + 1), param, 1);
    }

    db.execute(&final_query, [])
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
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let delete_query = format!("DELETE FROM `{}` WHERE `{}` = ?", table, primary_key_col);

    let pk_str = match primary_key_value {
        DataValue::String(s) => format!("'{}'", s.replace('\'', "''")),
        DataValue::Integer(i) => i.to_string(),
        DataValue::Float(f) => f.to_string(),
        _ => return Err("Unsupported primary key value type".to_string()),
    };

    let final_query = delete_query.replace("?", &pk_str);

    db.execute(&final_query, [])
        .map_err(|e| format!("Failed to delete row: {}", e))?;

    Ok(())
}

// Helper function for string formatting
fn format_args_into_string(template: &str, args: &[String]) -> String {
    let mut result = template.to_string();
    for (_i, arg) in args.iter().enumerate() {
        result = result.replacen("{}", arg, 1);
    }
    result
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
    fn test_format_args_into_string() {
        let template = "SELECT * FROM {} WHERE id = {}";
        let args = vec!["users".to_string(), "123".to_string()];
        let result = format_args_into_string(template, &args);
        assert_eq!(result, "SELECT * FROM users WHERE id = 123");
    }
}
