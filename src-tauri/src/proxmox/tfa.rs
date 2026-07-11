// TFA (Two-Factor Authentication) management module
// Provides operations for managing Proxmox TFA entries

use serde::{Deserialize, Serialize};

/// TFA entry as returned by GET /access/tfa
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TfaEntry {
    pub id: String,
    pub userid: String,
    #[serde(rename = "type")]
    pub tfa_type: String,
    pub description: Option<String>,
    pub enable: Option<bool>,
    pub created: Option<i64>,
}

/// Validate a userid: must contain exactly one "@", local part is alphanumeric+hyphen+dot,
/// realm part is alphanumeric+hyphen.
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

/// Validate a TFA entry id: alphanumeric only.
fn validate_tfa_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("TFA id must not be empty".to_string());
    }
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(format!(
            "Invalid TFA id '{id}': must contain only alphanumeric characters and hyphens"
        ));
    }
    Ok(())
}

/// List all TFA entries visible to the authenticated user.
///
/// GET /access/tfa
pub async fn list_tfa_entries(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<TfaEntry>, String> {
    let response: serde_json::Value = client
        .get("access/tfa", Some(ticket))
        .await
        .map_err(|e| format!("Failed to list TFA entries: {e}"))?;

    match response.as_array() {
        Some(arr) => {
            let entries: Vec<TfaEntry> = arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            Ok(entries)
        }
        None => Ok(vec![]),
    }
}

/// Add a TFA entry for a user.
///
/// POST /access/tfa/{userid}
///
/// `tfa_type` is the TFA type string (e.g. "totp", "webauthn", "recovery", "yubico").
/// Optional fields are only included in the form when `Some`.
#[allow(clippy::too_many_arguments)]
pub async fn add_tfa_entry(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    userid: &str,
    tfa_type: &str,
    description: Option<&str>,
    totp: Option<&str>,
    value: Option<&str>,
    key: Option<&str>,
) -> Result<serde_json::Value, String> {
    validate_userid(userid)?;

    let path = format!("access/tfa/{}", urlencoding::encode(userid));

    let mut params: Vec<(&str, &str)> = vec![("type", tfa_type)];
    if let Some(d) = description {
        params.push(("description", d));
    }
    if let Some(t) = totp {
        params.push(("totp", t));
    }
    if let Some(v) = value {
        params.push(("value", v));
    }
    if let Some(k) = key {
        params.push(("key", k));
    }

    client
        .post_form(&path, &params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to add TFA entry for user '{userid}': {e}"))
}

/// Delete a specific TFA entry for a user.
///
/// DELETE /access/tfa/{userid}/{id}
pub async fn delete_tfa_entry(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
    userid: &str,
    id: &str,
) -> Result<(), String> {
    validate_userid(userid)?;
    validate_tfa_id(id)?;

    let path = format!(
        "access/tfa/{}/{}",
        urlencoding::encode(userid),
        urlencoding::encode(id)
    );

    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete TFA entry '{id}' for user '{userid}': {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tfa_entry_deserialisation() {
        let json = r#"{
            "id": "totp-123",
            "userid": "root@pam",
            "type": "totp",
            "description": "My authenticator",
            "enable": true,
            "created": 1700000000
        }"#;
        let entry: TfaEntry = serde_json::from_str(json).expect("should deserialise");
        assert_eq!(entry.id, "totp-123");
        assert_eq!(entry.userid, "root@pam");
        assert_eq!(entry.tfa_type, "totp");
        assert_eq!(entry.description.as_deref(), Some("My authenticator"));
        assert_eq!(entry.enable, Some(true));
        assert_eq!(entry.created, Some(1700000000));
    }

    #[test]
    fn test_tfa_entry_deserialisation_minimal() {
        let json = r#"{
            "id": "abc123",
            "userid": "admin@pve",
            "type": "webauthn"
        }"#;
        let entry: TfaEntry = serde_json::from_str(json).expect("should deserialise minimal");
        assert_eq!(entry.id, "abc123");
        assert_eq!(entry.tfa_type, "webauthn");
        assert!(entry.description.is_none());
        assert!(entry.enable.is_none());
        assert!(entry.created.is_none());
    }

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
    fn test_validate_userid_multiple_at_signs() {
        // splitn(2, '@') means "root@pam@extra" would give local="root", realm="pam@extra"
        // realm "pam@extra" contains '@' which is not alphanumeric/hyphen — rejected correctly
        assert!(validate_userid("root@pam@extra").is_err());
    }

    #[test]
    fn test_validate_userid_empty_parts() {
        assert!(validate_userid("@pam").is_err());
        assert!(validate_userid("root@").is_err());
    }

    #[test]
    fn test_validate_tfa_id_valid() {
        assert!(validate_tfa_id("totp123").is_ok());
        assert!(validate_tfa_id("abc-def").is_ok());
        assert!(validate_tfa_id("ABC123").is_ok());
    }

    #[test]
    fn test_validate_tfa_id_invalid() {
        assert!(validate_tfa_id("").is_err());
        assert!(validate_tfa_id("id/with/slash").is_err());
        assert!(validate_tfa_id("id..traversal").is_err());
        assert!(validate_tfa_id("id with space").is_err());
    }

    #[test]
    fn test_path_building_uses_encoded_userid() {
        // Verify that a userid with special chars would be encoded in the path.
        // We test the encoding logic itself since we can't call the async function
        // without a real client.
        let userid = "user@pam";
        let encoded = urlencoding::encode(userid).to_string();
        let path = format!("access/tfa/{encoded}");
        assert_eq!(path, "access/tfa/user%40pam");
    }

    #[test]
    fn test_path_building_delete() {
        let userid = "root@pam";
        let id = "totp-abc123";
        let path = format!(
            "access/tfa/{}/{}",
            urlencoding::encode(userid),
            urlencoding::encode(id)
        );
        assert_eq!(path, "access/tfa/root%40pam/totp-abc123");
    }
}
