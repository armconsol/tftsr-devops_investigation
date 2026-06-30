//! Remote connection management module
//!
//! Provides functionality for managing RDP, VNC, and SSH connections
//! with support for SSH tunneling.

pub mod connection;
pub mod input;
pub mod rdp;
pub mod rdp_client;
pub mod ssh_tunnel;
pub mod types;
pub mod websocket_server;

// Re-export types for convenience
pub use types::*;
