// Copyright (c) 2025 Shaun Arman
// MIT License - see LICENSE file for details

//! Remote Desktop connection management module.
//!
//! This module provides functionality for managing RDP and VNC remote desktop connections,
//! including connection storage, retrieval, and management.

pub mod connection;
pub mod rdp;
pub mod types;
pub mod vnc;

pub use connection::*;
pub use types::*;
