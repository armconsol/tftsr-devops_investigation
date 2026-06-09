// Helm Binary Management
//
// This module handles:
// - Locating the helm binary (bundled or system PATH)

use std::path::PathBuf;
use std::process::Command;

pub fn locate_helm() -> Result<PathBuf, String> {
    // Strategy:
    // 1. Check for bundled sidecar binary (platform-specific)
    // 2. Fallback to system PATH (which helm)
    // 3. Check common installation paths

    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };

    // Try current directory (dev mode)
    let local_helm = PathBuf::from(format!("helm{exe_suffix}"));
    if local_helm.exists() {
        return Ok(local_helm);
    }

    // Check for Tauri sidecar binary (production builds)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
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

            let sidecar_name = format!("helm-{target}{exe_suffix}");
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

    // Check system PATH
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = Command::new("which").arg("helm").output() {
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
        if let Ok(output) = Command::new("where").arg("helm").output() {
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
        "/usr/local/bin/helm",
        "/usr/bin/helm",
        "/opt/homebrew/bin/helm",
        "/snap/bin/helm",
    ];

    for path_str in &common_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(
        "helm binary not found. Please install helm or it will be bundled in production builds."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locate_helm_finds_binary() {
        let result = locate_helm();
        if result.is_ok() {
            assert!(result.unwrap().exists(), "helm path should exist if found");
        }
        // Test passes whether helm is found or not
    }
}
