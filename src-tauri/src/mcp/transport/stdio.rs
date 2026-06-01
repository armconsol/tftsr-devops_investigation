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
        let mut env = HashMap::new();
        env.insert("DEBUG".to_string(), "1".to_string());
        env.insert("API_KEY".to_string(), "secret123".to_string());
        env.insert("PATH".to_string(), "/usr/bin".to_string());

        // Note: This will fail to spawn since /usr/bin/test may not exist,
        // but we're testing that it doesn't reject the env vars
        let result = build_stdio_transport("/usr/bin/test", &[], env);

        // Should fail with spawn error, not env var validation error
        if let Err(err) = result {
            assert!(
                !err.contains("Dangerous environment variable"),
                "Should not reject safe env vars, got: {}",
                err
            );
        }
    }
}
