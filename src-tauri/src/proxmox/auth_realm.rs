// User Management (LDAP/AD/OpenID realms) module
// Provides operations for managing authentication realms and user API tokens

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
#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Clone, Serialize, Deserialize)]
pub struct OpenidRealmConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
    pub mapping: String,
}

impl std::fmt::Debug for LdapRealmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LdapRealmConfig")
            .field("server", &self.server)
            .field("port", &self.port)
            .field("base_dn", &self.base_dn)
            .field("bind_dn", &self.bind_dn)
            .field("bind_password", &"[REDACTED]")
            .field("filter", &self.filter)
            .field("scope", &self.scope)
            .field("start_tls", &self.start_tls)
            .field("certificate", &self.certificate)
            .finish()
    }
}

impl std::fmt::Debug for AdRealmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdRealmConfig")
            .field("server", &self.server)
            .field("port", &self.port)
            .field("base_dn", &self.base_dn)
            .field("bind_dn", &self.bind_dn)
            .field("bind_password", &"[REDACTED]")
            .field("filter", &self.filter)
            .field("scope", &self.scope)
            .field("use_ssl", &self.use_ssl)
            .field("certificate", &self.certificate)
            .finish()
    }
}

impl std::fmt::Debug for OpenidRealmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenidRealmConfig")
            .field("issuer", &self.issuer)
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("redirect_url", &self.redirect_url)
            .field("scopes", &self.scopes)
            .field("mapping", &self.mapping)
            .finish()
    }
}

/// Validate a realm id: alphanumeric + hyphens + underscores + dots, max 64 chars.
/// Prevents path traversal / injection when interpolated into access/domains/{realm_id}.
fn validate_realm_id(realm_id: &str) -> Result<(), String> {
    if realm_id.is_empty() || realm_id.len() > 64 {
        return Err(format!(
            "Invalid realm id '{realm_id}': must be 1–64 characters"
        ));
    }
    if !realm_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(format!("Invalid realm id '{realm_id}': only alphanumeric characters, '-', '_', and '.' are allowed"));
    }
    Ok(())
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
        .map_err(|e| format!("Failed to list authentication realms: {e}"))?;

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
    validate_realm_id(realm_id)?;
    let path = format!("access/domains/{realm_id}");
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
        .map_err(|e| format!("Failed to add LDAP realm {realm_id}: {e}"))?;
    Ok(())
}

/// Add AD realm
pub async fn add_ad_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &AdRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    validate_realm_id(realm_id)?;
    let path = format!("access/domains/{realm_id}");
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
        .map_err(|e| format!("Failed to add AD realm {realm_id}: {e}"))?;
    Ok(())
}

/// Add OpenID realm
pub async fn add_openid_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &OpenidRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    validate_realm_id(realm_id)?;
    let path = format!("access/domains/{realm_id}");
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
        .map_err(|e| format!("Failed to add OpenID realm {realm_id}: {e}"))?;
    Ok(())
}

/// Update LDAP realm
pub async fn update_ldap_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &LdapRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    validate_realm_id(realm_id)?;
    let path = format!("access/domains/{realm_id}");
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
        .map_err(|e| format!("Failed to update LDAP realm {realm_id}: {e}"))?;
    Ok(())
}

/// Update AD realm
pub async fn update_ad_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &AdRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    validate_realm_id(realm_id)?;
    let path = format!("access/domains/{realm_id}");
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
        .map_err(|e| format!("Failed to update AD realm {realm_id}: {e}"))?;
    Ok(())
}

/// Update OpenID realm
pub async fn update_openid_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    config: &OpenidRealmConfig,
    ticket: &str,
) -> Result<(), String> {
    validate_realm_id(realm_id)?;
    let path = format!("access/domains/{realm_id}");
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
        .map_err(|e| format!("Failed to update OpenID realm {realm_id}: {e}"))?;
    Ok(())
}

/// Delete realm
pub async fn delete_realm(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    ticket: &str,
) -> Result<(), String> {
    validate_realm_id(realm_id)?;
    let path = format!("access/domains/{realm_id}");
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete realm {realm_id}: {e}"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// User Token Management
// ---------------------------------------------------------------------------

/// An API token associated with a Proxmox user.
///
/// Returned by GET /access/users/{userid}/token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserToken {
    pub tokenid: String,
    pub comment: Option<String>,
    /// Privilege separation flag — 0 = disabled, 1 = enabled.
    pub privsep: Option<u8>,
    pub expire: Option<i64>,
}

/// Result of creating a new user token.
///
/// The `value` field contains the raw token secret and is **only** present
/// on the initial create response — it is never retrievable afterwards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTokenCreateResult {
    #[serde(rename = "full-tokenid")]
    pub full_tokenid: Option<String>,
    pub info: Option<serde_json::Value>,
    /// The token secret (only returned on create).
    pub value: Option<String>,
}

/// Validate a userid: exactly one "@", local part alphanumeric+hyphen+dot,
/// realm part alphanumeric+hyphen.
fn validate_userid(userid: &str) -> Result<(), String> {
    let parts: Vec<&str> = userid.splitn(2, '@').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid userid '{userid}': must contain exactly one '@'"
        ));
    }
    let local = parts[0];
    let realm = parts[1];

    if local.is_empty() || realm.is_empty() {
        return Err(format!(
            "Invalid userid '{userid}': local and realm parts must not be empty"
        ));
    }
    if !local
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '.')
    {
        return Err(format!(
            "Invalid userid '{userid}': local part contains disallowed characters"
        ));
    }
    if !realm.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(format!(
            "Invalid userid '{userid}': realm part contains disallowed characters"
        ));
    }
    Ok(())
}

/// Validate a token name: alphanumeric + hyphens only, max 64 chars.
fn validate_tokenname(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 64 {
        return Err(format!(
            "Invalid token name '{name}': must be 1–64 characters"
        ));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(format!(
            "Invalid token name '{name}': only alphanumeric characters and hyphens allowed"
        ));
    }
    Ok(())
}

/// List all API tokens for a user.
///
/// GET /access/users/{userid}/token
pub async fn list_user_tokens(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    userid: &str,
) -> Result<Vec<UserToken>, String> {
    validate_userid(userid)?;

    let path = format!("access/users/{}/token", urlencoding::encode(userid));

    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list tokens for user '{userid}': {e}"))?;

    match response.as_array() {
        Some(arr) => {
            let tokens: Vec<UserToken> = arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            Ok(tokens)
        }
        None => Ok(vec![]),
    }
}

/// Create a new API token for a user.
///
/// POST /access/users/{userid}/token/{tokenname}
///
/// The returned `UserTokenCreateResult.value` contains the token secret and
/// is only available at creation time.
pub async fn create_user_token(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    userid: &str,
    tokenname: &str,
    comment: Option<&str>,
    privsep: bool,
    expire: Option<i64>,
) -> Result<UserTokenCreateResult, String> {
    validate_userid(userid)?;
    validate_tokenname(tokenname)?;

    let path = format!(
        "access/users/{}/token/{}",
        urlencoding::encode(userid),
        urlencoding::encode(tokenname)
    );

    let privsep_str = if privsep { "1" } else { "0" };
    let expire_str;

    let mut params: Vec<(&str, &str)> = vec![("privsep", privsep_str)];
    if let Some(c) = comment {
        params.push(("comment", c));
    }
    if let Some(exp) = expire {
        expire_str = exp.to_string();
        params.push(("expire", &expire_str));
    }

    client
        .post_form(&path, &params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create token '{tokenname}' for user '{userid}': {e}"))
}

/// Delete an API token for a user.
///
/// DELETE /access/users/{userid}/token/{tokenname}
pub async fn delete_user_token(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    userid: &str,
    tokenname: &str,
) -> Result<(), String> {
    validate_userid(userid)?;
    validate_tokenname(tokenname)?;

    let path = format!(
        "access/users/{}/token/{}",
        urlencoding::encode(userid),
        urlencoding::encode(tokenname)
    );

    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete token '{tokenname}' for user '{userid}': {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------

/// Get realm configuration
pub async fn get_realm_config(
    client: &crate::proxmox::client::ProxmoxClient,
    realm_id: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    validate_realm_id(realm_id)?;
    let path = format!("access/domains/{realm_id}");
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get realm config {realm_id}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- UserToken deserialisation ---

    #[test]
    fn test_user_token_deserialisation_full() {
        let json = r#"{
            "tokenid": "migrate-vm",
            "privsep": 0,
            "expire": 0,
            "comment": "migration token"
        }"#;
        let token: UserToken = serde_json::from_str(json).expect("should deserialise");
        assert_eq!(token.tokenid, "migrate-vm");
        assert_eq!(token.expire, Some(0));
        assert_eq!(token.comment.as_deref(), Some("migration token"));
    }

    #[test]
    fn test_user_token_deserialisation_no_comment() {
        let json = r#"{"tokenid": "pdm-admin-pdm", "privsep": 0, "expire": 0}"#;
        let token: UserToken = serde_json::from_str(json).expect("should deserialise");
        assert_eq!(token.tokenid, "pdm-admin-pdm");
        assert!(token.comment.is_none());
    }

    #[test]
    fn test_user_token_create_result_deserialisation() {
        let json = r#"{
            "full-tokenid": "root@pam!mytoken",
            "value": "supersecretvalue",
            "info": {"privsep": 0, "expire": 0}
        }"#;
        let result: UserTokenCreateResult = serde_json::from_str(json).expect("should deserialise");
        assert_eq!(result.full_tokenid.as_deref(), Some("root@pam!mytoken"));
        assert_eq!(result.value.as_deref(), Some("supersecretvalue"));
        assert!(result.info.is_some());
    }

    // --- validate_userid ---

    #[test]
    fn test_validate_userid_valid() {
        assert!(validate_userid("root@pam").is_ok());
        assert!(validate_userid("admin@pve").is_ok());
        assert!(validate_userid("user.name@ldap-realm").is_ok());
        assert!(validate_userid("user-name@ad").is_ok());
    }

    #[test]
    fn test_validate_userid_path_traversal_rejected() {
        assert!(validate_userid("root/../evil@pam").is_err());
        assert!(validate_userid("root@../../etc").is_err());
        assert!(validate_userid("root@pam/etc").is_err());
    }

    #[test]
    fn test_validate_userid_no_at_rejected() {
        assert!(validate_userid("rootpam").is_err());
    }

    #[test]
    fn test_validate_userid_empty_parts() {
        assert!(validate_userid("@pam").is_err());
        assert!(validate_userid("root@").is_err());
    }

    // --- validate_tokenname ---

    #[test]
    fn test_validate_tokenname_valid() {
        assert!(validate_tokenname("mytoken").is_ok());
        assert!(validate_tokenname("migrate-vm").is_ok());
        assert!(validate_tokenname("TOKEN123").is_ok());
        assert!(validate_tokenname("a".repeat(64).as_str()).is_ok());
    }

    #[test]
    fn test_validate_tokenname_invalid() {
        assert!(validate_tokenname("").is_err());
        assert!(validate_tokenname("a".repeat(65).as_str()).is_err());
        assert!(validate_tokenname("token.with.dot").is_err());
        assert!(validate_tokenname("token/slash").is_err());
        assert!(validate_tokenname("token name").is_err());
        assert!(validate_tokenname("token@pam").is_err());
    }

    // --- Path building ---

    // --- validate_realm_id ---

    #[test]
    fn test_validate_realm_id_valid() {
        assert!(validate_realm_id("pam").is_ok());
        assert!(validate_realm_id("my-ldap").is_ok());
        assert!(validate_realm_id("ad_realm.01").is_ok());
        assert!(validate_realm_id("a".repeat(64).as_str()).is_ok());
    }

    #[test]
    fn test_validate_realm_id_rejects_traversal_and_injection() {
        assert!(validate_realm_id("").is_err());
        assert!(validate_realm_id("a".repeat(65).as_str()).is_err());
        assert!(validate_realm_id("../access/users").is_err());
        assert!(validate_realm_id("realm/sub").is_err());
        assert!(validate_realm_id("realm name").is_err());
        assert!(validate_realm_id("realm?x=1").is_err());
        assert!(validate_realm_id("realm@evil").is_err());
    }

    #[test]
    fn test_realm_config_debug_redacts_secrets() {
        let ldap = LdapRealmConfig {
            server: "ldap.example.com".to_string(),
            port: 636,
            base_dn: "dc=example,dc=com".to_string(),
            bind_dn: "cn=admin".to_string(),
            bind_password: "supersecret".to_string(),
            filter: "(uid=%s)".to_string(),
            scope: "sub".to_string(),
            start_tls: true,
            certificate: "".to_string(),
        };
        let dbg = format!("{ldap:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("supersecret"));

        let openid = OpenidRealmConfig {
            issuer: "https://idp".to_string(),
            client_id: "client".to_string(),
            client_secret: "topsecret".to_string(),
            redirect_url: "https://app/cb".to_string(),
            scopes: vec!["openid".to_string()],
            mapping: "".to_string(),
        };
        let dbg = format!("{openid:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("topsecret"));
    }

    #[test]
    fn test_list_token_path_encoding() {
        let userid = "root@pam";
        let path = format!("access/users/{}/token", urlencoding::encode(userid));
        assert_eq!(path, "access/users/root%40pam/token");
    }

    #[test]
    fn test_create_token_path_encoding() {
        let userid = "root@pam";
        let tokenname = "my-token";
        let path = format!(
            "access/users/{}/token/{}",
            urlencoding::encode(userid),
            urlencoding::encode(tokenname)
        );
        assert_eq!(path, "access/users/root%40pam/token/my-token");
    }
}
