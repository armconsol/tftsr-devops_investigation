use crate::ollama::{InstallGuide, OllamaStatus};

pub async fn check_ollama() -> anyhow::Result<OllamaStatus> {
    // Check if binary exists
    let which_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    let which_result = std::process::Command::new(which_cmd).arg("ollama").output();

    // Check common install paths explicitly — Tauri's process PATH may omit /usr/local/bin
    let in_common_path = [
        "/usr/local/bin/ollama",
        "/opt/homebrew/bin/ollama",
        "/usr/bin/ollama",
    ]
    .iter()
    .any(|p| std::path::Path::new(p).exists());

    let installed = which_result.map(|o| o.status.success()).unwrap_or(false) || in_common_path;

    let version = if installed {
        std::process::Command::new("ollama")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    // Check if Ollama API is responding
    let running = reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    // If the API is responding, Ollama is definitely installed even if binary wasn't found in PATH
    let installed = installed || running;

    Ok(OllamaStatus {
        installed,
        version,
        running,
    })
}

/// Attempt to start Ollama service if installed but not running
pub async fn start_ollama_service() -> anyhow::Result<bool> {
    let status = check_ollama().await?;

    // If already running, nothing to do
    if status.running {
        tracing::info!("Ollama is already running");
        return Ok(true);
    }

    // If not installed, can't start it
    if !status.installed {
        tracing::warn!("Ollama is not installed, cannot auto-start");
        return Ok(false);
    }

    tracing::info!("Ollama is installed but not running, attempting to start...");

    // Platform-specific start logic
    #[cfg(target_os = "macos")]
    {
        // On macOS, try to launch Ollama.app which manages the service
        let ollama_app = "/Applications/Ollama.app";
        if std::path::Path::new(ollama_app).exists() {
            tracing::info!("Launching Ollama.app...");
            let result = std::process::Command::new("open")
                .arg(ollama_app)
                .spawn();

            match result {
                Ok(_) => {
                    // Wait a few seconds for Ollama to start
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                    // Check if it's now running
                    let new_status = check_ollama().await?;
                    if new_status.running {
                        tracing::info!("Ollama started successfully via Ollama.app");
                        return Ok(true);
                    } else {
                        tracing::warn!("Ollama.app launched but service not responding yet");
                        return Ok(false);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to launch Ollama.app: {}", e);
                }
            }
        }

        // Fallback: try direct ollama serve
        tracing::info!("Attempting to start ollama serve directly...");
        let result = std::process::Command::new("ollama")
            .arg("serve")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(_) => {
                // Wait for service to become available
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let new_status = check_ollama().await?;
                Ok(new_status.running)
            }
            Err(e) => {
                tracing::error!("Failed to start ollama serve: {}", e);
                Ok(false)
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, start ollama serve in background
        tracing::info!("Starting ollama serve...");
        let result = std::process::Command::new("ollama")
            .arg("serve")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(_) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let new_status = check_ollama().await?;
                if new_status.running {
                    tracing::info!("Ollama started successfully");
                    Ok(true)
                } else {
                    tracing::warn!("ollama serve started but not responding yet");
                    Ok(false)
                }
            }
            Err(e) => {
                tracing::error!("Failed to start ollama serve: {}", e);
                Ok(false)
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, Ollama should be running as a system service
        // If it's not, user needs to start it manually or reinstall
        tracing::warn!("Ollama is installed but not running on Windows - user should start it from system tray");
        Ok(false)
    }
}

pub fn get_install_instructions(platform: &str) -> InstallGuide {
    let url = "https://ollama.com/download".to_string();
    match platform {
        "linux" => InstallGuide {
            platform: "Linux".to_string(),
            steps: vec![
                "Open a terminal".to_string(),
                "Run: curl -fsSL https://ollama.com/install.sh | sh".to_string(),
                "Start Ollama: ollama serve".to_string(),
                "Pull a model: ollama pull llama3.2:3b".to_string(),
            ],
            url,
        },
        "macos" => InstallGuide {
            platform: "macOS".to_string(),
            steps: vec![
                "Download the macOS installer from ollama.com/download".to_string(),
                "Open the downloaded .dmg file".to_string(),
                "Drag Ollama to Applications".to_string(),
                "Launch Ollama from Applications".to_string(),
                "Pull a model: ollama pull llama3.2:3b".to_string(),
            ],
            url,
        },
        "windows" => InstallGuide {
            platform: "Windows".to_string(),
            steps: vec![
                "Download OllamaSetup.exe from ollama.com/download".to_string(),
                "Run the installer and follow the prompts".to_string(),
                "Ollama will start automatically in the system tray".to_string(),
                "Pull a model: ollama pull llama3.2:3b".to_string(),
            ],
            url,
        },
        _ => InstallGuide {
            platform: platform.to_string(),
            steps: vec![
                "Visit https://ollama.com/download for installation instructions".to_string(),
            ],
            url,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_install_guide() {
        let guide = get_install_instructions("linux");
        assert_eq!(guide.platform, "Linux");
        assert!(!guide.steps.is_empty());
        assert!(guide.steps.iter().any(|s| s.contains("curl")));
        assert!(guide.url.contains("ollama.com"));
    }

    #[test]
    fn test_macos_install_guide() {
        let guide = get_install_instructions("macos");
        assert_eq!(guide.platform, "macOS");
        assert!(guide
            .steps
            .iter()
            .any(|s| s.contains("dmg") || s.contains("Applications")));
    }

    #[test]
    fn test_windows_install_guide() {
        let guide = get_install_instructions("windows");
        assert_eq!(guide.platform, "Windows");
        assert!(guide.steps.iter().any(|s| s.contains("OllamaSetup")));
    }

    #[test]
    fn test_unknown_platform_fallback() {
        let guide = get_install_instructions("freebsd");
        assert_eq!(guide.platform, "freebsd");
        assert_eq!(guide.steps.len(), 1);
        assert!(guide.steps[0].contains("ollama.com"));
    }
}
