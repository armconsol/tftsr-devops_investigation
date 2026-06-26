// Input validation helpers for Proxmox path-interpolated identifiers.
// All node names flow into URL paths via format!("nodes/{node}/...");
// vmids flow into URL paths as decimal integers. A single validation
// choke-point here prevents traversal and injection in every caller.

pub fn validate_node(node: &str) -> Result<(), String> {
    if node.is_empty() || node.len() > 64 {
        return Err("node must be between 1 and 64 characters".to_string());
    }
    if !node.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(
            "node contains invalid characters — only alphanumeric and '-' are allowed".to_string(),
        );
    }
    Ok(())
}

pub fn validate_vmid(vmid: u32) -> Result<(), String> {
    if !(100..=999_999_999).contains(&vmid) {
        return Err("vmid must be between 100 and 999999999".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_node_valid() {
        assert!(validate_node("pve-node-1").is_ok());
        assert!(validate_node("node01").is_ok());
        assert!(validate_node("a").is_ok());
        assert!(validate_node(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn test_validate_node_invalid_empty() {
        assert!(validate_node("").is_err());
    }

    #[test]
    fn test_validate_node_invalid_too_long() {
        assert!(validate_node(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_validate_node_invalid_path_separators() {
        assert!(validate_node("node/evil").is_err());
        assert!(validate_node("node\\evil").is_err());
        assert!(validate_node("node.evil").is_err());
        assert!(validate_node("node..evil").is_err());
    }

    #[test]
    fn test_validate_node_invalid_url_encoded() {
        // '%' is not in the allowlist — URL-encoded traversal is rejected at the raw byte level
        assert!(validate_node("node%2fevil").is_err());
        assert!(validate_node("%2e%2e").is_err());
    }

    #[test]
    fn test_validate_node_invalid_control_chars() {
        assert!(validate_node("node\0evil").is_err());
        assert!(validate_node("node\revil").is_err());
        assert!(validate_node("node\nevil").is_err());
    }

    #[test]
    fn test_validate_node_invalid_space_and_specials() {
        assert!(validate_node("node evil").is_err());
        assert!(validate_node("node@realm").is_err());
        assert!(validate_node("node:port").is_err());
    }

    #[test]
    fn test_validate_vmid_valid() {
        assert!(validate_vmid(100).is_ok());
        assert!(validate_vmid(200).is_ok());
        assert!(validate_vmid(999_999_999).is_ok());
    }

    #[test]
    fn test_validate_vmid_invalid() {
        assert!(validate_vmid(0).is_err());
        assert!(validate_vmid(99).is_err());
        assert!(validate_vmid(1_000_000_000).is_err());
    }
}
