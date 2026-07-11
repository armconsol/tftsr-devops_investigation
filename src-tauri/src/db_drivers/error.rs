// Error types for database drivers

use std::fmt;

/// Result type alias for database driver operations
pub type DriverResult<T> = Result<T, DriverError>;

/// Comprehensive error type for all database operations
#[derive(Debug, Clone)]
pub enum DriverError {
    /// Connection failed (network, auth, or config issues)
    ConnectionFailed(String),

    /// Already connected when trying to connect
    AlreadyConnected,

    /// Not connected when operation requires connection
    NotConnected,

    /// Query execution failed
    QueryExecutionFailed(String),

    /// Transaction operation failed
    TransactionFailed(String),

    /// Schema introspection failed
    SchemaIntrospectionFailed(String),

    /// Unsupported operation for this database type
    UnsupportedOperation(String),

    /// Invalid configuration
    InvalidConfig(String),

    /// Timeout occurred
    Timeout(String),

    /// Data type conversion error
    TypeConversionError(String),

    /// SSL/TLS error
    SslError(String),

    /// Authentication error
    AuthenticationError(String),

    /// Permission denied
    PermissionDenied(String),

    /// Database not found
    DatabaseNotFound(String),

    /// Table not found
    TableNotFound(String),

    /// Validation error (invalid data format, constraints, etc.)
    ValidationError(String),

    /// Serialization/deserialization error
    SerializationError(String),

    /// Parse error (query parsing, data parsing, etc.)
    ParseError(String),

    /// Feature not supported by this driver
    NotSupported(String),

    /// IO error (file operations, network, etc.)
    IoError(String),

    /// Generic driver error
    Other(String),
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Self::AlreadyConnected => write!(f, "Already connected to database"),
            Self::NotConnected => write!(f, "Not connected to database"),
            Self::QueryExecutionFailed(msg) => write!(f, "Query execution failed: {}", msg),
            Self::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            Self::SchemaIntrospectionFailed(msg) => {
                write!(f, "Schema introspection failed: {}", msg)
            }
            Self::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Self::TypeConversionError(msg) => write!(f, "Type conversion error: {}", msg),
            Self::SslError(msg) => write!(f, "SSL/TLS error: {}", msg),
            Self::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::DatabaseNotFound(msg) => write!(f, "Database not found: {}", msg),
            Self::TableNotFound(msg) => write!(f, "Table not found: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::Other(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for DriverError {}

// Implement From traits for common error types
impl From<std::io::Error> for DriverError {
    fn from(err: std::io::Error) -> Self {
        DriverError::Other(err.to_string())
    }
}

impl From<serde_json::Error> for DriverError {
    fn from(err: serde_json::Error) -> Self {
        DriverError::TypeConversionError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = DriverError::ConnectionFailed("host unreachable".to_string());
        assert_eq!(error.to_string(), "Connection failed: host unreachable");
    }

    #[test]
    fn test_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let driver_error: DriverError = io_error.into();
        assert!(matches!(driver_error, DriverError::Other(_)));
    }
}
