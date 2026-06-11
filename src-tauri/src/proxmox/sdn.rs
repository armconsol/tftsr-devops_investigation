// SDN (Software-Defined Networking) management module
// Provides operations for managing Proxmox SDN

use serde::{Deserialize, Serialize};

/// EVPN zone information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvpnZone {
    pub zone: String,
    pub asn: u32,
    pub vni: u32,
    pub gateways: Vec<String>,
    pub status: String,
}

/// Virtual network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualNetwork {
    pub vnet: String,
    pub zone: String,
    pub l2vni: u32,
    pub dhcp: bool,
    pub status: String,
}

/// List EVPN zones
pub async fn list_evpn_zones(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<EvpnZone>, String> {
    let path = "cluster/sdn/zones";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list EVPN zones: {}", e))?;

    if let Some(zones) = response.get("data").and_then(|d| d.as_array()) {
        let zone_list: Vec<EvpnZone> = zones
            .iter()
            .filter_map(|zone| {
                let name = zone.get("zone")?.as_str()?.to_string();
                let asn = zone.get("asn")?.as_u64()? as u32;
                let vni = zone.get("vni")?.as_u64()? as u32;
                let gateways: Vec<String> = zone
                    .get("gateways")
                    .and_then(|g| g.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|g| g.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let status = zone.get("status")?.as_str().unwrap_or("unknown").to_string();

                Some(EvpnZone {
                    zone: name,
                    asn,
                    vni,
                    gateways,
                    status,
                })
            })
            .collect();

        Ok(zone_list)
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Create EVPN zone
pub async fn create_evpn_zone(
    client: &crate::proxmox::client::ProxmoxClient,
    zone: &str,
    asn: u32,
    vni: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = "cluster/sdn/zones";
    let config = serde_json::json!({
        "zone": zone,
        "asn": asn,
        "vni": vni
    });

    let _response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create EVPN zone {}: {}", zone, e))?;
    Ok(())
}

/// Update EVPN zone
pub async fn update_evpn_zone(
    client: &crate::proxmox::client::ProxmoxClient,
    zone: &str,
    asn: u32,
    vni: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/sdn/zones/{}", zone);
    let config = serde_json::json!({
        "asn": asn,
        "vni": vni
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update EVPN zone {}: {}", zone, e))?;
    Ok(())
}

/// Delete EVPN zone
pub async fn delete_evpn_zone(
    client: &crate::proxmox::client::ProxmoxClient,
    zone: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/sdn/zones/{}", zone);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete EVPN zone {}: {}", zone, e))?;
    Ok(())
}

/// List virtual networks
pub async fn list_vnets(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<VirtualNetwork>, String> {
    let path = "cluster/sdn/vnets";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list virtual networks: {}", e))?;

    if let Some(vnets) = response.get("data").and_then(|d| d.as_array()) {
        let vnet_list: Vec<VirtualNetwork> = vnets
            .iter()
            .filter_map(|vnet| {
                let name = vnet.get("vnet")?.as_str()?.to_string();
                let zone = vnet.get("zone")?.as_str()?.to_string();
                let l2vni = vnet.get("l2vni")?.as_u64()? as u32;
                let dhcp = vnet.get("dhcp")?.as_bool()?;
                let status = vnet.get("status")?.as_str().unwrap_or("unknown").to_string();

                Some(VirtualNetwork {
                    vnet: name,
                    zone,
                    l2vni,
                    dhcp,
                    status,
                })
            })
            .collect();

        Ok(vnet_list)
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Create virtual network
pub async fn create_vnet(
    client: &crate::proxmox::client::ProxmoxClient,
    vnet: &str,
    zone: &str,
    l2vni: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = "cluster/sdn/vnets";
    let config = serde_json::json!({
        "vnet": vnet,
        "zone": zone,
        "l2vni": l2vni
    });

    let _response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create virtual network {}: {}", vnet, e))?;
    Ok(())
}

/// Update virtual network
pub async fn update_vnet(
    client: &crate::proxmox::client::ProxmoxClient,
    vnet: &str,
    zone: &str,
    l2vni: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/sdn/vnets/{}", vnet);
    let config = serde_json::json!({
        "zone": zone,
        "l2vni": l2vni
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update virtual network {}: {}", vnet, e))?;
    Ok(())
}

/// Delete virtual network
pub async fn delete_vnet(
    client: &crate::proxmox::client::ProxmoxClient,
    vnet: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/sdn/vnets/{}", vnet);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete virtual network {}: {}", vnet, e))?;
    Ok(())
}

/// Get virtual network status
pub async fn get_vnet_status(
    client: &crate::proxmox::client::ProxmoxClient,
    vnet: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("cluster/sdn/vnets/{}/status", vnet);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get virtual network {}: {}", vnet, e))
}

/// List DHCP leases
pub async fn list_dhcp_leases(
    client: &crate::proxmox::client::ProxmoxClient,
    vnet: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let path = format!("cluster/sdn/vnets/{}/dhcp/status", vnet);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list DHCP leases for vnet {}: {}", vnet, e))?;

    if let Some(leases) = response.get("data").and_then(|d| d.as_array()) {
        Ok(leases.to_vec())
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evpn_zone_serialization() {
        let zone = EvpnZone {
            zone: "primary".to_string(),
            asn: 65001,
            vni: 1000,
            gateways: vec!["10.0.0.1".to_string()],
            status: "active".to_string(),
        };

        let json = serde_json::to_string(&zone).unwrap();
        let deserialized: EvpnZone = serde_json::from_str(&json).unwrap();

        assert_eq!(zone.zone, deserialized.zone);
        assert_eq!(zone.status, "active");
    }

    #[test]
    fn test_virtual_network_serialization() {
        let vnet = VirtualNetwork {
            vnet: "vm-network".to_string(),
            zone: "primary".to_string(),
            l2vni: 1000,
            dhcp: true,
            status: "active".to_string(),
        };

        let json = serde_json::to_string(&vnet).unwrap();
        let deserialized: VirtualNetwork = serde_json::from_str(&json).unwrap();

        assert_eq!(vnet.vnet, deserialized.vnet);
        assert_eq!(vnet.dhcp, deserialized.dhcp);
    }
}
