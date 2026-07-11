// Data import/export functionality for database management

pub mod csv;
pub mod json;
pub mod sql;

use serde::{Deserialize, Serialize};

/// Statistics returned after import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStats {
    pub rows_processed: usize,
    pub rows_inserted: usize,
    pub rows_failed: usize,
    pub errors: Vec<String>,
    pub execution_time_ms: u64,
}

/// Statistics returned after export operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    pub rows_exported: usize,
    pub file_size_bytes: u64,
    pub execution_time_ms: u64,
}

/// Column mapping from source to target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub source_column: String,
    pub target_column: String,
    pub transform: Option<String>, // Optional SQL expression for transformation
}

/// Import options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub skip_errors: bool,
    pub batch_size: usize,
    pub truncate_first: bool,
    pub column_mappings: Vec<ColumnMapping>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            skip_errors: false,
            batch_size: 1000,
            truncate_first: false,
            column_mappings: vec![],
        }
    }
}
