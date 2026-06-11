// Certificate Management module
// Provides operations for managing certificates

use serde::{Deserialize, Serialize};

/// Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub certificate_id: String,
    pub common_name: String,
    pub issuer: String,
    pub serial: String,
    pub not_before: String,
    pub not_after: String,
    pub fingerprint: String,
    pub key_size: u32,
    pub signature_algorithm: String,
    pub san: Vec<String>,
}

/// Certificate chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateChain {
    pub certificates: Vec<Certificate>,
    pub chain_length: u32,
}

/// Upload certificate
pub async fn upload_certificate(
    client: &crate::proxmox::client::ProxmoxClient,
    certificate: &str,
    private_key: &str,
    name: Option<&str>,
    ticket: &str,
) -> Result<Certificate, String> {
    let path = "config/certificate";
    let config = serde_json::json!({
        "certificate": certificate,
        "privatekey": private_key,
        "name": name.unwrap_or("")
    });

    let response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to upload certificate: {}", e))?;

    if let Some(data) = response.get("data") {
        let id = data
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let common_name = data
            .get("common_name")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let issuer = data
            .get("issuer")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let serial = data
            .get("serial")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let not_before = data
            .get("not_before")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let not_after = data
            .get("not_after")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let fingerprint = data
            .get("fingerprint")
            .and_then(|f| f.as_str())
            .unwrap_or("")
            .to_string();
        let key_size = data.get("key_size").and_then(|k| k.as_u64()).unwrap_or(0) as u32;
        let signature_algorithm = data
            .get("signature_algorithm")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        let san: Vec<String> = data
            .get("san")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Certificate {
            certificate_id: id,
            common_name,
            issuer,
            serial,
            not_before,
            not_after,
            fingerprint,
            key_size,
            signature_algorithm,
            san,
        })
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Get certificate details
pub async fn get_certificate(
    client: &crate::proxmox::client::ProxmoxClient,
    cert_id: &str,
    ticket: &str,
) -> Result<Certificate, String> {
    let path = format!("config/certificate/{}", cert_id);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get certificate {}: {}", cert_id, e))?;

    if let Some(data) = response.get("data") {
        let id = data
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let common_name = data
            .get("common_name")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let issuer = data
            .get("issuer")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let serial = data
            .get("serial")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let not_before = data
            .get("not_before")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let not_after = data
            .get("not_after")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let fingerprint = data
            .get("fingerprint")
            .and_then(|f| f.as_str())
            .unwrap_or("")
            .to_string();
        let key_size = data.get("key_size").and_then(|k| k.as_u64()).unwrap_or(0) as u32;
        let signature_algorithm = data
            .get("signature_algorithm")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        let san: Vec<String> = data
            .get("san")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Certificate {
            certificate_id: id,
            common_name,
            issuer,
            serial,
            not_before,
            not_after,
            fingerprint,
            key_size,
            signature_algorithm,
            san,
        })
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// List certificates
pub async fn list_certificates(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<Certificate>, String> {
    let path = "config/certificate";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list certificates: {}", e))?;

    if let Some(certs) = response.get("data").and_then(|d| d.as_array()) {
        let cert_list: Vec<Certificate> = certs
            .iter()
            .filter_map(|cert| {
                let id = cert.get("id")?.as_str()?.to_string();
                let common_name = cert
                    .get("common_name")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let issuer = cert
                    .get("issuer")
                    .and_then(|i| i.as_str())
                    .unwrap_or("")
                    .to_string();
                let serial = cert
                    .get("serial")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let not_before = cert
                    .get("not_before")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let not_after = cert
                    .get("not_after")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let fingerprint = cert
                    .get("fingerprint")
                    .and_then(|f| f.as_str())
                    .unwrap_or("")
                    .to_string();
                let key_size = cert.get("key_size").and_then(|k| k.as_u64()).unwrap_or(0) as u32;
                let signature_algorithm = cert
                    .get("signature_algorithm")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();

                let san: Vec<String> = cert
                    .get("san")
                    .and_then(|s| s.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                Some(Certificate {
                    certificate_id: id,
                    common_name,
                    issuer,
                    serial,
                    not_before,
                    not_after,
                    fingerprint,
                    key_size,
                    signature_algorithm,
                    san,
                })
            })
            .collect();

        Ok(cert_list)
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Delete certificate
pub async fn delete_certificate(
    client: &crate::proxmox::client::ProxmoxClient,
    cert_id: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("config/certificate/{}", cert_id);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete certificate {}: {}", cert_id, e))?;
    Ok(())
}

/// Get node certificates
pub async fn list_node_certificates(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<Certificate>, String> {
    let path = format!("nodes/{}/certificates", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list node certificates for {}: {}", node, e))?;

    if let Some(certs) = response.get("data").and_then(|d| d.as_array()) {
        let cert_list: Vec<Certificate> = certs
            .iter()
            .filter_map(|cert| {
                let id = cert.get("id")?.as_str()?.to_string();
                let common_name = cert
                    .get("common_name")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let issuer = cert
                    .get("issuer")
                    .and_then(|i| i.as_str())
                    .unwrap_or("")
                    .to_string();
                let serial = cert
                    .get("serial")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let not_before = cert
                    .get("not_before")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let not_after = cert
                    .get("not_after")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let fingerprint = cert
                    .get("fingerprint")
                    .and_then(|f| f.as_str())
                    .unwrap_or("")
                    .to_string();
                let key_size = cert.get("key_size").and_then(|k| k.as_u64()).unwrap_or(0) as u32;
                let signature_algorithm = cert
                    .get("signature_algorithm")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();

                let san: Vec<String> = cert
                    .get("san")
                    .and_then(|s| s.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                Some(Certificate {
                    certificate_id: id,
                    common_name,
                    issuer,
                    serial,
                    not_before,
                    not_after,
                    fingerprint,
                    key_size,
                    signature_algorithm,
                    san,
                })
            })
            .collect();

        Ok(cert_list)
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Upload node certificate
pub async fn upload_node_certificate(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    certificate: &str,
    private_key: &str,
    name: Option<&str>,
    ticket: &str,
) -> Result<Certificate, String> {
    let path = format!("nodes/{}/certificates", node);
    let config = serde_json::json!({
        "certificate": certificate,
        "privatekey": private_key,
        "name": name.unwrap_or("")
    });

    let response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to upload node certificate for {}: {}", node, e))?;

    if let Some(data) = response.get("data") {
        let id = data
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let common_name = data
            .get("common_name")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let issuer = data
            .get("issuer")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let serial = data
            .get("serial")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let not_before = data
            .get("not_before")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let not_after = data
            .get("not_after")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let fingerprint = data
            .get("fingerprint")
            .and_then(|f| f.as_str())
            .unwrap_or("")
            .to_string();
        let key_size = data.get("key_size").and_then(|k| k.as_u64()).unwrap_or(0) as u32;
        let signature_algorithm = data
            .get("signature_algorithm")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        let san: Vec<String> = data
            .get("san")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Certificate {
            certificate_id: id,
            common_name,
            issuer,
            serial,
            not_before,
            not_after,
            fingerprint,
            key_size,
            signature_algorithm,
            san,
        })
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}
