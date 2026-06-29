//! Remote connection types for RDP/SSH tunneling
//! 
//! This module defines types for remote desktop connections with SSH tunnel support.
//! Credentials are stored separately and encrypted at rest using AES-256-GCM.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Remote Connection Types ────────────────────────────────────────────────

/// Represents a remote desktop connection configuration.
/// Sensitive credentials (passwords) are stored in the remote_credentials table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConnection {
    pub id: String,
    pub name: String,
    pub connection_type: String, // "rdp", "vnc", "ssh"
    pub hostname: String,
    pub port: u16,
    pub username: String,
    // SSH tunnel configuration (non-sensitive)
    pub ssh_enabled: bool,
    pub ssh_hostname: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_username: Option<String>,
    // Metadata
    pub description: Option<String>,
    pub color: Option<String>,
    pub resolution: Option<String>, // e.g., "1920x1080"
    pub created_at: String,
    pub updated_at: String,
}

impl RemoteConnection {
    pub fn new(
        name: String,
        connection_type: String,
        hostname: String,
        port: u16,
        username: String,
    ) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        RemoteConnection {
            id: Uuid::now_v7().to_string(),
            name,
            connection_type,
            hostname,
            port,
            username,
            ssh_enabled: false,
            ssh_hostname: None,
            ssh_port: None,
            ssh_username: None,
            description: None,
            color: None,
            resolution: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Enable SSH tunnel for this connection
    pub fn with_ssh_tunnel(
        mut self,
        ssh_hostname: String,
        ssh_port: u16,
        ssh_username: String,
    ) -> Self {
        self.ssh_enabled = true;
        self.ssh_hostname = Some(ssh_hostname);
        self.ssh_port = Some(ssh_port);
        self.ssh_username = Some(ssh_username);
        self
    }
}

/// Input for creating a new remote connection.
/// Password should be provided separately and stored in remote_credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRemoteConnection {
    pub name: String,
    pub connection_type: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String, // Will be encrypted and stored separately
    // SSH tunnel configuration
    pub ssh_enabled: bool,
    pub ssh_hostname: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_username: Option<String>,
    // Optional metadata
    pub description: Option<String>,
    pub color: Option<String>,
    pub resolution: Option<String>,
}

/// Update for an existing remote connection.
/// Password updates require separate credential update.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteConnectionUpdate {
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    // SSH tunnel configuration
    pub ssh_enabled: Option<bool>,
    pub ssh_hostname: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_username: Option<String>,
    // Metadata
    pub description: Option<String>,
    pub color: Option<String>,
    pub resolution: Option<String>,
}

// ─── Remote Credentials ─────────────────────────────────────────────────────

/// Encrypted credentials for remote connections.
/// Stored separately from connection config for enhanced security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCredentials {
    pub id: String,
    pub connection_id: String,
    pub password_encrypted: String, // AES-256-GCM encrypted password
    pub ssh_password_encrypted: Option<String>, // Optional SSH password
    pub ssh_key_encrypted: Option<String>, // Optional SSH private key
    pub created_at: String,
    pub updated_at: String,
}

impl RemoteCredentials {
    pub fn new(connection_id: String, password: String) -> Result<Self, String> {
        use crate::integrations::auth::encrypt_token;
        
        let encrypted_password = encrypt_token(&password)?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        Ok(RemoteCredentials {
            id: Uuid::now_v7().to_string(),
            connection_id,
            password_encrypted: encrypted_password,
            ssh_password_encrypted: None,
            ssh_key_encrypted: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Create credentials with SSH password
    pub fn with_ssh_password(mut self, ssh_password: String) -> Result<Self, String> {
        use crate::integrations::auth::encrypt_token;
        
        self.ssh_password_encrypted = Some(encrypt_token(&ssh_password)?);
        self.updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Ok(self)
    }

    /// Create credentials with SSH key
    pub fn with_ssh_key(mut self, ssh_key: String) -> Result<Self, String> {
        use crate::integrations::auth::encrypt_token;
        
        self.ssh_key_encrypted = Some(encrypt_token(&ssh_key)?);
        self.updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Ok(self)
    }
}

/// Input for creating remote credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRemoteCredentials {
    pub connection_id: String,
    pub password: String,
    pub ssh_password: Option<String>,
    pub ssh_key: Option<String>,
}

impl TryFrom<NewRemoteCredentials> for RemoteCredentials {
    type Error = String;

    fn try_from(new: NewRemoteCredentials) -> Result<Self, Self::Error> {
        use crate::integrations::auth::encrypt_token;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let password_encrypted = encrypt_token(&new.password)?;
        let ssh_password_encrypted = new
            .ssh_password
            .as_deref()
            .map(encrypt_token)
            .transpose()?;
        let ssh_key_encrypted = new.ssh_key.as_deref().map(encrypt_token).transpose()?;

        Ok(RemoteCredentials {
            id: Uuid::now_v7().to_string(),
            connection_id: new.connection_id,
            password_encrypted,
            ssh_password_encrypted,
            ssh_key_encrypted,
            created_at: now.clone(),
            updated_at: now,
        })
    }
}

/// Update for existing remote credentials
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteCredentialsUpdate {
    pub password: Option<String>,
    pub ssh_password: Option<String>,
    pub ssh_key: Option<String>,
}

/// Lightweight summary for listing remote connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConnectionSummary {
    pub id: String,
    pub name: String,
    pub connection_type: String,
    pub hostname: String,
    pub port: u16,
    pub ssh_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Filter for listing remote connections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteConnectionFilter {
    pub connection_type: Option<String>,
    pub ssh_only: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
