pub mod hardware;
pub mod installer;
pub mod manager;
pub mod recommender;

pub use hardware::*;
pub use installer::*;
pub use recommender::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallGuide {
    pub platform: String,
    pub steps: Vec<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub name: String,
    pub size: String,
    pub min_ram_gb: f64,
    pub description: String,
    pub recommended: bool,
}
