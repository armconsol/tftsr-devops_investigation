//! Remote desktop types and enums.

use serde::{Deserialize, Serialize};

/// Protocol types for remote desktop connections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Rdp,
    Vnc,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Rdp => write!(f, "rdp"),
            Protocol::Vnc => write!(f, "vnc"),
        }
    }
}

impl std::str::FromStr for Protocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rdp" => Ok(Protocol::Rdp),
            "vnc" => Ok(Protocol::Vnc),
            _ => Err(format!("Invalid protocol: {}", s)),
        }
    }
}

/// Connection status for remote desktop sessions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error(String),
}

/// Resolution settings for remote desktop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Resolution { width, height }
    }

    pub fn from_string(s: &str) -> Self {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() == 2 {
            let width = parts[0].parse::<u32>().unwrap_or(1280);
            let height = parts[1].parse::<u32>().unwrap_or(800);
            Resolution { width, height }
        } else {
            Resolution {
                width: 1280,
                height: 800,
            }
        }
    }
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Resolution {
            width: 1280,
            height: 800,
        }
    }
}

/// RDP-specific connection settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RdpSettings {
    pub color_depth: u32,
    pub enable_clipboard: bool,
    pub enable_drive_redirect: bool,
    pub enable_multi_monitor: bool,
    pub compression_enabled: bool,
    pub quality: u32,
}

/// VNC-specific connection settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VncSettings {
    pub color_depth: u32,
    pub enable_clipboard: bool,
    pub encoding: VncEncoding,
    pub quality: u32,
}

/// VNC encoding types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum VncEncoding {
    Raw,
    CopyRect,
    Rre,
    #[default]
    Zrle,
    Tight,
}

/// Combined settings for remote desktop connections.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteSettings {
    pub rdp: RdpSettings,
    pub vnc: VncSettings,
}
