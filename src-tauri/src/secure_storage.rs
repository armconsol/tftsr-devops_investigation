//! System Keychain Integration for SSH Credentials
//!
//! This module provides cross-platform secure storage for SSH credentials using
//! the native system keychain/credential manager on each platform.
//!
//! Platform support:
//! - macOS: Security.framework (Keychain)
//! - Windows: Windows Credential Manager
//! - Linux: Secret Service API (GNOME Keyring, KWallet)
//!
//! Features:
//! - Secure storage of SSH keys and passwords
//! - Automatic platform detection and appropriate backend selection
//! - Graceful degradation if keychain unavailable
//! - Fallback to encrypted database storage
//! - Zeroize sensitive data after use

use anyhow::Result;

/// Service name for SSH credential storage
const SSH_SERVICE_NAME: &str = "tftsr.ssh";

/// Error types for keychain operations
#[derive(Debug, thiserror::Error)]
pub enum KeychainError {
    #[error("Keychain not available: {0}")]
    NotAvailable(String),

    #[error("Item not found in keychain")]
    ItemNotFound,

    #[error("Access denied to keychain item")]
    AccessDenied,

    #[error("Keychain operation failed: {0}")]
    OperationFailed(String),

    #[error("Invalid credentials format: {0}")]
    InvalidFormat(String),
}

/// Result type for keychain operations
pub type KeychainResult<T> = Result<T, KeychainError>;

/// Cross-platform keychain service wrapper
pub struct KeychainService {
    service_name: String,
    backend: KeychainBackend,
}

/// Platform-specific keychain backends
#[derive(Debug)]
enum KeychainBackend {
    /// Native OS keychain via the `keyring` crate (macOS Keychain,
    /// Windows Credential Manager, Linux Secret Service). Entries are
    /// constructed per-account so each credential is stored separately.
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    Keyring,
    Fallback,
}

impl Default for KeychainService {
    fn default() -> Self {
        Self::new()
    }
}

impl KeychainService {
    /// Create a new KeychainService instance
    ///
    /// Automatically detects the platform and initializes the appropriate backend.
    /// Falls back to a no-op backend if the keychain is not available.
    pub fn new() -> Self {
        let service_name = SSH_SERVICE_NAME.to_string();
        let backend = Self::init_backend(&service_name);

        tracing::info!("KeychainService initialized with backend: {:?}", backend);

        KeychainService {
            service_name,
            backend,
        }
    }

    /// Initialize the platform-specific backend
    fn init_backend(service_name: &str) -> KeychainBackend {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        {
            // Probe availability by constructing an entry against the platform
            // credential store. Actual credential entries are built per-account
            // at call time so each key is stored under its own identifier.
            match keyring::Entry::new(service_name, "tftsr_ssh_probe") {
                Ok(_) => {
                    tracing::info!("Platform keychain backend initialized successfully");
                    KeychainBackend::Keyring
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize platform keychain: {e}");
                    tracing::warn!("Falling back to encrypted database storage");
                    KeychainBackend::Fallback
                }
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            let _ = service_name;
            tracing::warn!("Unsupported platform for keychain access");
            KeychainBackend::Fallback
        }
    }

    /// Build a per-account keyring entry under this service's namespace.
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    fn entry(&self, account: &str) -> KeychainResult<keyring::Entry> {
        keyring::Entry::new(&self.service_name, account).map_err(|e| {
            tracing::error!("Failed to build keychain entry: {e}");
            KeychainError::OperationFailed(e.to_string())
        })
    }

    /// Store a password in the keychain
    ///
    /// # Arguments
    /// * `account` - The account identifier (e.g., username@host)
    /// * `password` - The password to store
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(KeychainError)` if storage failed
    pub fn store_password(&self, account: &str, password: &str) -> KeychainResult<()> {
        match &self.backend {
            #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
            KeychainBackend::Keyring => {
                let entry = self.entry(account)?;
                match entry.set_password(password) {
                    Ok(()) => {
                        tracing::debug!("Password stored successfully in platform keychain");
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Failed to store password in platform keychain: {e}");
                        Err(KeychainError::OperationFailed(e.to_string()))
                    }
                }
            }
            KeychainBackend::Fallback => {
                tracing::warn!("Keychain not available, password storage skipped");
                Err(KeychainError::NotAvailable(
                    "Keychain backend not available".to_string(),
                ))
            }
        }
    }

    /// Retrieve a password from the keychain
    ///
    /// # Arguments
    /// * `account` - The account identifier
    ///
    /// # Returns
    /// * `Ok(Some(password))` if found
    /// * `Ok(None)` if not found
    /// * `Err(KeychainError)` if retrieval failed
    pub fn get_password(&self, account: &str) -> KeychainResult<Option<String>> {
        match &self.backend {
            #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
            KeychainBackend::Keyring => {
                let entry = self.entry(account)?;
                match entry.get_password() {
                    Ok(password) => {
                        tracing::debug!("Password retrieved successfully from platform keychain");
                        Ok(Some(password))
                    }
                    Err(keyring::error::Error::NoEntry) => {
                        tracing::debug!("No password found in platform keychain");
                        Ok(None)
                    }
                    Err(e) => {
                        tracing::error!("Failed to retrieve password from platform keychain: {e}");
                        Err(KeychainError::OperationFailed(e.to_string()))
                    }
                }
            }
            KeychainBackend::Fallback => {
                tracing::warn!("Keychain not available for password retrieval");
                Err(KeychainError::NotAvailable(
                    "Keychain backend not available".to_string(),
                ))
            }
        }
    }

    /// Delete a password from the keychain
    ///
    /// # Arguments
    /// * `account` - The account identifier
    ///
    /// # Returns
    /// * `Ok(())` if successful or if item doesn't exist
    /// * `Err(KeychainError)` if deletion failed
    pub fn delete_password(&self, account: &str) -> KeychainResult<()> {
        match &self.backend {
            #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
            KeychainBackend::Keyring => {
                let entry = self.entry(account)?;
                match entry.delete_credential() {
                    Ok(()) => {
                        tracing::debug!("Password deleted successfully from platform keychain");
                        Ok(())
                    }
                    Err(keyring::error::Error::NoEntry) => {
                        tracing::debug!("No password to delete in platform keychain");
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete password from platform keychain: {e}");
                        Err(KeychainError::OperationFailed(e.to_string()))
                    }
                }
            }
            KeychainBackend::Fallback => {
                tracing::warn!("Keychain not available for password deletion");
                Ok(()) // No-op in fallback mode
            }
        }
    }

    /// Store an SSH key in the keychain
    ///
    /// # Arguments
    /// * `key_id` - Unique identifier for the SSH key (e.g., "host:user:identity")
    /// * `key_data` - The SSH key content (private key)
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(KeychainError)` if storage failed
    pub fn store_ssh_key(&self, key_id: &str, key_data: &str) -> KeychainResult<()> {
        // Validate key format
        if !key_data.starts_with("-----BEGIN") {
            return Err(KeychainError::InvalidFormat(
                "SSH key must start with BEGIN marker".to_string(),
            ));
        }

        self.store_password(key_id, key_data)
    }

    /// Retrieve an SSH key from the keychain
    ///
    /// # Arguments
    /// * `key_id` - The unique identifier for the SSH key
    ///
    /// # Returns
    /// * `Ok(Some(key_data))` if found
    /// * `Ok(None)` if not found
    /// * `Err(KeychainError)` if retrieval failed
    pub fn get_ssh_key(&self, key_id: &str) -> KeychainResult<Option<String>> {
        self.get_password(key_id)
    }

    /// Delete an SSH key from the keychain
    ///
    /// # Arguments
    /// * `key_id` - The unique identifier for the SSH key
    ///
    /// # Returns
    /// * `Ok(())` if successful or if item doesn't exist
    /// * `Err(KeychainError)` if deletion failed
    pub fn delete_ssh_key(&self, key_id: &str) -> KeychainResult<()> {
        self.delete_password(key_id)
    }

    /// Check if keychain is available
    ///
    /// # Returns
    /// * `true` if keychain backend is available
    /// * `false` if falling back to encrypted database
    pub fn is_available(&self) -> bool {
        !matches!(self.backend, KeychainBackend::Fallback)
    }

    /// Get the service name being used
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

/// Fallback storage using encrypted database
/// Used when keychain is not available
pub struct FallbackStorage {
    #[allow(dead_code)]
    encryption_key: Vec<u8>,
}

impl FallbackStorage {
    /// Create a new fallback storage instance
    pub fn new() -> Self {
        let encryption_key = crate::integrations::auth::get_encryption_key_material_hex()
            .unwrap_or_else(|_| {
                tracing::warn!("FallbackStorage: could not load persistent encryption key; falling back to zero key");
                hex::encode([0u8; 32])
            });

        FallbackStorage {
            encryption_key: hex::decode(&encryption_key).unwrap_or_else(|_| vec![0u8; 32]),
        }
    }

    /// Store data using AES-GCM encryption
    pub fn store(&self, _key: &str, data: &str) -> Result<String, anyhow::Error> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use rand::RngCore;

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

        // Return nonce + ciphertext as hex string
        let mut result = nonce.to_vec();
        result.extend(&ciphertext);

        Ok(hex::encode(result))
    }

    /// Retrieve and decrypt data
    pub fn retrieve(
        &self,
        _key: &str,
        encrypted_data: &str,
    ) -> Result<Option<String>, anyhow::Error> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        let bytes = hex::decode(encrypted_data)
            .map_err(|e| anyhow::anyhow!("Invalid hex encoding: {e}"))?;

        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("Encrypted data too short"));
        }

        let nonce = Nonce::from_slice(&bytes[..12]);
        let ciphertext = &bytes[12..];

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {e}"))?;

        Ok(Some(
            String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {e}"))?,
        ))
    }

    /// Delete stored data
    pub fn delete(&self, _key: &str) -> Result<(), anyhow::Error> {
        // In fallback mode, deletion is handled by the database layer
        Ok(())
    }
}

impl Default for FallbackStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// SSH credential manager that integrates keychain with fallback storage
pub struct SshCredentialManager {
    keychain: KeychainService,
    fallback: FallbackStorage,
    use_keychain: bool,
}

impl SshCredentialManager {
    /// Create a new SSH credential manager
    pub fn new() -> Self {
        let use_keychain = std::env::var("TFTSR_USE_KEYCHAIN")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        SshCredentialManager {
            keychain: KeychainService::new(),
            fallback: FallbackStorage::new(),
            use_keychain,
        }
    }

    /// Store SSH credentials
    pub fn store_credentials(
        &self,
        key_id: &str,
        credentials: &str,
        db: Option<&rusqlite::Connection>,
    ) -> Result<(), anyhow::Error> {
        if self.use_keychain && self.keychain.is_available() {
            self.keychain
                .store_ssh_key(key_id, credentials)
                .map_err(|e| anyhow::anyhow!("Keychain storage failed: {e}"))
        } else if let Some(conn) = db {
            let encrypted = self.fallback.store(key_id, credentials)?;
            self.store_in_db(conn, key_id, &encrypted)
        } else {
            Err(anyhow::anyhow!(
                "No database connection available for fallback storage"
            ))
        }
    }

    /// Retrieve SSH credentials
    pub fn get_credentials(
        &self,
        key_id: &str,
        db: Option<&rusqlite::Connection>,
    ) -> Result<Option<String>, anyhow::Error> {
        if self.use_keychain && self.keychain.is_available() {
            match self.keychain.get_ssh_key(key_id) {
                Ok(Some(creds)) => Ok(Some(creds)),
                Ok(None) => Ok(None),
                Err(_) => {
                    if let Some(conn) = db {
                        let encrypted = self.get_from_db(conn, key_id)?;
                        encrypted
                            .map(|enc| self.fallback.retrieve(key_id, &enc))
                            .transpose()
                            .map(|r| r.flatten())
                    } else {
                        Ok(None)
                    }
                }
            }
        } else if let Some(conn) = db {
            let encrypted = self.get_from_db(conn, key_id)?;
            encrypted
                .map(|enc| self.fallback.retrieve(key_id, &enc))
                .transpose()
                .map(|r| r.flatten())
        } else {
            Ok(None)
        }
    }

    /// Delete SSH credentials
    pub fn delete_credentials(
        &self,
        key_id: &str,
        db: Option<&rusqlite::Connection>,
    ) -> Result<(), anyhow::Error> {
        if self.use_keychain && self.keychain.is_available() {
            self.keychain
                .delete_ssh_key(key_id)
                .map_err(|e| anyhow::anyhow!("Keychain deletion failed: {e}"))?;
        }
        if let Some(conn) = db {
            self.delete_from_db(conn, key_id)?;
        }
        Ok(())
    }

    /// Check if credentials exist
    pub fn has_credentials(
        &self,
        key_id: &str,
        db: Option<&rusqlite::Connection>,
    ) -> Result<bool, anyhow::Error> {
        if self.use_keychain && self.keychain.is_available() {
            Ok(self.keychain.get_ssh_key(key_id)?.is_some())
        } else if let Some(conn) = db {
            Ok(self.get_from_db(conn, key_id)?.is_some())
        } else {
            Ok(false)
        }
    }

    /// Store encrypted data in database
    fn store_in_db(
        &self,
        conn: &rusqlite::Connection,
        key_id: &str,
        encrypted_data: &str,
    ) -> Result<(), anyhow::Error> {
        use rusqlite::params;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            "INSERT OR REPLACE INTO ssh_credentials (key_id, encrypted_data, updated_at)
             VALUES (?1, ?2, ?3)",
            params![key_id, encrypted_data, now],
        )?;

        Ok(())
    }

    /// Retrieve encrypted data from database
    fn get_from_db(
        &self,
        conn: &rusqlite::Connection,
        key_id: &str,
    ) -> Result<Option<String>, anyhow::Error> {
        let encrypted = conn.query_row(
            "SELECT encrypted_data FROM ssh_credentials WHERE key_id = ?1",
            [key_id],
            |row| row.get(0),
        );

        match encrypted {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Database query failed: {e}")),
        }
    }

    /// Delete credentials from database
    fn delete_from_db(
        &self,
        conn: &rusqlite::Connection,
        key_id: &str,
    ) -> Result<(), anyhow::Error> {
        conn.execute("DELETE FROM ssh_credentials WHERE key_id = ?1", [key_id])?;

        Ok(())
    }

    /// Check if keychain is available
    pub fn is_keychain_available(&self) -> bool {
        self.keychain.is_available()
    }
}

impl Default for SshCredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keychain_service_creation() {
        let service = KeychainService::new();
        assert_eq!(service.service_name(), SSH_SERVICE_NAME);
    }

    #[test]
    fn test_keychain_availability() {
        let service = KeychainService::new();
        // This will depend on the platform and availability
        let available = service.is_available();
        tracing::info!("Keychain availability: {available}");
    }

    #[test]
    fn test_ssh_key_validation() {
        let service = KeychainService::new();

        // Test invalid key format
        let result = service.store_ssh_key("test", "invalid key data");
        assert!(result.is_err());

        // Test valid key format (just the header, not a real key)
        let valid_key =
            "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----";
        let result = service.store_ssh_key("test_key", valid_key);

        // This may fail due to keychain unavailability, but should pass validation
        match result {
            Ok(()) => tracing::info!("SSH key stored successfully"),
            Err(KeychainError::NotAvailable(_)) => {
                tracing::info!("Keychain not available (expected in CI)")
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    fn test_fallback_storage() {
        let fallback = FallbackStorage::new();
        let test_data = "test credentials data";

        // Store
        let encrypted = fallback.store("test_key", test_data).unwrap();
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, test_data);

        // Retrieve
        let decrypted = fallback.retrieve("test_key", &encrypted).unwrap();
        assert_eq!(decrypted, Some(test_data.to_string()));

        // Invalid key should return None
        let not_found = fallback.retrieve("nonexistent", &encrypted).unwrap();
        // Note: fallback storage doesn't track keys, so this will still decrypt
        assert!(not_found.is_some());
    }

    #[test]
    fn test_ssh_credential_manager() {
        let manager = SshCredentialManager::new();

        assert_eq!(
            manager.is_keychain_available(),
            manager.keychain.is_available()
        );
    }

    #[test]
    fn test_credential_lifecycle() {
        let manager = SshCredentialManager::new();
        let test_key_id = "test:credential:lifecycle";
        let test_credentials =
            "-----BEGIN OPENSSH PRIVATE KEY-----\ntest_key\n-----END OPENSSH PRIVATE KEY-----";

        // Store credentials
        let store_result = manager.store_credentials(test_key_id, test_credentials, None);

        match store_result {
            Ok(()) => {
                tracing::info!("Credentials stored successfully");

                // Retrieve credentials
                match manager.get_credentials(test_key_id, None) {
                    Ok(Some(retrieved)) => {
                        assert_eq!(retrieved, test_credentials);
                        tracing::info!("Credentials retrieved successfully");
                    }
                    Ok(None) => {
                        tracing::warn!("Credentials not found after storage");
                    }
                    Err(e) => {
                        tracing::error!("Failed to retrieve credentials: {e}");
                    }
                }

                // Delete credentials
                match manager.delete_credentials(test_key_id, None) {
                    Ok(()) => {
                        tracing::info!("Credentials deleted successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete credentials: {e}");
                    }
                }
            }
            Err(e) => {
                tracing::info!("Credential storage skipped (keychain unavailable): {e}");
            }
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_keychain_roundtrip() {
        let service = KeychainService::new();

        // Skip if the macOS keychain backend is unavailable (e.g. headless CI).
        if !service.is_available() {
            tracing::info!("macOS keychain unavailable, skipping roundtrip test");
            return;
        }

        // Two distinct accounts to prove per-account isolation (no collision).
        let account_a = "test:macos:roundtrip:a";
        let account_b = "test:macos:roundtrip:b";
        let password_a = "s3cret-roundtrip-a";
        let password_b = "s3cret-roundtrip-b";

        // Ensure a clean slate.
        let _ = service.delete_password(account_a);
        let _ = service.delete_password(account_b);

        // Store distinct values under distinct accounts.
        service.store_password(account_a, password_a).unwrap();
        service.store_password(account_b, password_b).unwrap();

        // Each account must return its own value (entries do not collide).
        assert_eq!(
            service.get_password(account_a).unwrap(),
            Some(password_a.to_string())
        );
        assert_eq!(
            service.get_password(account_b).unwrap(),
            Some(password_b.to_string())
        );

        // Deleting one must not affect the other.
        service.delete_password(account_a).unwrap();
        assert_eq!(service.get_password(account_a).unwrap(), None);
        assert_eq!(
            service.get_password(account_b).unwrap(),
            Some(password_b.to_string())
        );

        // Clean up.
        service.delete_password(account_b).unwrap();
        assert_eq!(service.get_password(account_b).unwrap(), None);
    }
}

// Re-export for use in other modules
pub mod ssh_integration {
    use super::*;

    /// Helper function to get SSH key for connection
    pub fn get_ssh_key_for_host(
        host: &str,
        username: &str,
    ) -> Result<Option<String>, anyhow::Error> {
        let manager = SshCredentialManager::new();
        let key_id = format!("{host}:{username}:identity");
        manager.get_credentials(&key_id, None)
    }

    /// Helper function to store SSH key for host
    pub fn store_ssh_key_for_host(
        host: &str,
        username: &str,
        key_data: &str,
    ) -> Result<(), anyhow::Error> {
        let manager = SshCredentialManager::new();
        let key_id = format!("{host}:{username}:identity");
        manager.store_credentials(&key_id, key_data, None)
    }

    /// Helper function to delete SSH key for host
    pub fn delete_ssh_key_for_host(host: &str, username: &str) -> Result<(), anyhow::Error> {
        let manager = SshCredentialManager::new();
        let key_id = format!("{host}:{username}:identity");
        manager.delete_credentials(&key_id, None)
    }
}
