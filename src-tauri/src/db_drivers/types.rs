// Common data types for database drivers

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Supported database types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MongoDB,
    Cassandra,
    Redis,
}

impl DatabaseType {
    pub fn default_port(&self) -> u16 {
        match self {
            Self::PostgreSQL => 5432,
            Self::MySQL => 3306,
            Self::MongoDB => 27017,
            Self::Cassandra => 9042,
            Self::Redis => 6379,
        }
    }

    /// Parse a database type from a string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "postgresql" | "postgres" | "pg" => Some(Self::PostgreSQL),
            "mysql" => Some(Self::MySQL),
            "mongodb" | "mongo" => Some(Self::MongoDB),
            "cassandra" => Some(Self::Cassandra),
            "redis" => Some(Self::Redis),
            _ => None,
        }
    }
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PostgreSQL => write!(f, "PostgreSQL"),
            Self::MySQL => write!(f, "MySQL"),
            Self::MongoDB => write!(f, "MongoDB"),
            Self::Cassandra => write!(f, "Cassandra"),
            Self::Redis => write!(f, "Redis"),
        }
    }
}

impl FromStr for DatabaseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("Unknown database type: {}", s))
    }
}

/// SSL/TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub verify_server: bool,
}

/// SSH tunnel configuration for database connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSshTunnelConfig {
    pub enabled: bool,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub auth_method: Option<String>,
    pub password: Option<String>,
    pub private_key_data: Option<String>,
    pub key_passphrase: Option<String>,
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub database_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
    pub username: String,
    pub password: String, // Will be encrypted before storage
    pub ssl_config: Option<SslConfig>,
    pub ssh_tunnel_config: Option<DbSshTunnelConfig>,
    pub options: HashMap<String, String>,
}

/// Represents a single data value from query results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum DataValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Date(String),     // ISO 8601 date string
    DateTime(String), // ISO 8601 datetime string
    Json(serde_json::Value),
    Array(Vec<DataValue>),
}

impl DataValue {
    pub fn from_i32(val: i32) -> Self {
        Self::Integer(val as i64)
    }

    pub fn from_i64(val: i64) -> Self {
        Self::Integer(val)
    }

    pub fn from_f64(val: f64) -> Self {
        Self::Float(val)
    }

    pub fn from_bool(val: bool) -> Self {
        Self::Boolean(val)
    }

    pub fn from_string(val: String) -> Self {
        Self::String(val)
    }

    pub fn from_bytes(val: Vec<u8>) -> Self {
        Self::Bytes(val)
    }

    pub fn from_date(date: NaiveDate) -> Self {
        Self::Date(date.format("%Y-%m-%d").to_string())
    }

    pub fn from_datetime(dt: NaiveDateTime) -> Self {
        Self::DateTime(dt.format("%Y-%m-%dT%H:%M:%S").to_string())
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl std::fmt::Display for DataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Integer(i) => write!(f, "{}", i),
            Self::Float(fl) => write!(f, "{}", fl),
            Self::String(s) => write!(f, "{}", s),
            Self::Bytes(b) => write!(f, "<{} bytes>", b.len()),
            Self::Date(d) => write!(f, "{}", d),
            Self::DateTime(dt) => write!(f, "{}", dt),
            Self::Json(j) => write!(f, "{}", serde_json::to_string(j).unwrap_or_default()),
            Self::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
        }
    }
}

/// Column metadata in query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
}

/// Query execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<Vec<DataValue>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
}

/// Database schema column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
    pub auto_increment: bool,
}

/// Database schema index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub index_type: String, // e.g., "BTREE", "HASH"
}

/// Foreign key relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    pub name: String,
    pub from_table: String,
    pub from_columns: Vec<String>,
    pub to_table: String,
    pub to_columns: Vec<String>,
    pub on_delete: String, // e.g., "CASCADE", "SET NULL"
    pub on_update: String,
}

/// Database table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub schema: Option<String>, // Schema/database name (for PostgreSQL)
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub row_count: Option<usize>,
}

/// Complete database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub database_name: String,
    pub tables: Vec<Table>,
}

/// Transaction handle for commit/rollback operations
#[derive(Debug, Clone)]
pub struct TransactionHandle {
    pub id: String,
    pub active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_default_ports() {
        assert_eq!(DatabaseType::PostgreSQL.default_port(), 5432);
        assert_eq!(DatabaseType::MySQL.default_port(), 3306);
        assert_eq!(DatabaseType::MongoDB.default_port(), 27017);
        assert_eq!(DatabaseType::Redis.default_port(), 6379);
        assert_eq!(DatabaseType::Cassandra.default_port(), 9042);
    }

    #[test]
    fn test_database_type_from_str() {
        assert_eq!(
            "postgresql".parse::<DatabaseType>(),
            Ok(DatabaseType::PostgreSQL)
        );
        assert_eq!(
            "postgres".parse::<DatabaseType>(),
            Ok(DatabaseType::PostgreSQL)
        );
        assert_eq!("mysql".parse::<DatabaseType>(), Ok(DatabaseType::MySQL));
        assert_eq!("mongodb".parse::<DatabaseType>(), Ok(DatabaseType::MongoDB));
        assert_eq!("redis".parse::<DatabaseType>(), Ok(DatabaseType::Redis));
        assert_eq!(
            "cassandra".parse::<DatabaseType>(),
            Ok(DatabaseType::Cassandra)
        );
        assert!("invalid".parse::<DatabaseType>().is_err());
    }

    #[test]
    fn test_database_type_display() {
        assert_eq!(DatabaseType::PostgreSQL.to_string(), "PostgreSQL");
        assert_eq!(DatabaseType::MySQL.to_string(), "MySQL");
        assert_eq!(DatabaseType::MongoDB.to_string(), "MongoDB");
        assert_eq!(DatabaseType::Redis.to_string(), "Redis");
        assert_eq!(DatabaseType::Cassandra.to_string(), "Cassandra");
    }

    #[test]
    fn test_data_value_display() {
        assert_eq!(DataValue::Null.to_string(), "NULL");
        assert_eq!(DataValue::Boolean(true).to_string(), "true");
        assert_eq!(DataValue::Integer(42).to_string(), "42");
        assert_eq!(DataValue::Float(3.14).to_string(), "3.14");
        assert_eq!(DataValue::String("test".to_string()).to_string(), "test");
    }

    #[test]
    fn test_data_value_is_null() {
        assert!(DataValue::Null.is_null());
        assert!(!DataValue::Integer(0).is_null());
        assert!(!DataValue::String("".to_string()).is_null());
    }
}
