// Proxmox integration module
// Provides management for Proxmox VE and Proxmox Backup Server clusters

pub mod acme;
pub mod apt;
pub mod auth_realm;
pub mod backup;
pub mod ceph;
pub mod ceph_cluster;
pub mod certificates;
pub mod client;
pub mod cluster;
pub mod firewall;
pub mod ha;
pub mod metrics;
pub mod migration;
pub mod network;
pub mod node;
pub mod sdn;
pub mod shell;
pub mod storage;
pub mod tasks;
pub mod updates;
pub mod updates_ext;
pub mod views;
pub mod vm;

pub use client::ProxmoxClient;
pub use cluster::{ClusterInfo, ClusterRegistry, ClusterType};
