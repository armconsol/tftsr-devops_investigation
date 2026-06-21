// User Management (LDAP/AD/OpenID realms) module
// Provides operations for managing authentication realms

use serde::{Deserialize, Serialize};

/// Authentication realm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRealm {
    pub realm: String,
    pub realm_type: String,
    pub comment: String,
    pub enabled: bool,
}

/// LDAP realm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapRealmConfig {
    pub server: String,
    pub port: u16,
    pub base_dn: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub filter: String,
    pub scope: String,
    pub start_tls: bool,
    pub certificate: String,
}

/// AD realm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdRealmConfig {
    pub server: String,
    pub port: u16,
    pub base_dn: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub filter: String,
    pub scope: String,
    pub use_ssl: bool,
    pub certificate: String,
}

/// OpenID realm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenidRealmConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
    pub mapping: String,
}

/// List authentication realms
pub async fn list_auth_realms(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<AuthRealm>, String> {
    let path = "access/domains";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list authentication realms: {}", e))?;

    if let Some(realms) = response.as_array() {
        let realm_list: Vec<AuthRealm> = realms
            .iter()
            .filter_map(|realm| {
                let name = realm.get("realm")?.as_str()?.to_string();
                let realm_type = realm.get("type")?.as_str()?.to_string();
                let comment = realm
                    .get("comment")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let enabled = realm
                    .get("enable")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(true);

                Some(AuthRealm {
                    realm: name,
                    realm_type,
                    comment,
                    enabled,
                })
            })
            .collect();

        Ok(realm_list)
    } else {
        Ok(vec![])
    }
}

/// Add LDAP realm
pub async fn add_ldap_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &LdapRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("access/domains/{}", realm_id);
    let config_json = serde_json::json!({
        "type": "ldap",
        "server": config.server,
        "port": config.port,
        "basedn": config.base_dn,
        "binddn": config.bind_dn,
        "bindpw": config.bind_password,
        "filter": config.filter,
        "scope": config.scope,
        "starttls": config.start_tls,
        "certificate": config.certificate
    });

    let _response: serde_json::Value = client
        .post(&path, &config_json, Some(ticket))
        .await
        .map_err(|e| format!("Failed to add LDAP realm {}: {}", realm_id, e))?;
    Ok(())
}

/// Add AD realm
pub async fn add_ad_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &AdRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("access/domains/{}", realm_id);
    let config_json = serde_json::json!({
        "type": "ad",
        "server": config.server,
        "port": config.port,
        "basedn": config.base_dn,
        "binddn": config.bind_dn,
        "bindpw": config.bind_password,
        "filter": config.filter,
        "scope": config.scope,
        "ssl": config.use_ssl,
        "certificate": config.certificate
    });

    let _response: serde_json::Value = client
        .post(&path, &config_json, Some(ticket))
        .await
        .map_err(|e| format!("Failed to add AD realm {}: {}", realm_id, e))?;
    Ok(())
}

/// Add OpenID realm
pub async fn add_openid_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &OpenidRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("access/domains/{}", realm_id);
    let config_json = serde_json::json!({
        "type": "openid",
        "issuer": config.issuer,
        "clientid": config.client_id,
        "clientsecret": config.client_secret,
        "redirecturl": config.redirect_url,
        "scopes": config.scopes.join(","),
        "mapping": config.mapping
    });

    let _response: serde_json::Value = client
        .post(&path, &config_json, Some(ticket))
        .await
        .map_err(|e| format!("Failed to add OpenID realm {}: {}", realm_id, e))?;
    Ok(())
}

/// Update LDAP realm
pub async fn update_ldap_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &LdapRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("access/domains/{}", realm_id);
    let config_json = serde_json::json!({
        "server": config.server,
        "port": config.port,
        "basedn": config.base_dn,
        "binddn": config.bind_dn,
        "bindpw": config.bind_password,
        "filter": config.filter,
        "scope": config.scope,
        "starttls": config.start_tls,
        "certificate": config.certificate
    });

    let _response: serde_json::Value = client
        .put(&path, &config_json, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update LDAP realm {}: {}", realm_id, e))?;
    Ok(())
}

/// Update AD realm
pub async fn update_ad_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &AdRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("access/domains/{}", realm_id);
    let config_json = serde_json::json!({
        "server": config.server,
        "port": config.port,
        "basedn": config.base_dn,
        "binddn": config.bind_dn,
        "bindpw": config.bind_password,
        "filter": config.filter,
        "scope": config.scope,
        "ssl": config.use_ssl,
        "certificate": config.certificate
    });

    let _response: serde_json::Value = client
        .put(&path, &config_json, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update AD realm {}: {}", realm_id, e))?;
    Ok(())
}

/// Update OpenID realm
pub async fn update_openid_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &OpenidRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("access/domains/{}", realm_id);
    let config_json = serde_json::json!({
        "issuer": config.issuer,
        "clientid": config.client_id,
        "clientsecret": config.client_secret,
        "redirecturl": config.redirect_url,
        "scopes": config.scopes.join(","),
        "mapping": config.mapping
    });

    let _response: serde_json::Value = client
        .put(&path, &config_json, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update OpenID realm {}: {}", realm_id, e))?;
    Ok(())
}

/// Delete realm
pub async fn delete_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("access/domains/{}", realm_id);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete realm {}: {}", realm_id, e))?;
    Ok(())
}

/// Get realm configuration
pub async fn get_realm_config(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("access/domains/{}", realm_id);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get realm config {}: {}", realm_id, e))
}
