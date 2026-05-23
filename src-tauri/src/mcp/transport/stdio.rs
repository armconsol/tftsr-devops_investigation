use rmcp::transport::TokioChildProcess;
use std::path::Path;
use tokio::process::Command;

/// Build a stdio transport from a command path and argument list.
/// Rejects relative paths to prevent path traversal.
pub fn build_stdio_transport(
    command: &str,
    args: &[String],
) -> Result<TokioChildProcess, String> {
    if !Path::new(command).is_absolute() {
        return Err(format!(
            "stdio command must be an absolute path, got: {command}"
        ));
    }

    let mut cmd = Command::new(command);
    cmd.args(args);

    TokioChildProcess::new(cmd).map_err(|e| format!("Failed to spawn stdio process: {e}"))
}
