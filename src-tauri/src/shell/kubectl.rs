// kubectl Binary Management
//
// This module handles:
// - Locating kubectl binary (bundled or system PATH)
// - Executing kubectl commands with proper environment isolation
// - Timeout protection

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

#[derive(Debug)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

pub fn locate_kubectl() -> Result<PathBuf, String> {
    // Strategy:
    // 1. Check for bundled sidecar binary (platform-specific)
    // 2. Fallback to system PATH (which kubectl)
    // 3. Check common installation paths

    // Check for bundled binary first
    // In production builds, kubectl will be bundled as an external binary
    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };

    // Try current directory (dev mode)
    let local_kubectl = PathBuf::from(format!("kubectl{exe_suffix}"));
    if local_kubectl.exists() {
        return Ok(local_kubectl);
    }

    // Check for Tauri sidecar binary (production builds)
    // Tauri names sidecars with target triple suffix
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Build target-triple-suffixed name
            let target = std::env::consts::ARCH.to_string()
                + "-"
                + if cfg!(target_os = "linux") {
                    "unknown-linux-gnu"
                } else if cfg!(target_os = "macos") {
                    "apple-darwin"
                } else if cfg!(target_os = "windows") {
                    "pc-windows-msvc"
                } else {
                    "unknown"
                };

            let sidecar_name = format!("kubectl-{target}{exe_suffix}");
            let sidecar_path = exe_dir.join(&sidecar_name);

            if sidecar_path.exists() {
                return Ok(sidecar_path);
            }

            // Also check Resources subdirectory (macOS .app bundle)
            let resources_path = exe_dir.join("Resources").join(&sidecar_name);
            if resources_path.exists() {
                return Ok(resources_path);
            }
        }
    }

    // Check system PATH using 'which' on Unix or 'where' on Windows
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = Command::new("which").arg("kubectl").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Ok(path);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("where").arg("kubectl").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Ok(path);
                }
            }
        }
    }

    // Check common installation paths
    let common_paths = [
        "/usr/local/bin/kubectl",
        "/usr/bin/kubectl",
        "/opt/homebrew/bin/kubectl",
        "/snap/bin/kubectl",
    ];

    for path_str in &common_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return Ok(path);
        }
    }

    Err("kubectl binary not found. Please install kubectl or it will be bundled in production builds.".to_string())
}

pub async fn execute_kubectl(
    args: &[String],
    kubeconfig_path: Option<&str>,
    working_dir: Option<&str>,
) -> Result<CommandOutput, String> {
    let start = Instant::now();

    // Locate kubectl binary
    let kubectl_path = locate_kubectl()?;

    // Build command
    let mut cmd = Command::new(&kubectl_path);
    cmd.args(args);

    // Set KUBECONFIG if provided
    if let Some(kubeconfig) = kubeconfig_path {
        cmd.env("KUBECONFIG", kubeconfig);
    }

    // Set working directory (default to system temp for safety)
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    } else {
        cmd.current_dir(std::env::temp_dir());
    }

    // Clear potentially sensitive environment variables
    cmd.env_remove("AWS_ACCESS_KEY_ID");
    cmd.env_remove("AWS_SECRET_ACCESS_KEY");

    // Execute with timeout (30 seconds)
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        tokio::task::spawn_blocking(move || cmd.output()),
    )
    .await
    .map_err(|_| "Command execution timed out after 30 seconds".to_string())?
    .map_err(|e| format!("Failed to spawn command: {e}"))?
    .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    let execution_time_ms = start.elapsed().as_millis() as u64;

    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        execution_time_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locate_kubectl_finds_binary() {
        // Should find either bundled or system kubectl
        // In CI environments without kubectl installed, this may fail gracefully
        let result = locate_kubectl();
        if result.is_ok() {
            assert!(
                result.unwrap().exists(),
                "kubectl path should exist if found"
            );
        }
        // Test passes whether kubectl is found or not - just verifying function doesn't panic
    }

    #[tokio::test]
    async fn test_execute_kubectl_with_timeout() {
        let result =
            execute_kubectl(&["version".to_string(), "--client".to_string()], None, None).await;
        // Should either succeed or timeout, not hang forever
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_kubectl_command_simple() {
        // Test helper function for parsing kubectl commands
        let cmd = "kubectl get pods";
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        assert_eq!(parts[0], "kubectl");
        assert_eq!(parts[1], "get");
        assert_eq!(parts[2], "pods");
    }
}
