// MySQL schema introspection utilities

use super::driver::MySQLDriver;
use super::types::MySQLTypeConverter;
use crate::db_drivers::{
    error::{DriverError, DriverResult},
    traits::DatabaseDriver,
    types::{Column, DataValue, ForeignKey, Index, Schema, Table},
};

/// Type alias for foreign key tuple: (name, from_cols, to_table, to_cols, on_delete, on_update)
type ForeignKeyTuple = (String, Vec<String>, String, Vec<String>, String, String);

/// Handles schema introspection for MySQL databases
pub struct MySQLSchemaIntrospector<'a> {
    driver: &'a MySQLDriver,
}

impl<'a> MySQLSchemaIntrospector<'a> {
    pub fn new(driver: &'a MySQLDriver) -> Self {
        Self { driver }
    }

    /// Get complete schema for a database
    pub async fn get_schema(&self, database: &str) -> DriverResult<Schema> {
        // Get all tables in the database
        let table_names = self.get_table_names(database).await?;

        let mut tables = Vec::new();
        for table_name in table_names {
            match self.get_table_schema(database, &table_name).await {
                Ok(table) => tables.push(table),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to get schema for table {}: {}",
                        table_name, e
                    );
                    // Continue with other tables instead of failing completely
                }
            }
        }

        Ok(Schema {
            database_name: database.to_string(),
            tables,
        })
    }

    /// Get list of table names in the database
    async fn get_table_names(&self, database: &str) -> DriverResult<Vec<String>> {
        let database = validate_identifier(database)?;
        let query = format!(
            "SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_SCHEMA = '{}' AND TABLE_TYPE = 'BASE TABLE' ORDER BY TABLE_NAME",
            database
        );

        let result = self.driver.execute_query(&query, vec![]).await?;

        Ok(result
            .rows
            .into_iter()
            .filter_map(|row: Vec<DataValue>| {
                if let Some(DataValue::String(name)) = row.first() {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect())
    }

    /// Get complete schema for a single table
    async fn get_table_schema(&self, database: &str, table_name: &str) -> DriverResult<Table> {
        let columns = self.get_columns(database, table_name).await?;
        let indexes = self.get_indexes(database, table_name).await?;
        let foreign_keys = self.get_foreign_keys(database, table_name).await?;
        let row_count = self.get_row_count(database, table_name).await.ok();

        Ok(Table {
            name: table_name.to_string(),
            schema: Some(database.to_string()),
            columns,
            indexes,
            foreign_keys,
            row_count,
        })
    }

    /// Get column information for a table
    async fn get_columns(&self, database: &str, table_name: &str) -> DriverResult<Vec<Column>> {
        let database = validate_identifier(database)?;
        let table_name = validate_identifier(table_name)?;
        let query = format!(
            r#"
            SELECT
                COLUMN_NAME,
                COLUMN_TYPE,
                IS_NULLABLE,
                COLUMN_DEFAULT,
                COLUMN_KEY,
                EXTRA
            FROM information_schema.COLUMNS
            WHERE TABLE_SCHEMA = '{}'
              AND TABLE_NAME = '{}'
            ORDER BY ORDINAL_POSITION
            "#,
            database, table_name
        );

        let result = self
            .driver
            .execute_query(&query, vec![])
            .await
            .map_err(|e| {
                DriverError::SchemaIntrospectionFailed(format!(
                    "Failed to get columns for {}: {}",
                    table_name, e
                ))
            })?;

        let mut columns = Vec::new();
        for row in result.rows {
            if row.len() >= 6 {
                let column_name = match &row[0] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                let data_type = match &row[1] {
                    DataValue::String(s) => MySQLTypeConverter::type_name_to_string(s),
                    _ => "UNKNOWN".to_string(),
                };

                let nullable = match &row[2] {
                    DataValue::String(s) => s.eq_ignore_ascii_case("YES"),
                    _ => true,
                };

                let default_value = match &row[3] {
                    DataValue::String(s) => Some(s.clone()),
                    DataValue::Null => None,
                    _ => None,
                };

                let column_key = match &row[4] {
                    DataValue::String(s) => s.clone(),
                    _ => String::new(),
                };

                let primary_key = column_key.eq_ignore_ascii_case("PRI");

                let extra = match &row[5] {
                    DataValue::String(s) => s.clone(),
                    _ => String::new(),
                };

                let auto_increment = extra.to_lowercase().contains("auto_increment");

                columns.push(Column {
                    name: column_name,
                    data_type,
                    nullable,
                    default_value,
                    primary_key,
                    auto_increment,
                });
            }
        }

        Ok(columns)
    }

    /// Get index information for a table
    async fn get_indexes(&self, database: &str, table_name: &str) -> DriverResult<Vec<Index>> {
        let database = validate_identifier(database)?;
        let table_name = validate_identifier(table_name)?;
        let query = format!(
            r#"
            SELECT
                INDEX_NAME,
                COLUMN_NAME,
                NON_UNIQUE,
                INDEX_TYPE
            FROM information_schema.STATISTICS
            WHERE TABLE_SCHEMA = '{}'
              AND TABLE_NAME = '{}'
            ORDER BY INDEX_NAME, SEQ_IN_INDEX
            "#,
            database, table_name
        );

        let result = self
            .driver
            .execute_query(&query, vec![])
            .await
            .map_err(|e| {
                DriverError::SchemaIntrospectionFailed(format!(
                    "Failed to get indexes for {}: {}",
                    table_name, e
                ))
            })?;

        // Group columns by index name
        let mut index_map: std::collections::HashMap<String, (Vec<String>, bool, String)> =
            std::collections::HashMap::new();

        for row in result.rows {
            if row.len() >= 4 {
                let index_name = match &row[0] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                // Skip primary key index as it's represented in column metadata
                if index_name.eq_ignore_ascii_case("PRIMARY") {
                    continue;
                }

                let column_name = match &row[1] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                let non_unique = match &row[2] {
                    DataValue::Integer(i) => *i != 0,
                    DataValue::String(s) => !s.eq_ignore_ascii_case("0"),
                    _ => true,
                };

                let is_unique = !non_unique;

                let index_type = match &row[3] {
                    DataValue::String(s) => s.clone(),
                    _ => "BTREE".to_string(),
                };

                index_map
                    .entry(index_name)
                    .or_insert((Vec::new(), is_unique, index_type.clone()))
                    .0
                    .push(column_name);
            }
        }

        // Convert map to vector of indexes
        let mut indexes: Vec<Index> = index_map
            .into_iter()
            .map(|(name, (columns, unique, index_type))| Index {
                name,
                columns,
                unique,
                index_type,
            })
            .collect();

        indexes.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(indexes)
    }

    /// Get foreign key relationships for a table
    async fn get_foreign_keys(
        &self,
        database: &str,
        table_name: &str,
    ) -> DriverResult<Vec<ForeignKey>> {
        let database = validate_identifier(database)?;
        let table_name = validate_identifier(table_name)?;
        let query = format!(
            r#"
            SELECT
                kcu.CONSTRAINT_NAME,
                kcu.TABLE_NAME as FROM_TABLE,
                kcu.COLUMN_NAME as FROM_COLUMN,
                kcu.REFERENCED_TABLE_NAME as TO_TABLE,
                kcu.REFERENCED_COLUMN_NAME as TO_COLUMN,
                rc.UPDATE_RULE as ON_UPDATE,
                rc.DELETE_RULE as ON_DELETE
            FROM information_schema.KEY_COLUMN_USAGE kcu
            JOIN information_schema.REFERENTIAL_CONSTRAINTS rc
              ON kcu.CONSTRAINT_NAME = rc.CONSTRAINT_NAME
              AND kcu.CONSTRAINT_SCHEMA = rc.CONSTRAINT_SCHEMA
            WHERE kcu.TABLE_SCHEMA = '{}'
              AND kcu.TABLE_NAME = '{}'
              AND kcu.REFERENCED_TABLE_NAME IS NOT NULL
            ORDER BY kcu.CONSTRAINT_NAME, kcu.ORDINAL_POSITION
            "#,
            database, table_name
        );

        let result = self
            .driver
            .execute_query(&query, vec![])
            .await
            .map_err(|e| {
                DriverError::SchemaIntrospectionFailed(format!(
                    "Failed to get foreign keys for {}: {}",
                    table_name, e
                ))
            })?;

        // Group columns by constraint name
        let mut fk_map: std::collections::HashMap<String, ForeignKeyTuple> =
            std::collections::HashMap::new();

        for row in result.rows {
            if row.len() >= 7 {
                let constraint_name = match &row[0] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                let from_table = match &row[1] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                let from_column = match &row[2] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                let to_table = match &row[3] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                let to_column = match &row[4] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                let on_update = match &row[5] {
                    DataValue::String(s) => s.clone(),
                    _ => "NO ACTION".to_string(),
                };

                let on_delete = match &row[6] {
                    DataValue::String(s) => s.clone(),
                    _ => "NO ACTION".to_string(),
                };

                fk_map.entry(constraint_name).or_insert((
                    from_table,
                    Vec::new(),
                    to_table,
                    Vec::new(),
                    on_update,
                    on_delete,
                ));

                let entry = fk_map.get_mut(&row[0].to_string()).unwrap();
                entry.1.push(from_column);
                entry.3.push(to_column);
            }
        }

        // Convert map to vector of foreign keys
        let mut foreign_keys: Vec<ForeignKey> = fk_map
            .into_iter()
            .map(
                |(name, (from_table, from_columns, to_table, to_columns, on_update, on_delete))| {
                    ForeignKey {
                        name,
                        from_table,
                        from_columns,
                        to_table,
                        to_columns,
                        on_update,
                        on_delete,
                    }
                },
            )
            .collect();

        foreign_keys.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(foreign_keys)
    }

    /// Get exact row count for a table
    async fn get_row_count(&self, database: &str, table_name: &str) -> DriverResult<usize> {
        let database = validate_identifier(database)?;
        let table_name = validate_identifier(table_name)?;
        // Use information_schema for fast approximate count
        let query = format!(
            "SELECT TABLE_ROWS FROM information_schema.TABLES WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}'",
            database, table_name
        );

        let result = self
            .driver
            .execute_query(&query, vec![])
            .await
            .map_err(|e| {
                DriverError::SchemaIntrospectionFailed(format!(
                    "Failed to get row count for {}: {}",
                    table_name, e
                ))
            })?;

        if let Some(row) = result.rows.first() {
            if let Some(DataValue::Integer(count)) = row.first() {
                return Ok(*count as usize);
            }
        }

        // Fallback to exact count if information_schema query fails
        let fallback_query = format!("SELECT COUNT(*) FROM `{}`.`{}`", database, table_name);
        let result = self.driver.execute_query(&fallback_query, vec![]).await?;

        if let Some(row) = result.rows.first() {
            if let Some(DataValue::Integer(count)) = row.first() {
                return Ok(*count as usize);
            }
        }

        Ok(0)
    }
}

fn validate_identifier(value: &str) -> DriverResult<&str> {
    if value.is_empty() {
        return Err(DriverError::ValidationError(
            "Identifier cannot be empty".to_string(),
        ));
    }

    if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(DriverError::ValidationError(format!(
            "Invalid identifier: {}",
            value
        )));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_drivers::types::{ConnectionConfig, DatabaseType};
    use std::collections::HashMap;

    #[test]
    fn test_schema_introspector_creation() {
        let config = ConnectionConfig {
            database_type: DatabaseType::MySQL,
            host: "localhost".to_string(),
            port: 3306,
            database: Some("test".to_string()),
            username: "root".to_string(),
            password: "password".to_string(),
            ssl_config: None,
            options: HashMap::new(),
        };

        let driver = MySQLDriver::new(config);
        let introspector = MySQLSchemaIntrospector::new(&driver);

        // Just verify it can be constructed
        assert_eq!(
            std::mem::size_of_val(&introspector),
            std::mem::size_of::<&MySQLDriver>()
        );
    }
}
