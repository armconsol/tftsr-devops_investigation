// Proxmox integration module
// Provides management for Proxmox VE and Proxmox Backup Server clusters

pub mod backup;
pub mod ceph;
pub mod client;
pub mod cluster;
pub mod firewall;
pub mod ha;
pub mod metrics;
pub mod node;
pub mod sdn;
pub mod storage;
pub mod updates;
pub mod vm;

pub use client::ProxmoxClient;
pub use cluster::{ClusterInfo, ClusterRegistry, ClusterType};
