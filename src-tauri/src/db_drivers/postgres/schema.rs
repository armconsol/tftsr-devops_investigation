// PostgreSQL schema introspection utilities

use super::driver::PostgresDriver;
use super::types::PostgresTypeConverter;
use crate::db_drivers::{
    error::{DriverError, DriverResult},
    traits::DatabaseDriver,
    types::{Column, DataValue, ForeignKey, Index, Schema, Table},
};

/// Handles schema introspection for PostgreSQL databases
pub struct PostgresSchemaIntrospector<'a> {
    driver: &'a PostgresDriver,
}

impl<'a> PostgresSchemaIntrospector<'a> {
    pub fn new(driver: &'a PostgresDriver) -> Self {
        Self { driver }
    }

    /// Get complete schema for a database
    pub async fn get_schema(&self, database: &str) -> DriverResult<Schema> {
        // Get all tables in the public schema
        let table_names = self.get_table_names().await?;

        let mut tables = Vec::new();
        for table_name in table_names {
            match self.get_table_schema(&table_name).await {
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

    /// Get list of table names in the public schema
    async fn get_table_names(&self) -> DriverResult<Vec<String>> {
        let query = r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_type = 'BASE TABLE'
            ORDER BY table_name
        "#;

        let result = self.driver.execute_query(query, vec![]).await?;

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
    async fn get_table_schema(&self, table_name: &str) -> DriverResult<Table> {
        let columns = self.get_columns(table_name).await?;
        let indexes = self.get_indexes(table_name).await?;
        let foreign_keys = self.get_foreign_keys(table_name).await?;
        let row_count = self.get_row_count(table_name).await.ok();

        Ok(Table {
            name: table_name.to_string(),
            schema: Some("public".to_string()),
            columns,
            indexes,
            foreign_keys,
            row_count,
        })
    }

    /// Get column information for a table
    async fn get_columns(&self, table_name: &str) -> DriverResult<Vec<Column>> {
        let table_name = validate_identifier(table_name)?;
        let query = r#"
            SELECT
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_default,
                COALESCE(
                    (SELECT true
                     FROM information_schema.table_constraints tc
                     JOIN information_schema.key_column_usage kcu
                       ON tc.constraint_name = kcu.constraint_name
                     WHERE tc.table_schema = 'public'
                       AND tc.table_name = c.table_name
                       AND kcu.column_name = c.column_name
                       AND tc.constraint_type = 'PRIMARY KEY'
                     LIMIT 1),
                    false
                ) as is_primary_key,
                COALESCE(
                    c.column_default LIKE 'nextval(%',
                    false
                ) as is_auto_increment
            FROM information_schema.columns c
            WHERE c.table_schema = 'public'
              AND c.table_name = $1
            ORDER BY c.ordinal_position
        "#;

        let query_with_param = query.replace("$1", &format!("'{}'", table_name));

        let result = self
            .driver
            .execute_query(&query_with_param, vec![])
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
                    DataValue::String(s) => PostgresTypeConverter::type_name_to_string(s),
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

                let primary_key = match &row[4] {
                    DataValue::Boolean(b) => *b,
                    DataValue::String(s) => s.eq_ignore_ascii_case("true") || s == "t",
                    _ => false,
                };

                let auto_increment = match &row[5] {
                    DataValue::Boolean(b) => *b,
                    DataValue::String(s) => s.eq_ignore_ascii_case("true") || s == "t",
                    _ => false,
                };

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
    async fn get_indexes(&self, table_name: &str) -> DriverResult<Vec<Index>> {
        let table_name = validate_identifier(table_name)?;
        let query = r#"
            SELECT
                i.indexname as index_name,
                i.indexdef as index_definition,
                ix.indisunique as is_unique
            FROM pg_indexes i
            JOIN pg_class c ON c.relname = i.tablename
            JOIN pg_index ix ON ix.indexrelid = (
                SELECT oid FROM pg_class WHERE relname = i.indexname
            )
            WHERE i.schemaname = 'public'
              AND i.tablename = $1
            ORDER BY i.indexname
        "#;

        let query_with_param = query.replace("$1", &format!("'{}'", table_name));

        let result = self
            .driver
            .execute_query(&query_with_param, vec![])
            .await
            .map_err(|e| {
                DriverError::SchemaIntrospectionFailed(format!(
                    "Failed to get indexes for {}: {}",
                    table_name, e
                ))
            })?;

        let mut indexes = Vec::new();
        for row in result.rows {
            if row.len() >= 3 {
                let index_name = match &row[0] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                let index_def = match &row[1] {
                    DataValue::String(s) => s.clone(),
                    _ => continue,
                };

                let is_unique = match &row[2] {
                    DataValue::Boolean(b) => *b,
                    DataValue::String(s) => s.eq_ignore_ascii_case("true") || s == "t",
                    _ => false,
                };

                // Extract column names from index definition
                let columns = Self::parse_index_columns(&index_def);

                // Determine index type from definition
                let index_type = if index_def.contains("USING btree") {
                    "BTREE"
                } else if index_def.contains("USING hash") {
                    "HASH"
                } else if index_def.contains("USING gin") {
                    "GIN"
                } else if index_def.contains("USING gist") {
                    "GIST"
                } else if index_def.contains("USING brin") {
                    "BRIN"
                } else {
                    "BTREE" // Default
                };

                indexes.push(Index {
                    name: index_name,
                    columns,
                    unique: is_unique,
                    index_type: index_type.to_string(),
                });
            }
        }

        Ok(indexes)
    }

    /// Parse column names from PostgreSQL index definition
    fn parse_index_columns(index_def: &str) -> Vec<String> {
        // Example: "CREATE INDEX idx_name ON table_name USING btree (col1, col2)"
        if let Some(start) = index_def.find('(') {
            if let Some(end) = index_def.rfind(')') {
                let columns_str = &index_def[start + 1..end].trim();
                if columns_str.is_empty() {
                    return Vec::new();
                }
                return columns_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            }
        }
        Vec::new()
    }

    /// Get foreign key relationships for a table
    async fn get_foreign_keys(&self, table_name: &str) -> DriverResult<Vec<ForeignKey>> {
        let table_name = validate_identifier(table_name)?;
        let query = r#"
            SELECT
                tc.constraint_name,
                tc.table_name as from_table,
                kcu.column_name as from_column,
                ccu.table_name as to_table,
                ccu.column_name as to_column,
                rc.update_rule as on_update,
                rc.delete_rule as on_delete
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
              ON tc.constraint_name = kcu.constraint_name
              AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
              ON ccu.constraint_name = tc.constraint_name
              AND ccu.table_schema = tc.table_schema
            JOIN information_schema.referential_constraints AS rc
              ON rc.constraint_name = tc.constraint_name
              AND rc.constraint_schema = tc.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
              AND tc.table_schema = 'public'
              AND tc.table_name = $1
            ORDER BY tc.constraint_name
        "#;

        let query_with_param = query.replace("$1", &format!("'{}'", table_name));

        let result = self
            .driver
            .execute_query(&query_with_param, vec![])
            .await
            .map_err(|e| {
                DriverError::SchemaIntrospectionFailed(format!(
                    "Failed to get foreign keys for {}: {}",
                    table_name, e
                ))
            })?;

        let mut foreign_keys = Vec::new();
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

                foreign_keys.push(ForeignKey {
                    name: constraint_name,
                    from_table,
                    from_columns: vec![from_column],
                    to_table,
                    to_columns: vec![to_column],
                    on_update,
                    on_delete,
                });
            }
        }

        Ok(foreign_keys)
    }

    /// Get approximate row count for a table
    async fn get_row_count(&self, table_name: &str) -> DriverResult<usize> {
        let table_name = validate_identifier(table_name)?;
        // Use pg_class for fast approximate count
        let query = format!(
            "SELECT reltuples::bigint FROM pg_class WHERE relname = '{}'",
            table_name
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

        // Fallback to exact count if pg_class query fails
        let fallback_query = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
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

    #[test]
    fn test_parse_index_columns_single() {
        let index_def = "CREATE INDEX idx_email ON users USING btree (email)";
        let columns = PostgresSchemaIntrospector::parse_index_columns(index_def);
        assert_eq!(columns, vec!["email"]);
    }

    #[test]
    fn test_parse_index_columns_multiple() {
        let index_def = "CREATE INDEX idx_name ON users USING btree (first_name, last_name)";
        let columns = PostgresSchemaIntrospector::parse_index_columns(index_def);
        assert_eq!(columns, vec!["first_name", "last_name"]);
    }

    #[test]
    fn test_parse_index_columns_with_spaces() {
        let index_def = "CREATE INDEX idx_user ON accounts USING btree ( user_id , created_at )";
        let columns = PostgresSchemaIntrospector::parse_index_columns(index_def);
        assert_eq!(columns, vec!["user_id", "created_at"]);
    }

    #[test]
    fn test_parse_index_columns_empty() {
        let index_def = "CREATE INDEX idx_test ON table_name USING btree ()";
        let columns = PostgresSchemaIntrospector::parse_index_columns(index_def);
        assert_eq!(columns, Vec::<String>::new());
    }

    #[test]
    fn test_parse_index_columns_no_parens() {
        let index_def = "INVALID INDEX DEFINITION";
        let columns = PostgresSchemaIntrospector::parse_index_columns(index_def);
        assert_eq!(columns, Vec::<String>::new());
    }

    #[test]
    fn test_parse_index_columns_gin() {
        let index_def = "CREATE INDEX idx_tags ON posts USING gin (tags)";
        let columns = PostgresSchemaIntrospector::parse_index_columns(index_def);
        assert_eq!(columns, vec!["tags"]);
    }

    #[test]
    fn test_parse_index_columns_complex() {
        let index_def =
            "CREATE UNIQUE INDEX idx_composite ON table_name USING btree (col1, col2, col3)";
        let columns = PostgresSchemaIntrospector::parse_index_columns(index_def);
        assert_eq!(columns, vec!["col1", "col2", "col3"]);
    }

    // Integration tests would require a real database connection
    // These are unit tests for helper functions only

    #[test]
    fn test_schema_introspector_creation() {
        use crate::db_drivers::types::{ConnectionConfig, DatabaseType};
        use std::collections::HashMap;

        let config = ConnectionConfig {
            database_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database: Some("test".to_string()),
            username: "postgres".to_string(),
            password: "password".to_string(),
            ssl_config: None,
            ssh_tunnel_config: None,
            options: HashMap::new(),
        };

        let driver = PostgresDriver::new(config);
        let introspector = PostgresSchemaIntrospector::new(&driver);

        // Just verify it can be constructed
        assert_eq!(
            std::mem::size_of_val(&introspector),
            std::mem::size_of::<&PostgresDriver>()
        );
    }
}
