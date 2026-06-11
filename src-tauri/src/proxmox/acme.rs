// ACME/Let's Encrypt certificate management module
// Provides operations for managing ACME certificates

use serde::{Deserialize, Serialize};

/// ACME account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcmeAccount {
    pub account_id: String,
    pub email: String,
    pub status: String,
    pub created_at: String,
}

/// ACME challenge information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcmeChallenge {
    pub challenge_id: String,
    pub challenge_type: String,
    pub domain: String,
    pub status: String,
    pub url: String,
    pub token: String,
}

/// ACME certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcmeCertificate {
    pub certificate_id: String,
    pub domains: Vec<String>,
    pub status: String,
    pub expires_at: String,
    pub issuer: String,
}

/// List ACME accounts
pub async fn list_acme_accounts(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<AcmeAccount>, String> {
    let path = "config/acme/accounts";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list ACME accounts: {}", e))?;

    if let Some(accounts) = response.get("data").and_then(|d| d.as_array()) {
        let account_list: Vec<AcmeAccount> = accounts
            .iter()
            .filter_map(|account| {
                let id = account.get("id")?.as_str()?.to_string();
                let email = account.get("email")?.as_str().unwrap_or("").to_string();
                let status = account
                    .get("status")?
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                let created_at = account
                    .get("created")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();

                Some(AcmeAccount {
                    account_id: id,
                    email,
                    status,
                    created_at,
                })
            })
            .collect();

        Ok(account_list)
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Register ACME account
pub async fn register_acme_account(
    client: &crate::proxmox::client::ProxmoxClient,
    email: &str,
    terms_of_service_agreed: bool,
    ticket: &str,
) -> Result<AcmeAccount, String> {
    let path = "config/acme/accounts";
    let config = serde_json::json!({
        "email": email,
        "terms_of_service_agreed": terms_of_service_agreed
    });

    let response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to register ACME account: {}", e))?;

    if let Some(data) = response.get("data") {
        let id = data
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let status = data
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        let created_at = data
            .get("created")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        Ok(AcmeAccount {
            account_id: id,
            email: email.to_string(),
            status,
            created_at,
        })
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Get ACME challenges for domain
pub async fn get_acme_challenges(
    client: &crate::proxmox::client::ProxmoxClient,
    domain: &str,
    ticket: &str,
) -> Result<Vec<AcmeChallenge>, String> {
    let path = format!("config/acme/challenges/{}", domain);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get ACME challenges for {}: {}", domain, e))?;

    if let Some(challenges) = response.get("data").and_then(|d| d.as_array()) {
        let challenge_list: Vec<AcmeChallenge> = challenges
            .iter()
            .filter_map(|challenge| {
                let id = challenge.get("id")?.as_str()?.to_string();
                let challenge_type = challenge.get("type")?.as_str()?.to_string();
                let status = challenge
                    .get("status")?
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                let url = challenge
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let token = challenge
                    .get("token")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();

                Some(AcmeChallenge {
                    challenge_id: id,
                    challenge_type,
                    domain: domain.to_string(),
                    status,
                    url,
                    token,
                })
            })
            .collect();

        Ok(challenge_list)
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Request ACME certificate
pub async fn request_certificate(
    client: &crate::proxmox::client::ProxmoxClient,
    domains: &[&str],
    account_id: &str,
    ticket: &str,
) -> Result<AcmeCertificate, String> {
    let path = "config/acme/certificates";
    let config = serde_json::json!({
        "domains": domains,
        "account": account_id
    });

    let response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to request ACME certificate: {}", e))?;

    if let Some(data) = response.get("data") {
        let id = data
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let status = data
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        let expires_at = data
            .get("expires")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .to_string();
        let issuer = data
            .get("issuer")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();

        let domains: Vec<String> = data
            .get("domains")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(AcmeCertificate {
            certificate_id: id,
            domains,
            status,
            expires_at,
            issuer,
        })
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Get ACME certificate details
pub async fn get_certificate_details(
    client: &crate::proxmox::client::ProxmoxClient,
    cert_id: &str,
    ticket: &str,
) -> Result<AcmeCertificate, String> {
    let path = format!("config/acme/certificates/{}", cert_id);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get ACME certificate {}: {}", cert_id, e))?;

    if let Some(data) = response.get("data") {
        let id = data
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let status = data
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        let expires_at = data
            .get("expires")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .to_string();
        let issuer = data
            .get("issuer")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();

        let domains: Vec<String> = data
            .get("domains")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(AcmeCertificate {
            certificate_id: id,
            domains,
            status,
            expires_at,
            issuer,
        })
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Revoke ACME certificate
pub async fn revoke_certificate(
    client: &crate::proxmox::client::ProxmoxClient,
    cert_id: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("config/acme/certificates/{}", cert_id);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to revoke ACME certificate {}: {}", cert_id, e))?;
    Ok(())
}
