use rmcp::transport::TokioChildProcess;
use std::collections::HashMap;
use std::path::Path;
use tokio::process::Command;

/// Build a stdio transport from a command path, argument list, and environment variables.
/// Rejects relative paths to prevent path traversal.
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

    let mut cmd = Command::new(command);
    cmd.args(args);

    // Apply environment variables
    for (key, value) in env {
        cmd.env(key, value);
    }

    TokioChildProcess::new(cmd).map_err(|e| format!("Failed to spawn stdio process: {e}"))
}
