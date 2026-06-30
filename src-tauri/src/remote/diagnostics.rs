// Copyright (c) 2025 Shaun Arman
// MIT License - see LICENSE file for details

//! Diagnostic and health monitoring for RDP sessions.
//!
//! Provides structured diagnostics for troubleshooting blank screens and
//! connection issues, including frame statistics, WebSocket state, and
//! connection health metrics.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Comprehensive diagnostics for an RDP session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RdpDiagnostics {
    pub session_id: String,
    pub connection_state: ConnectionState,
    pub frame_stats: FrameStatistics,
    pub websocket_state: WebSocketState,
    pub health: ConnectionHealth,
    pub timestamp: u64,
}

/// Current state of the RDP connection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

/// Statistics about frame delivery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameStatistics {
    pub frames_sent: u64,
    pub frames_received: u64,
    pub last_frame_timestamp: u64,
    pub last_frame_width: u32,
    pub last_frame_height: u32,
    pub total_bytes_sent: u64,
    pub frame_stall_detected: bool,
}

/// State of the WebSocket connection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebSocketState {
    pub connected: bool,
    pub session_registered: bool,
    pub last_message_timestamp: u64,
}

/// Overall health assessment of the connection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionHealth {
    Healthy,
    Degraded,
    Stalled,
    Failed,
}

#[allow(clippy::derivable_impls)]
impl Default for RdpDiagnostics {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            connection_state: ConnectionState::Disconnected,
            frame_stats: FrameStatistics::default(),
            websocket_state: WebSocketState::default(),
            health: ConnectionHealth::Healthy,
            timestamp: current_timestamp(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for FrameStatistics {
    fn default() -> Self {
        Self {
            frames_sent: 0,
            frames_received: 0,
            last_frame_timestamp: 0,
            last_frame_width: 0,
            last_frame_height: 0,
            total_bytes_sent: 0,
            frame_stall_detected: false,
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for WebSocketState {
    fn default() -> Self {
        Self {
            connected: false,
            session_registered: false,
            last_message_timestamp: 0,
        }
    }
}

impl RdpDiagnostics {
    /// Create new diagnostics for a session.
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            ..Default::default()
        }
    }

    /// Update connection state.
    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.connection_state = state;
        self.timestamp = current_timestamp();
    }

    /// Record a frame being sent.
    pub fn record_frame_sent(&mut self, width: u32, height: u32, bytes: u64) {
        self.frame_stats.frames_sent += 1;
        self.frame_stats.last_frame_timestamp = current_timestamp();
        self.frame_stats.last_frame_width = width;
        self.frame_stats.last_frame_height = height;
        self.frame_stats.total_bytes_sent += bytes;
        self.frame_stats.frame_stall_detected = false;
        self.timestamp = current_timestamp();
    }

    /// Record a frame being received by the frontend.
    pub fn record_frame_received(&mut self) {
        self.frame_stats.frames_received += 1;
        self.timestamp = current_timestamp();
    }

    /// Update WebSocket connection state.
    pub fn set_websocket_state(&mut self, connected: bool, registered: bool) {
        self.websocket_state.connected = connected;
        self.websocket_state.session_registered = registered;
        self.websocket_state.last_message_timestamp = current_timestamp();
        self.timestamp = current_timestamp();
    }

    /// Detect if frames have stalled (no frames for > 5 seconds).
    pub fn detect_frame_stall(&mut self, stall_threshold_secs: u64) {
        let now = current_timestamp();
        let last_frame_age = now.saturating_sub(self.frame_stats.last_frame_timestamp);

        // Only consider it a stall if we've sent frames but none recently
        if self.frame_stats.frames_sent > 0 && last_frame_age > stall_threshold_secs {
            self.frame_stats.frame_stall_detected = true;
        }

        self.timestamp = now;
    }

    /// Compute overall connection health.
    pub fn compute_health(&mut self) -> ConnectionHealth {
        let health = match self.connection_state {
            ConnectionState::Disconnected => ConnectionHealth::Failed,
            ConnectionState::Failed => ConnectionHealth::Failed,
            ConnectionState::Connecting => ConnectionHealth::Degraded,
            ConnectionState::Connected => {
                if self.frame_stats.frame_stall_detected {
                    ConnectionHealth::Stalled
                } else if !self.websocket_state.connected {
                    ConnectionHealth::Degraded
                } else if self.frame_stats.frames_sent > 0 && self.frame_stats.frames_received == 0
                {
                    // Frames being sent but none received = potential rendering issue
                    ConnectionHealth::Degraded
                } else {
                    ConnectionHealth::Healthy
                }
            }
        };

        self.health = health;
        self.timestamp = current_timestamp();
        health
    }
}

/// Get current Unix timestamp in seconds.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Validate frame dimensions are reasonable.
pub fn validate_frame_dimensions(width: u32, height: u32) -> Result<(), String> {
    const MAX_DIMENSION: u32 = 8192;

    if width == 0 {
        return Err("Frame width cannot be zero".to_string());
    }
    if height == 0 {
        return Err("Frame height cannot be zero".to_string());
    }
    if width > MAX_DIMENSION {
        return Err(format!(
            "Frame width {} exceeds maximum {}",
            width, MAX_DIMENSION
        ));
    }
    if height > MAX_DIMENSION {
        return Err(format!(
            "Frame height {} exceeds maximum {}",
            height, MAX_DIMENSION
        ));
    }

    Ok(())
}

/// Validate frame data size matches expected dimensions.
pub fn validate_frame_data_size(data_len: usize, width: u32, height: u32) -> Result<(), String> {
    let expected = (width as usize) * (height as usize) * 4; // RGBA = 4 bytes per pixel

    if data_len < expected {
        return Err(format!(
            "Frame data size {} is less than expected {} for {}x{} RGBA",
            data_len, expected, width, height
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_capture_session_state() {
        let mut diag = RdpDiagnostics::new("test-session-123");

        assert_eq!(diag.session_id, "test-session-123");
        assert_eq!(diag.connection_state, ConnectionState::Disconnected);

        diag.set_connection_state(ConnectionState::Connecting);
        assert_eq!(diag.connection_state, ConnectionState::Connecting);

        diag.set_connection_state(ConnectionState::Connected);
        assert_eq!(diag.connection_state, ConnectionState::Connected);
    }

    #[test]
    fn test_diagnostics_frame_statistics() {
        let mut diag = RdpDiagnostics::new("test-session");

        assert_eq!(diag.frame_stats.frames_sent, 0);
        assert_eq!(diag.frame_stats.total_bytes_sent, 0);

        diag.record_frame_sent(1920, 1080, 8_294_400);

        assert_eq!(diag.frame_stats.frames_sent, 1);
        assert_eq!(diag.frame_stats.last_frame_width, 1920);
        assert_eq!(diag.frame_stats.last_frame_height, 1080);
        assert_eq!(diag.frame_stats.total_bytes_sent, 8_294_400);
        assert!(diag.frame_stats.last_frame_timestamp > 0);

        diag.record_frame_sent(1920, 1080, 8_294_400);
        assert_eq!(diag.frame_stats.frames_sent, 2);
        assert_eq!(diag.frame_stats.total_bytes_sent, 16_588_800);
    }

    #[test]
    fn test_diagnostics_connection_health() {
        let mut diag = RdpDiagnostics::new("test-session");

        // Disconnected = Failed health
        assert_eq!(diag.compute_health(), ConnectionHealth::Failed);

        // Connecting = Degraded health
        diag.set_connection_state(ConnectionState::Connecting);
        assert_eq!(diag.compute_health(), ConnectionHealth::Degraded);

        // Connected with WebSocket = Healthy
        diag.set_connection_state(ConnectionState::Connected);
        diag.set_websocket_state(true, true);
        assert_eq!(diag.compute_health(), ConnectionHealth::Healthy);

        // Frames sent but none received = Degraded (potential rendering issue)
        diag.record_frame_sent(1920, 1080, 8_294_400);
        assert_eq!(diag.compute_health(), ConnectionHealth::Degraded);

        // Frame received = Healthy
        diag.record_frame_received();
        assert_eq!(diag.compute_health(), ConnectionHealth::Healthy);
    }

    #[test]
    fn test_frame_stall_detection() {
        let mut diag = RdpDiagnostics::new("test-session");

        // No stall initially (no frames sent yet)
        diag.detect_frame_stall(5);
        assert!(!diag.frame_stats.frame_stall_detected);

        // Send a frame, check immediately - no stall
        diag.record_frame_sent(1920, 1080, 8_294_400);
        diag.detect_frame_stall(5);
        assert!(!diag.frame_stats.frame_stall_detected);

        // Simulate old frame timestamp (10 seconds ago)
        diag.frame_stats.last_frame_timestamp = current_timestamp() - 10;
        diag.detect_frame_stall(5);
        assert!(diag.frame_stats.frame_stall_detected);

        // Sending new frame clears stall flag
        diag.record_frame_sent(1920, 1080, 8_294_400);
        assert!(!diag.frame_stats.frame_stall_detected);
    }

    #[test]
    fn test_validate_frame_dimensions() {
        // Valid dimensions
        assert!(validate_frame_dimensions(1920, 1080).is_ok());
        assert!(validate_frame_dimensions(1, 1).is_ok());
        assert!(validate_frame_dimensions(8192, 8192).is_ok());

        // Invalid dimensions
        assert!(validate_frame_dimensions(0, 1080).is_err());
        assert!(validate_frame_dimensions(1920, 0).is_err());
        assert!(validate_frame_dimensions(0, 0).is_err());
        assert!(validate_frame_dimensions(9000, 1080).is_err());
        assert!(validate_frame_dimensions(1920, 9000).is_err());
    }

    #[test]
    fn test_validate_frame_data_size() {
        // 1920x1080 RGBA = 8,294,400 bytes
        let expected = 1920 * 1080 * 4;

        // Exact size
        assert!(validate_frame_data_size(expected, 1920, 1080).is_ok());

        // Larger is OK (header + padding)
        assert!(validate_frame_data_size(expected + 100, 1920, 1080).is_ok());

        // Too small = error
        assert!(validate_frame_data_size(expected - 1, 1920, 1080).is_err());
        assert!(validate_frame_data_size(0, 1920, 1080).is_err());
    }

    #[test]
    fn test_websocket_state_tracking() {
        let mut diag = RdpDiagnostics::new("test-session");

        assert!(!diag.websocket_state.connected);
        assert!(!diag.websocket_state.session_registered);

        diag.set_websocket_state(true, false);
        assert!(diag.websocket_state.connected);
        assert!(!diag.websocket_state.session_registered);

        diag.set_websocket_state(true, true);
        assert!(diag.websocket_state.connected);
        assert!(diag.websocket_state.session_registered);
        assert!(diag.websocket_state.last_message_timestamp > 0);
    }

    #[test]
    fn test_stalled_connection_health() {
        let mut diag = RdpDiagnostics::new("test-session");

        diag.set_connection_state(ConnectionState::Connected);
        diag.set_websocket_state(true, true);
        diag.record_frame_sent(1920, 1080, 8_294_400);

        // Simulate stall
        diag.frame_stats.last_frame_timestamp = current_timestamp() - 10;
        diag.detect_frame_stall(5);

        assert_eq!(diag.compute_health(), ConnectionHealth::Stalled);
    }
}
