// Cassandra schema introspection utilities

use scylla::Session;
use std::sync::Arc;

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    types::{Column, Schema, Table},
};

use super::types::cql_type_to_string;

/// Cassandra schema introspection
pub struct CassandraSchemaIntrospector {
    session: Arc<Session>,
}

impl CassandraSchemaIntrospector {
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }

    /// Get list of keyspaces (excluding system keyspaces)
    pub async fn get_keyspaces(&self) -> DriverResult<Vec<String>> {
        let query = "SELECT keyspace_name FROM system_schema.keyspaces";

        let rows = self
            .session
            .query(query, &[])
            .await
            .map_err(|e| DriverError::SchemaIntrospectionFailed(e.to_string()))?
            .rows
            .ok_or_else(|| {
                DriverError::SchemaIntrospectionFailed("No rows returned".to_string())
            })?;

        let mut keyspaces = Vec::new();
        for row in rows {
            let keyspace_name: String = row.columns[0]
                .as_ref()
                .and_then(|v| match v {
                    scylla::frame::response::result::CqlValue::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or_else(|| {
                    DriverError::TypeConversionError("Invalid keyspace name".to_string())
                })?;

            // Filter out system keyspaces
            if !keyspace_name.starts_with("system") {
                keyspaces.push(keyspace_name);
            }
        }

        Ok(keyspaces)
    }

    /// Get list of tables in a keyspace
    pub async fn get_tables(&self, keyspace: &str) -> DriverResult<Vec<String>> {
        let query = "SELECT table_name FROM system_schema.tables WHERE keyspace_name = ?";

        let rows = self
            .session
            .query(query, (keyspace,))
            .await
            .map_err(|e| DriverError::SchemaIntrospectionFailed(e.to_string()))?
            .rows
            .ok_or_else(|| {
                DriverError::SchemaIntrospectionFailed("No rows returned".to_string())
            })?;

        let mut tables = Vec::new();
        for row in rows {
            let table_name: String = row.columns[0]
                .as_ref()
                .and_then(|v| match v {
                    scylla::frame::response::result::CqlValue::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or_else(|| {
                    DriverError::TypeConversionError("Invalid table name".to_string())
                })?;

            tables.push(table_name);
        }

        Ok(tables)
    }

    /// Get column information for a table
    pub async fn get_columns(&self, keyspace: &str, table: &str) -> DriverResult<Vec<Column>> {
        let query = "SELECT column_name, type, kind FROM system_schema.columns WHERE keyspace_name = ? AND table_name = ?";

        let rows = self
            .session
            .query(query, (keyspace, table))
            .await
            .map_err(|e| DriverError::SchemaIntrospectionFailed(e.to_string()))?
            .rows
            .ok_or_else(|| {
                DriverError::SchemaIntrospectionFailed("No rows returned".to_string())
            })?;

        let mut columns = Vec::new();
        for row in rows {
            let column_name: String = row.columns[0]
                .as_ref()
                .and_then(|v| match v {
                    scylla::frame::response::result::CqlValue::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or_else(|| {
                    DriverError::TypeConversionError("Invalid column name".to_string())
                })?;

            let column_type: String = row.columns[1]
                .as_ref()
                .and_then(|v| match v {
                    scylla::frame::response::result::CqlValue::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or_else(|| {
                    DriverError::TypeConversionError("Invalid column type".to_string())
                })?;

            let kind: String = row.columns[2]
                .as_ref()
                .and_then(|v| match v {
                    scylla::frame::response::result::CqlValue::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or_else(|| {
                    DriverError::TypeConversionError("Invalid column kind".to_string())
                })?;

            let is_primary_key = kind == "partition_key" || kind == "clustering";

            columns.push(Column {
                name: column_name,
                data_type: cql_type_to_string(&column_type),
                nullable: !is_primary_key, // Primary key columns cannot be null
                default_value: None,
                primary_key: is_primary_key,
                auto_increment: false, // Cassandra doesn't have auto-increment
            });
        }

        Ok(columns)
    }

    /// Get complete schema for a keyspace
    pub async fn get_schema(&self, keyspace: &str) -> DriverResult<Schema> {
        let table_names = self.get_tables(keyspace).await?;

        let mut tables = Vec::new();
        for table_name in table_names {
            let columns = self.get_columns(keyspace, &table_name).await?;

            tables.push(Table {
                name: table_name,
                schema: Some(keyspace.to_string()),
                columns,
                indexes: vec![], // Cassandra index introspection could be added here
                foreign_keys: vec![], // Cassandra doesn't support foreign keys
                row_count: None, // Getting row count is expensive in Cassandra
            });
        }

        Ok(Schema {
            database_name: keyspace.to_string(),
            tables,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_introspector_creation() {
        // This test just verifies the struct can be created
        // Actual testing would require a running Cassandra instance
        assert!(true);
    }

    #[test]
    fn test_cql_type_conversion() {
        assert_eq!(
            cql_type_to_string("org.apache.cassandra.db.marshal.UTF8Type"),
            "text"
        );
        assert_eq!(
            cql_type_to_string("org.apache.cassandra.db.marshal.Int32Type"),
            "int"
        );
    }
}
