// Remote Shell module
// Provides WebSocket-based terminal access to remote nodes

use serde::{Deserialize, Serialize};

/// Shell ticket information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellTicket {
    pub ticket: String,
    pub node: String,
    pub expires: u64,
    pub permissions: Vec<String>,
}

/// Get shell ticket for remote access
pub async fn get_shell_ticket(
    client: &crate::proxmox::client::ProxmoxClient,
    remote: &str,
    ticket: &str,
) -> Result<ShellTicket, String> {
    let path = format!("remotes/{}/shell-ticket", remote);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get shell ticket for remote {}: {}", remote, e))?;

    {
        let data = &response;
        let ticket_value = data
            .get("ticket")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let node = data
            .get("node")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let expires = data.get("expires").and_then(|e| e.as_u64()).unwrap_or(0);

        let permissions: Vec<String> = data
            .get("permissions")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| p.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ShellTicket {
            ticket: ticket_value,
            node,
            expires,
            permissions,
        })
    }
}

/// Validate shell ticket
pub async fn validate_shell_ticket(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<bool, String> {
    let path = "access/ticket";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to validate shell ticket: {}", e))?;

    Ok(!response.is_null())
}

/// Get shell WebSocket URL
pub fn get_shell_ws_url(base_url: &str, remote: &str, ticket: &str) -> String {
    let base = base_url.trim_end_matches('/');
    format!(
        "wss://{}/api2/json/remotes/{}/shell?ticket={}",
        base, remote, ticket
    )
}
