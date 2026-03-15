use crate::ollama::{InstallGuide, OllamaStatus};

pub async fn check_ollama() -> anyhow::Result<OllamaStatus> {
    // Check if binary exists
    let which_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    let which_result = std::process::Command::new(which_cmd)
        .arg("ollama")
        .output();

    let installed = which_result
        .map(|o| o.status.success())
        .unwrap_or(false);

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

    Ok(OllamaStatus {
        installed,
        version,
        running,
    })
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
        assert!(guide.steps.iter().any(|s| s.contains("dmg") || s.contains("Applications")));
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
