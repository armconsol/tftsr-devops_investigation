use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub total_ram_gb: f64,
    pub cpu_arch: String,
    pub gpu_vendor: Option<String>,
    pub gpu_vram_gb: Option<f64>,
}

pub fn probe_hardware() -> anyhow::Result<HardwareInfo> {
    let total_ram_gb = detect_ram_gb();
    let cpu_arch = std::env::consts::ARCH.to_string();
    let (gpu_vendor, gpu_vram_gb) = detect_gpu();

    Ok(HardwareInfo {
        total_ram_gb,
        cpu_arch,
        gpu_vendor,
        gpu_vram_gb,
    })
}

fn detect_ram_gb() -> f64 {
    // Linux: parse /proc/meminfo
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(kb_str) = parts.get(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb as f64 / 1_048_576.0;
                        }
                    }
                }
            }
        }
    }
    // macOS: use sysctl
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output();
        if let Ok(out) = output {
            if let Ok(s) = String::from_utf8(out.stdout) {
                if let Ok(bytes) = s.trim().parse::<u64>() {
                    return bytes as f64 / 1_073_741_824.0;
                }
            }
        }
    }
    // Windows: use wmic
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("wmic")
            .args(["computersystem", "get", "TotalPhysicalMemory"])
            .output();
        if let Ok(out) = output {
            if let Ok(s) = String::from_utf8(out.stdout) {
                for line in s.lines().skip(1) {
                    if let Ok(bytes) = line.trim().parse::<u64>() {
                        return bytes as f64 / 1_073_741_824.0;
                    }
                }
            }
        }
    }
    8.0 // fallback
}

fn detect_gpu() -> (Option<String>, Option<f64>) {
    // Try nvidia-smi first
    if let Ok(out) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            let line = s.lines().next().unwrap_or("");
            let parts: Vec<&str> = line.split(',').collect();
            let name = parts.first().map(|s| s.trim().to_string());
            let vram = parts
                .get(1)
                .and_then(|s| s.trim().strip_suffix(" MiB"))
                .and_then(|s| s.parse::<f64>().ok())
                .map(|mb| mb / 1024.0);
            return (name, vram);
        }
    }

    // Check for AMD (rocm-smi)
    if let Ok(out) = std::process::Command::new("rocm-smi")
        .arg("--showmeminfo")
        .arg("vram")
        .output()
    {
        if out.status.success() {
            return (Some("AMD GPU".to_string()), None);
        }
    }

    // macOS: Apple Silicon GPU is integrated, detected via system_profiler
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            if s.contains("Apple") {
                return (Some("Apple Silicon (integrated)".to_string()), None);
            }
        }
    }

    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_hardware_returns_valid_result() {
        let hw = probe_hardware().expect("probe_hardware should not fail");
        // RAM should be > 0
        assert!(hw.total_ram_gb > 0.0);
        // CPU arch should not be empty
        assert!(!hw.cpu_arch.is_empty());
    }

    #[test]
    fn test_detect_ram_returns_positive() {
        let ram = detect_ram_gb();
        assert!(ram > 0.0, "RAM should be a positive number, got {}", ram);
    }

    #[test]
    fn test_hardware_info_serializes() {
        let hw = HardwareInfo {
            total_ram_gb: 16.0,
            cpu_arch: "x86_64".to_string(),
            gpu_vendor: Some("NVIDIA".to_string()),
            gpu_vram_gb: Some(8.0),
        };
        let json = serde_json::to_string(&hw).expect("should serialize");
        assert!(json.contains("total_ram_gb"));
        assert!(json.contains("NVIDIA"));
    }

    #[test]
    fn test_hardware_info_without_gpu_serializes() {
        let hw = HardwareInfo {
            total_ram_gb: 8.0,
            cpu_arch: "aarch64".to_string(),
            gpu_vendor: None,
            gpu_vram_gb: None,
        };
        let json = serde_json::to_string(&hw).expect("should serialize");
        assert!(json.contains("null"));
    }
}
