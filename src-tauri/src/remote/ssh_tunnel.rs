//! SSH Tunnel implementation for RDP/VNC connections
//!
//! Provides secure SSH tunneling with port forwarding for remote desktop connections.
//! Supports both password authentication and SSH key authentication with ssh-agent.

use anyhow::{Context, Result};
use ssh2::Session as Ssh2Session;
use std::io::{Read, Write};
use std::path::Path;
use tracing::{info, warn};

/// SSH tunnel configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshTunnelConfig {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    /// Path to an SSH private key file on disk (mutually exclusive with
    /// `private_key_data`; `private_key_data` takes precedence when both
    /// are present).
    pub private_key_path: Option<String>,
    /// Raw PEM-encoded private key content loaded from the credentials store.
    /// When set, key authentication is performed in-memory without touching
    /// the filesystem.
    pub private_key_data: Option<String>,
    pub key_passphrase: Option<String>,
}

/// A TCP stream through SSH tunnel
pub struct SshTcpStream {
    channel: ssh2::Channel,
}

impl Read for SshTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.channel.read(buf)
    }
}

impl Write for SshTcpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.channel.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.channel.flush()
    }
}

/// SSH tunnel for remote desktop connections
pub struct SshTunnel {
    config: SshTunnelConfig,
    session_id: String,
    connected: bool,
    session: Option<Ssh2Session>,
    tcp_stream: Option<std::net::TcpStream>,
}

impl SshTunnel {
    /// Create a new SSH tunnel
    pub fn new(config: SshTunnelConfig) -> Self {
        let session_id = uuid::Uuid::now_v7().to_string();
        SshTunnel {
            config,
            session_id,
            connected: false,
            session: None,
            tcp_stream: None,
        }
    }

    /// Connect to the SSH server
    pub async fn connect(&mut self) -> Result<()> {
        info!(
            "Connecting to SSH server: {}:{} as {}",
            self.config.hostname, self.config.port, self.config.username
        );

        // Create TCP connection to SSH server
        let tcp =
            std::net::TcpStream::connect(format!("{}:{}", self.config.hostname, self.config.port))
                .context("Failed to connect to SSH server")?;

        // Create SSH session
        let mut session = Ssh2Session::new()?;
        session.set_tcp_stream(tcp.try_clone()?);
        session.handshake()?;

        // Authenticate
        self.authenticate(&mut session).await?;

        self.session = Some(session);
        info!("SSH tunnel connected: {}", self.session_id);
        self.connected = true;
        Ok(())
    }

    /// Authenticate with the SSH server
    async fn authenticate(&self, session: &mut Ssh2Session) -> Result<()> {
        let auth_methods = session
            .auth_methods(&self.config.username)
            .unwrap_or("password,publickey,keyboard-interactive");

        info!("Available SSH auth methods: {}", auth_methods);

        // Try in-memory SSH key authentication first (key data from credentials store)
        if let Some(ref key_data) = self.config.private_key_data {
            info!("Attempting SSH key authentication with in-memory key data");
            match self.authenticate_with_key_data(session, key_data) {
                Ok(_) => {
                    info!("SSH in-memory key authentication successful");
                    return Ok(());
                }
                Err(e) => {
                    warn!("SSH in-memory key authentication failed: {}", e);
                }
            }
        }

        // Try SSH key authentication from file path if configured
        if let Some(ref key_path) = self.config.private_key_path {
            info!("Attempting SSH key authentication with: {}", key_path);

            if Path::new(key_path).exists() {
                match self.authenticate_with_key(session, key_path) {
                    Ok(_) => {
                        info!("SSH key authentication successful");
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("SSH key authentication failed: {}", e);
                        // Continue to password auth if key fails
                    }
                }
            } else {
                warn!("SSH key file not found: {}", key_path);
            }
        }

        // Try ssh-agent authentication
        info!("Attempting SSH agent authentication");
        if self.authenticate_with_agent(session).is_ok() {
            info!("SSH agent authentication successful");
            return Ok(());
        }
        warn!("SSH agent authentication failed");

        // Try password authentication
        if let Some(ref password) = self.config.password {
            info!("Attempting password authentication");
            session
                .userauth_password(&self.config.username, password)
                .context("Password authentication failed")?;
            info!("Password authentication successful");
            return Ok(());
        }

        Err(anyhow::anyhow!("No authentication method succeeded"))
    }

    /// Authenticate using SSH private key
    fn authenticate_with_key(&self, session: &mut Ssh2Session, key_path: &str) -> Result<()> {
        let key_path = Path::new(key_path);

        if let Some(ref passphrase) = self.config.key_passphrase {
            session
                .userauth_pubkey_file(
                    &self.config.username,
                    None,
                    key_path,
                    Some(passphrase.as_str()),
                )
                .context("Key-based authentication failed")?;
        } else {
            session
                .userauth_pubkey_file(&self.config.username, None, key_path, None)
                .context("Key-based authentication failed")?;
        }

        Ok(())
    }

    /// Authenticate using in-memory SSH private key data (PEM-encoded)
    fn authenticate_with_key_data(&self, session: &mut Ssh2Session, key_data: &str) -> Result<()> {
        let passphrase = self.config.key_passphrase.as_deref();
        session
            .userauth_pubkey_memory(&self.config.username, None, key_data, passphrase)
            .context("In-memory key authentication failed")?;
        Ok(())
    }

    /// Authenticate using SSH agent
    fn authenticate_with_agent(&self, session: &mut Ssh2Session) -> Result<()> {
        // Try to authenticate using keys from ssh-agent
        session
            .userauth_agent(&self.config.username)
            .context("SSH agent authentication failed")?;
        Ok(())
    }

    /// Create a TCP stream through SSH tunnel
    pub async fn create_tcp_stream(
        &self,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<SshTcpStream> {
        if !self.connected {
            return Err(anyhow::anyhow!("SSH tunnel not connected"));
        }

        let session = self
            .session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("SSH session not initialized"))?;

        // Open a direct TCP channel through SSH
        let channel = session
            .channel_direct_tcpip(remote_host, remote_port, None)
            .map_err(|e| anyhow::anyhow!("Failed to open SSH channel: {}", e))?;

        info!(
            "SSH tunnel established to {}:{} via direct TCP",
            remote_host, remote_port
        );

        Ok(SshTcpStream { channel })
    }

    /// Disconnect from the SSH server
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("SSH tunnel disconnect: {}", self.session_id);

        if let Some(ref mut session) = self.session {
            session.disconnect(None, "Bye", None).ok();
        }

        self.tcp_stream.take();
        self.session = None;
        self.connected = false;
        Ok(())
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get the configuration
    pub fn config(&self) -> &SshTunnelConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_tunnel_creation() {
        let config = SshTunnelConfig {
            hostname: "ssh.example.com".to_string(),
            port: 22,
            username: "user".to_string(),
            password: Some("password".to_string()),
            private_key_path: None,
            private_key_data: None,
            key_passphrase: None,
        };

        let tunnel = SshTunnel::new(config);

        assert!(!tunnel.is_connected());
        assert!(!tunnel.session_id.is_empty());
        assert_eq!(tunnel.config.username, "user");
    }

    #[test]
    fn test_ssh_tunnel_with_key() {
        let config = SshTunnelConfig {
            hostname: "ssh.example.com".to_string(),
            port: 22,
            username: "user".to_string(),
            password: None,
            private_key_path: Some("/home/user/.ssh/id_rsa".to_string()),
            private_key_data: None,
            key_passphrase: Some("passphrase".to_string()),
        };

        let tunnel = SshTunnel::new(config);

        assert!(tunnel.config.private_key_path.is_some());
        assert!(tunnel.config.key_passphrase.is_some());
    }

    #[tokio::test]
    #[ignore] // Skip actual SSH connection test - requires real SSH server
    async fn test_ssh_tunnel_connect_disconnect() {
        let config = SshTunnelConfig {
            hostname: "ssh.example.com".to_string(),
            port: 22,
            username: "user".to_string(),
            password: Some("password".to_string()),
            private_key_path: None,
            private_key_data: None,
            key_passphrase: None,
        };

        let mut tunnel = SshTunnel::new(config);

        // Test connect (will fail without real SSH server)
        let result = tunnel.connect().await;
        assert!(result.is_err()); // Expected to fail without real server

        // Test disconnect
        let result = tunnel.disconnect().await;
        assert!(result.is_ok());
    }
}
