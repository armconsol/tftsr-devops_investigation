use crate::ollama::hardware::HardwareInfo;
use crate::ollama::ModelRecommendation;

pub fn recommend_models(hw: &HardwareInfo) -> Vec<ModelRecommendation> {
    let ram = hw.total_ram_gb;
    let has_gpu = hw.gpu_vendor.is_some();

    let mut models = vec![
        ModelRecommendation {
            name: "llama3.2:1b".to_string(),
            size: "1.3 GB".to_string(),
            min_ram_gb: 4.0,
            description: "Smallest Llama 3.2 model. Fast, runs on minimal hardware.".to_string(),
            recommended: ram < 8.0,
        },
        ModelRecommendation {
            name: "llama3.2:3b".to_string(),
            size: "2.0 GB".to_string(),
            min_ram_gb: 6.0,
            description: "Balanced Llama 3.2 model. Good for most IT triage tasks.".to_string(),
            recommended: (8.0..16.0).contains(&ram),
        },
        ModelRecommendation {
            name: "phi3.5:3.8b".to_string(),
            size: "2.2 GB".to_string(),
            min_ram_gb: 6.0,
            description: "Microsoft Phi-3.5. Excellent reasoning for its size.".to_string(),
            recommended: false,
        },
        ModelRecommendation {
            name: "llama3.1:8b".to_string(),
            size: "4.7 GB".to_string(),
            min_ram_gb: 10.0,
            description: "Llama 3.1 8B. Strong performance for IT analysis.".to_string(),
            recommended: (16.0..32.0).contains(&ram),
        },
        ModelRecommendation {
            name: "qwen2.5:14b".to_string(),
            size: "9.0 GB".to_string(),
            min_ram_gb: 16.0,
            description: "Qwen 2.5 14B. Excellent for complex log analysis.".to_string(),
            recommended: (24.0..40.0).contains(&ram),
        },
        ModelRecommendation {
            name: "llama3.1:70b".to_string(),
            size: "40 GB".to_string(),
            min_ram_gb: 48.0,
            description: "Full Llama 3.1 70B. Best quality, requires significant RAM.".to_string(),
            recommended: ram >= 48.0 || (has_gpu && hw.gpu_vram_gb.unwrap_or(0.0) >= 40.0),
        },
    ];

    // Filter out models that don't fit in available RAM (with slight overcommit allowance)
    models.retain(|m| m.min_ram_gb <= ram + 2.0);
    models
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hw(ram: f64, gpu: Option<(&str, f64)>) -> HardwareInfo {
        HardwareInfo {
            total_ram_gb: ram,
            cpu_arch: "x86_64".to_string(),
            gpu_vendor: gpu.map(|(name, _)| name.to_string()),
            gpu_vram_gb: gpu.map(|(_, vram)| vram),
        }
    }

    #[test]
    fn test_low_ram_only_small_models() {
        let models = recommend_models(&hw(4.0, None));
        assert!(models.iter().all(|m| m.min_ram_gb <= 6.0));
        assert!(models.iter().any(|m| m.name == "llama3.2:1b"));
    }

    #[test]
    fn test_low_ram_recommends_1b() {
        let models = recommend_models(&hw(6.0, None));
        let rec = models.iter().find(|m| m.recommended);
        assert!(rec.is_some());
        assert_eq!(rec.unwrap().name, "llama3.2:1b");
    }

    #[test]
    fn test_medium_ram_recommends_3b() {
        let models = recommend_models(&hw(12.0, None));
        let rec: Vec<_> = models.iter().filter(|m| m.recommended).collect();
        assert!(rec.iter().any(|m| m.name == "llama3.2:3b"));
    }

    #[test]
    fn test_high_ram_recommends_8b() {
        let models = recommend_models(&hw(20.0, None));
        let rec: Vec<_> = models.iter().filter(|m| m.recommended).collect();
        assert!(rec.iter().any(|m| m.name == "llama3.1:8b"));
    }

    #[test]
    fn test_very_high_ram_includes_large_models() {
        let models = recommend_models(&hw(50.0, None));
        assert!(models.iter().any(|m| m.name == "llama3.1:70b"));
        assert!(models.iter().any(|m| m.name == "qwen2.5:14b"));
    }

    #[test]
    fn test_gpu_with_high_vram_recommends_70b() {
        let models = recommend_models(&hw(32.0, Some(("NVIDIA RTX 4090", 48.0))));
        let rec: Vec<_> = models.iter().filter(|m| m.recommended).collect();
        assert!(rec.iter().any(|m| m.name == "llama3.1:70b"));
    }

    #[test]
    fn test_no_models_below_minimum() {
        let models = recommend_models(&hw(2.0, None));
        // Only 1b model should be available (min_ram 4.0, with +2.0 tolerance allows it)
        assert!(models.len() <= 2);
    }
}
