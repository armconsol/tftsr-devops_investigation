// Proxmox Backup Server (PBS) API module
// Shares ProxmoxClient with PVE — the stored URL points at the PBS instance.
// PBS auth is identical to PVE (POST /api2/json/access/ticket).
// PBS API paths differ from PVE paths.

use serde::{Deserialize, Serialize};

/// A PBS datastore as returned by GET /datastore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbsDatastore {
    pub store: String,
    pub path: Option<String>,
    pub total: Option<u64>,
    pub used: Option<u64>,
    pub avail: Option<u64>,
    #[serde(rename = "type")]
    pub store_type: Option<String>,
}

/// A PBS namespace as returned by GET /datastore/{store}/namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbsNamespace {
    pub ns: String,
    pub comment: Option<String>,
}

/// A PBS backup snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PbsSnapshot {
    pub backup_id: String,
    pub backup_time: i64,
    pub backup_type: String,
    pub size: Option<u64>,
    pub files: Option<Vec<serde_json::Value>>,
    pub verify_state: Option<String>,
    pub notes: Option<String>,
}

/// A PBS task record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbsTask {
    pub upid: String,
    pub node: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub status: Option<String>,
    pub starttime: i64,
    pub endtime: Option<i64>,
}

/// Validate a datastore name: alphanumeric + hyphens + underscores, max 64 chars.
fn validate_store(store: &str) -> Result<(), String> {
    if store.is_empty() || store.len() > 64 {
        return Err(format!(
            "Invalid store name '{}': must be 1–64 characters",
            store
        ));
    }
    if !store
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "Invalid store name '{}': only alphanumeric, hyphens, and underscores allowed",
            store
        ));
    }
    Ok(())
}

/// Validate a node name: alphanumeric + hyphens only, max 64 chars.
fn validate_node(node: &str) -> Result<(), String> {
    if node.is_empty() || node.len() > 64 {
        return Err(format!(
            "Invalid node name '{}': must be 1–64 characters",
            node
        ));
    }
    if !node.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(format!(
            "Invalid node name '{}': only alphanumeric characters and hyphens allowed",
            node
        ));
    }
    Ok(())
}

/// List all datastores on a PBS instance.
///
/// GET /datastore
pub async fn list_pbs_datastores(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<PbsDatastore>, String> {
    let response: serde_json::Value = client
        .get("datastore", Some(ticket))
        .await
        .map_err(|e| format!("Failed to list PBS datastores: {}", e))?;

    match response.as_array() {
        Some(arr) => {
            let stores: Vec<PbsDatastore> = arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            Ok(stores)
        }
        None => Ok(vec![]),
    }
}

/// Get the current status of a specific PBS datastore.
///
/// GET /datastore/{store}/status
pub async fn get_pbs_datastore_status(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    store: &str,
) -> Result<serde_json::Value, String> {
    validate_store(store)?;
    let path = format!("datastore/{}/status", store);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get PBS datastore status for '{}': {}", store, e))
}

/// List namespaces within a PBS datastore.
///
/// GET /datastore/{store}/namespace
///
/// Returns an empty list if the PBS instance does not support namespaces (pre-2.4)
/// or if the endpoint returns a 404 / not-implemented error.
pub async fn list_pbs_namespaces(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    store: &str,
) -> Result<Vec<PbsNamespace>, String> {
    validate_store(store)?;
    let path = format!("datastore/{}/namespace", store);

    match client.get::<serde_json::Value>(&path, Some(ticket)).await {
        Ok(response) => match response.as_array() {
            Some(arr) => {
                let ns_list: Vec<PbsNamespace> = arr
                    .iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect();
                Ok(ns_list)
            }
            None => Ok(vec![]),
        },
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            if msg.contains("404")
                || msg.contains("not found")
                || msg.contains("not implemented")
                || msg.contains("unknown path")
            {
                Ok(vec![])
            } else {
                Err(format!(
                    "Failed to list PBS namespaces for store '{}': {}",
                    store, e
                ))
            }
        }
    }
}

/// List backup snapshots in a PBS datastore, optionally filtered by namespace.
///
/// GET /datastore/{store}/snapshots
/// GET /datastore/{store}/snapshots?ns={ns}  (when ns is non-empty)
pub async fn list_pbs_snapshots(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    store: &str,
    ns: &str,
) -> Result<Vec<PbsSnapshot>, String> {
    validate_store(store)?;

    let path = if ns.is_empty() {
        format!("datastore/{}/snapshots", store)
    } else {
        format!(
            "datastore/{}/snapshots?ns={}",
            store,
            urlencoding::encode(ns)
        )
    };

    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list PBS snapshots for store '{}': {}", store, e))?;

    match response.as_array() {
        Some(arr) => {
            let snapshots: Vec<PbsSnapshot> = arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            Ok(snapshots)
        }
        None => Ok(vec![]),
    }
}

/// List tasks on a PBS node.
///
/// GET /nodes/{node}/tasks
///
/// For PBS the node is typically "localhost" or the PBS server's own hostname.
pub async fn list_pbs_tasks(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    node: &str,
) -> Result<Vec<PbsTask>, String> {
    validate_node(node)?;
    let path = format!("nodes/{}/tasks", node);

    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list PBS tasks for node '{}': {}", node, e))?;

    match response.as_array() {
        Some(arr) => {
            let tasks: Vec<PbsTask> = arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            Ok(tasks)
        }
        None => Ok(vec![]),
    }
}

/// Get node status from a PBS instance.
///
/// GET /nodes/{node}/status
pub async fn get_pbs_node_status(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    node: &str,
) -> Result<serde_json::Value, String> {
    validate_node(node)?;
    let path = format!("nodes/{}/status", node);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get PBS node status for '{}': {}", node, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbs_datastore_deserialisation_full() {
        let json = r#"{
            "store": "main",
            "path": "/mnt/datastore/main",
            "total": 10737418240,
            "used": 5368709120,
            "avail": 5368709120,
            "type": "dir"
        }"#;
        let ds: PbsDatastore = serde_json::from_str(json).expect("should deserialise");
        assert_eq!(ds.store, "main");
        assert_eq!(ds.path.as_deref(), Some("/mnt/datastore/main"));
        assert_eq!(ds.total, Some(10737418240));
        assert_eq!(ds.used, Some(5368709120));
        assert_eq!(ds.store_type.as_deref(), Some("dir"));
    }

    #[test]
    fn test_pbs_datastore_deserialisation_minimal() {
        let json = r#"{"store": "backup"}"#;
        let ds: PbsDatastore = serde_json::from_str(json).expect("should deserialise minimal");
        assert_eq!(ds.store, "backup");
        assert!(ds.path.is_none());
        assert!(ds.total.is_none());
    }

    #[test]
    fn test_pbs_snapshot_deserialisation() {
        let json = r#"{
            "backup-id": "vm-100",
            "backup-time": 1700000000,
            "backup-type": "vm",
            "size": 1073741824,
            "verify-state": "ok",
            "notes": "daily backup"
        }"#;
        let snap: PbsSnapshot = serde_json::from_str(json).expect("should deserialise snapshot");
        assert_eq!(snap.backup_id, "vm-100");
        assert_eq!(snap.backup_time, 1700000000);
        assert_eq!(snap.backup_type, "vm");
        assert_eq!(snap.size, Some(1073741824));
        assert_eq!(snap.verify_state.as_deref(), Some("ok"));
        assert_eq!(snap.notes.as_deref(), Some("daily backup"));
    }

    #[test]
    fn test_pbs_snapshot_deserialisation_minimal() {
        let json = r#"{
            "backup-id": "ct-101",
            "backup-time": 1700001000,
            "backup-type": "ct"
        }"#;
        let snap: PbsSnapshot = serde_json::from_str(json).expect("should deserialise minimal");
        assert_eq!(snap.backup_id, "ct-101");
        assert!(snap.size.is_none());
        assert!(snap.verify_state.is_none());
    }

    #[test]
    fn test_pbs_task_deserialisation() {
        let json = r#"{
            "upid": "PBSUPID:localhost:00001234:abcdef:backup:main:root@pam:",
            "node": "localhost",
            "type": "backup",
            "status": "OK",
            "starttime": 1700000000,
            "endtime": 1700000060
        }"#;
        let task: PbsTask = serde_json::from_str(json).expect("should deserialise task");
        assert_eq!(task.node, "localhost");
        assert_eq!(task.task_type, "backup");
        assert_eq!(task.status.as_deref(), Some("OK"));
        assert_eq!(task.endtime, Some(1700000060));
    }

    #[test]
    fn test_validate_store_valid() {
        assert!(validate_store("main").is_ok());
        assert!(validate_store("backup-store").is_ok());
        assert!(validate_store("data_store_01").is_ok());
        assert!(validate_store("a".repeat(64).as_str()).is_ok());
    }

    #[test]
    fn test_validate_store_invalid() {
        assert!(validate_store("").is_err());
        assert!(validate_store("a".repeat(65).as_str()).is_err());
        assert!(validate_store("store/path").is_err());
        assert!(validate_store("store name").is_err());
        assert!(validate_store("../evil").is_err());
        assert!(validate_store("store@bad").is_err());
    }

    #[test]
    fn test_validate_node_valid() {
        assert!(validate_node("localhost").is_ok());
        assert!(validate_node("pbs-node01").is_ok());
        assert!(validate_node("PBS1").is_ok());
        assert!(validate_node("a".repeat(64).as_str()).is_ok());
    }

    #[test]
    fn test_validate_node_invalid() {
        assert!(validate_node("").is_err());
        assert!(validate_node("a".repeat(65).as_str()).is_err());
        assert!(validate_node("node.name").is_err());
        assert!(validate_node("node_name").is_err());
        assert!(validate_node("node/path").is_err());
        assert!(validate_node("node name").is_err());
    }

    #[test]
    fn test_namespace_list_empty_on_404_pattern() {
        // Simulate the error message that would come back for a 404
        let err_msg = "API request failed with status 404: not found";
        let lower = err_msg.to_lowercase();
        assert!(
            lower.contains("404") || lower.contains("not found"),
            "Should detect 404 in error"
        );
    }

    #[test]
    fn test_snapshot_path_with_namespace() {
        let store = "main";
        let ns = "my-namespace";
        let path = format!(
            "datastore/{}/snapshots?ns={}",
            store,
            urlencoding::encode(ns)
        );
        assert_eq!(path, "datastore/main/snapshots?ns=my-namespace");
    }

    #[test]
    fn test_snapshot_path_without_namespace() {
        let store = "main";
        let ns = "";
        let path = if ns.is_empty() {
            format!("datastore/{}/snapshots", store)
        } else {
            format!(
                "datastore/{}/snapshots?ns={}",
                store,
                urlencoding::encode(ns)
            )
        };
        assert_eq!(path, "datastore/main/snapshots");
    }
}
