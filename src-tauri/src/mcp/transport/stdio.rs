use rmcp::transport::TokioChildProcess;
use std::collections::HashMap;
use std::path::Path;
use tokio::process::Command;

/// Build a stdio transport from a command path, argument list, and environment variables.
/// Rejects relative paths to prevent path traversal.
/// Validates environment variable names to block known privilege escalation vectors.
pub fn build_stdio_transport(
    command: &str,
    args: &[String],
    env: HashMap<String, String>,
) -> Result<TokioChildProcess, String> {
    if !Path::new(command).is_absolute() {
        return Err(format!(
            "stdio command must be an absolute path, got: {command}"
        ));
    }

    // Validate env var names to block dangerous variables that could be used for privilege escalation
    const DANGEROUS_ENV_VARS: &[&str] = &[
        "LD_PRELOAD",
        "LD_LIBRARY_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_LIBRARY_PATH",
        "DYLD_FRAMEWORK_PATH",
        "DYLD_FALLBACK_LIBRARY_PATH",
    ];

    for key in env.keys() {
        let upper_key = key.to_uppercase();
        if DANGEROUS_ENV_VARS.contains(&upper_key.as_str()) {
            return Err(format!(
                "Dangerous environment variable '{key}' is not allowed for security reasons. \
                 This variable could be used for privilege escalation attacks."
            ));
        }
    }

    let mut cmd = Command::new(command);
    cmd.args(args);

    // Apply environment variables
    for (key, value) in env {
        cmd.env(key, value);
    }

    TokioChildProcess::new(cmd).map_err(|e| format!("Failed to spawn stdio process: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_relative_path() {
        let result = build_stdio_transport("./mcp-server", &[], HashMap::new());
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.contains("absolute path"));
        }
    }

    #[test]
    fn test_rejects_dangerous_env_vars() {
        let dangerous_vars = vec![
            "LD_PRELOAD",
            "LD_LIBRARY_PATH",
            "DYLD_INSERT_LIBRARIES",
            "DYLD_LIBRARY_PATH",
            "DYLD_FRAMEWORK_PATH",
            "DYLD_FALLBACK_LIBRARY_PATH",
        ];

        for var in dangerous_vars {
            let mut env = HashMap::new();
            env.insert(var.to_string(), "malicious.so".to_string());

            let result = build_stdio_transport("/usr/bin/test", &[], env);
            assert!(result.is_err(), "Should reject {}", var);
            if let Err(err) = result {
                assert!(
                    err.contains("Dangerous environment variable"),
                    "Error should mention dangerous variable: {}",
                    err
                );
            }
        }
    }

    #[test]
    fn test_rejects_dangerous_env_vars_case_insensitive() {
        let mut env = HashMap::new();
        env.insert("ld_preload".to_string(), "malicious.so".to_string());

        let result = build_stdio_transport("/usr/bin/test", &[], env);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.contains("Dangerous environment variable"));
        }
    }

    #[test]
    fn test_allows_safe_env_vars() {
        // Test that safe env vars pass validation (validation happens before spawn)
        let safe_vars = vec![
            ("DEBUG", "1"),
            ("API_KEY", "secret123"),
            ("PATH", "/usr/bin"),
            ("HOME", "/home/user"),
            ("GITHUB_PERSONAL_ACCESS_TOKEN", "ghp_token"),
            ("LOG_LEVEL", "info"),
        ];

        for (key, value) in safe_vars {
            let mut env = HashMap::new();
            env.insert(key.to_string(), value.to_string());

            // This will fail to spawn since /usr/bin/nonexistent doesn't exist,
            // but if validation passed, error won't mention "Dangerous environment variable"
            let result = build_stdio_transport("/usr/bin/nonexistent", &[], env);

            // Validation passes (doesn't reject env var), spawn fails (command doesn't exist)
            if let Err(err) = result {
                assert!(
                    !err.contains("Dangerous environment variable"),
                    "Should not reject safe env var '{}', got: {}",
                    key,
                    err
                );
            }
        }
    }
}
