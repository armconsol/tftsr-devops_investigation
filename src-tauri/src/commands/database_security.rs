// Security utilities for database operations

use regex::Regex;
use std::path::{Path, PathBuf};

/// Validates SQL identifiers (table names, column names) to prevent SQL injection
///
/// Only allows alphanumeric characters and underscores
pub fn validate_sql_identifier(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Identifier cannot be empty".to_string());
    }

    if name.len() > 64 {
        return Err(format!("Identifier too long: {} characters (max 64)", name.len()));
    }

    // Allow alphanumeric, underscore, and period (for schema.table notation)
    let valid_chars = name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.');

    if !valid_chars {
        return Err(format!(
            "Invalid identifier '{}': only alphanumeric, underscore, and period allowed",
            name
        ));
    }

    // Prevent SQL keywords being used as unquoted identifiers
    let uppercase = name.to_uppercase();
    let sql_keywords = [
        "SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "CREATE", "ALTER", "FROM", "WHERE",
        "JOIN", "UNION", "ORDER", "GROUP", "HAVING",
    ];

    if sql_keywords.contains(&uppercase.as_str()) {
        return Err(format!(
            "SQL keyword '{}' cannot be used as identifier",
            name
        ));
    }

    Ok(())
}

/// Validates file paths to prevent directory traversal attacks
///
/// Ensures paths are within allowed directories and don't contain traversal sequences
pub fn validate_file_path(path: &str, allowed_dirs: &[&str]) -> Result<PathBuf, String> {
    let path_buf = PathBuf::from(path);

    // Check for path traversal attempts
    if path.contains("..") {
        return Err("Path traversal attempt detected (..)".to_string());
    }

    // Canonicalize to resolve symlinks and relative paths
    let canonical = path_buf
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    // Check if path is within allowed directories
    let is_allowed = allowed_dirs.iter().any(|allowed_dir| {
        let allowed_canonical = PathBuf::from(allowed_dir)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(allowed_dir));
        canonical.starts_with(&allowed_canonical)
    });

    if !is_allowed {
        return Err(format!(
            "Path '{}' is outside allowed directories",
            canonical.display()
        ));
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sql_identifier_valid() {
        assert!(validate_sql_identifier("users").is_ok());
        assert!(validate_sql_identifier("user_accounts").is_ok());
        assert!(validate_sql_identifier("schema123").is_ok());
        assert!(validate_sql_identifier("public.users").is_ok());
    }

    #[test]
    fn test_validate_sql_identifier_invalid_chars() {
        assert!(validate_sql_identifier("users; DROP TABLE").is_err());
        assert!(validate_sql_identifier("user'name").is_err());
        assert!(validate_sql_identifier("user-name").is_err());
        assert!(validate_sql_identifier("user name").is_err());
    }

    #[test]
    fn test_validate_sql_identifier_empty() {
        assert!(validate_sql_identifier("").is_err());
    }

    #[test]
    fn test_validate_sql_identifier_too_long() {
        let long_name = "a".repeat(65);
        assert!(validate_sql_identifier(&long_name).is_err());
    }

    #[test]
    fn test_validate_sql_identifier_sql_keywords() {
        assert!(validate_sql_identifier("SELECT").is_err());
        assert!(validate_sql_identifier("drop").is_err());
        assert!(validate_sql_identifier("DELETE").is_err());
    }

    #[test]
    fn test_validate_file_path_traversal() {
        let result = validate_file_path("../etc/passwd", &["/tmp"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("traversal"));
    }

    #[test]
    fn test_validate_file_path_outside_allowed() {
        // Create a temp file for testing
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test.csv");
        std::fs::write(&temp_file, "test").unwrap();

        let result = validate_file_path(
            temp_file.to_str().unwrap(),
            &["/some/other/directory"],
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("outside allowed"));

        // Cleanup
        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_validate_file_path_valid() {
        // Create a temp file for testing
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test.csv");
        std::fs::write(&temp_file, "test").unwrap();

        let result = validate_file_path(
            temp_file.to_str().unwrap(),
            &[temp_dir.to_str().unwrap()],
        );
        assert!(result.is_ok());

        // Cleanup
        std::fs::remove_file(&temp_file).ok();
    }
}
