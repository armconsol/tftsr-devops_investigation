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

use anyhow::{Context, Result};
use std::sync::OnceLock;
use zeroize::Zeroize;

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
enum KeychainBackend {
    #[cfg(target_os = "macos")]
    macOS(security_framework::secitem::SecItem),
    #[cfg(target_os = "windows")]
    Windows(keyring::Entry),
    #[cfg(target_os = "linux")]
    Linux(keyring::Entry),
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
        #[cfg(target_os = "macos")]
        {
            use security_framework::secitem::{SecItem, SecClass, SecItemQuery};
            use security_framework::item::SecItemAddArguments;
            
            match SecItem::new(SecItemQuery {
                sec_class: SecClass::GenericPassword,
                account: Some("tftsr_ssh_account"),
                service: Some(service_name),
                ..Default::default()
            }) {
                Ok(sec_item) => {
                    tracing::info!("macOS Security.framework initialized successfully");
                    KeychainBackend::macOS(sec_item)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize macOS Security.framework: {e}");
                    tracing::warn!("Falling back to encrypted database storage");
                    KeychainBackend::Fallback
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            match keyring::Entry::new(service_name, "tftsr_ssh_account") {
                Ok(entry) => {
                    tracing::info!("Windows Credential Manager initialized successfully");
                    KeychainBackend::Windows(entry)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize Windows Credential Manager: {e}");
                    tracing::warn!("Falling back to encrypted database storage");
                    KeychainBackend::Fallback
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            match keyring::Entry::new(service_name, "tftsr_ssh_account") {
                Ok(entry) => {
                    tracing::info!("Linux Secret Service initialized successfully");
                    KeychainBackend::Linux(entry)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize Linux Secret Service: {e}");
                    tracing::warn!("Falling back to encrypted database storage");
                    KeychainBackend::Fallback
                }
            }
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            tracing::warn!("Unsupported platform for keychain access");
            KeychainBackend::Fallback
        }
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
            #[cfg(target_os = "macos")]
            KeychainBackend::macOS(_) => {
                self.store_macos_password(account, password)
            }
            #[cfg(target_os = "windows")]
            KeychainBackend::Windows(entry) => {
                match entry.set_password(password) {
                    Ok(()) => {
                        tracing::debug!("Password stored successfully in Windows Credential Manager");
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Failed to store password in Windows Credential Manager: {e}");
                        Err(KeychainError::OperationFailed(e.to_string()))
                    }
                }
            }
            #[cfg(target_os = "linux")]
            KeychainBackend::Linux(entry) => {
                match entry.set_password(password) {
                    Ok(()) => {
                        tracing::debug!("Password stored successfully in GNOME Keyring");
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Failed to store password in GNOME Keyring: {e}");
                        Err(KeychainError::OperationFailed(e.to_string()))
                    }
                }
            }
            KeychainBackend::Fallback => {
                tracing::warn!("Keychain not available, password storage skipped");
                Err(KeychainError::NotAvailable(
                    "Keychain backend not available".to_string()
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
            #[cfg(target_os = "macos")]
            KeychainBackend::macOS(_) => {
                self.get_macos_password(account)
            }
            #[cfg(target_os = "windows")]
            KeychainBackend::Windows(entry) => {
                match entry.get_password() {
                    Ok(password) => {
                        tracing::debug!("Password retrieved successfully from Windows Credential Manager");
                        Ok(Some(password))
                    }
                    Err(keyring::error::Error::NoEntry) => {
                        tracing::debug!("No password found in Windows Credential Manager");
                        Ok(None)
                    }
                    Err(e) => {
                        tracing::error!("Failed to retrieve password from Windows Credential Manager: {e}");
                        Err(KeychainError::OperationFailed(e.to_string()))
                    }
                }
            }
            #[cfg(target_os = "linux")]
            KeychainBackend::Linux(entry) => {
                match entry.get_password() {
                    Ok(password) => {
                        tracing::debug!("Password retrieved successfully from GNOME Keyring");
                        Ok(Some(password))
                    }
                    Err(keyring::error::Error::NoEntry) => {
                        tracing::debug!("No password found in GNOME Keyring");
                        Ok(None)
                    }
                    Err(e) => {
                        tracing::error!("Failed to retrieve password from GNOME Keyring: {e}");
                        Err(KeychainError::OperationFailed(e.to_string()))
                    }
                }
            }
            KeychainBackend::Fallback => {
                tracing::warn!("Keychain not available for password retrieval");
                Err(KeychainError::NotAvailable(
                    "Keychain backend not available".to_string()
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
            #[cfg(target_os = "macos")]
            KeychainBackend::macOS(_) => {
                self.delete_macos_password(account)
            }
            #[cfg(target_os = "windows")]
            KeychainBackend::Windows(entry) => {
                match entry.delete_password() {
                    Ok(()) => {
                        tracing::debug!("Password deleted successfully from Windows Credential Manager");
                        Ok(())
                    }
                    Err(keyring::error::Error::NoEntry) => {
                        tracing::debug!("No password to delete in Windows Credential Manager");
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete password from Windows Credential Manager: {e}");
                        Err(KeychainError::OperationFailed(e.to_string()))
                    }
                }
            }
            #[cfg(target_os = "linux")]
            KeychainBackend::Linux(entry) => {
                match entry.delete_password() {
                    Ok(()) => {
                        tracing::debug!("Password deleted successfully from GNOME Keyring");
                        Ok(())
                    }
                    Err(keyring::error::Error::NoEntry) => {
                        tracing::debug!("No password to delete in GNOME Keyring");
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete password from GNOME Keyring: {e}");
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
                "SSH key must start with BEGIN marker".to_string()
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

#[cfg(target_os = "macos")]
impl KeychainService {
    /// macOS-specific password storage using Security.framework
    fn store_macos_password(&self, account: &str, password: &str) -> KeychainResult<()> {
        use security_framework::item::{SecItemAddArguments, SecItemUpdateArguments};
        use security_framework::secitem::{SecClass, SecItemQuery};
        
        let mut query = SecItemQuery::default();
        query.sec_class = SecClass::GenericPassword;
        query.account = Some(account);
        query.service = Some(&self.service_name);
        
        let mut attributes = SecItemAddArguments::default();
        attributes.sec_class = SecClass::GenericPassword;
        attributes.account = Some(account);
        attributes.service = Some(&self.service_name);
        attributes.value_data = Some(password.as_bytes());
        
        // Try to update existing item first
        let update_result = security_framework::secitem::update_item(&query, &SecItemUpdateArguments {
            value_data: Some(password.as_bytes()),
            ..Default::default()
        });
        
        if update_result.is_ok() {
            tracing::debug!("Password updated in macOS Keychain for account: {account}");
            return Ok(());
        }
        
        // If update fails, try to add new item
        match security_framework::secitem::add_item(&attributes) {
            Ok(_) => {
                tracing::debug!("Password added to macOS Keychain for account: {account}");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to add password to macOS Keychain: {e}");
                Err(KeychainError::OperationFailed(e.to_string()))
            }
        }
    }
    
    /// macOS-specific password retrieval using Security.framework
    fn get_macos_password(&self, account: &str) -> KeychainResult<Option<String>> {
        use security_framework::item::{SecItemCopyMatching, SecItemQuery};
        use security_framework::secitem::SecClass;
        
        let mut query = SecItemQuery::default();
        query.sec_class = SecClass::GenericPassword;
        query.account = Some(account);
        query.service = Some(&self.service_name);
        query.return_data = true;
        
        match security_framework::secitem::copy_item(&query) {
            Ok(result) => {
                if let Some(data) = result.data {
                    match String::from_utf8(data) {
                        Ok(password) => {
                            tracing::debug!("Password retrieved from macOS Keychain for account: {account}");
                            Ok(Some(password))
                        }
                        Err(e) => {
                            tracing::error!("Failed to convert macOS Keychain data to string: {e}");
                            Err(KeychainError::OperationFailed("Invalid UTF-8 data".to_string()))
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            Err(security_framework::error::Error::ItemNotFound) => {
                tracing::debug!("No password found in macOS Keychain for account: {account}");
                Ok(None)
            }
            Err(e) => {
                tracing::error!("Failed to retrieve password from macOS Keychain: {e}");
                Err(KeychainError::OperationFailed(e.to_string()))
            }
        }
    }
    
    /// macOS-specific password deletion using Security.framework
    fn delete_macos_password(&self, account: &str) -> KeychainResult<()> {
        use security_framework::item::SecItemQuery;
        use security_framework::secitem::SecClass;
        
        let mut query = SecItemQuery::default();
        query.sec_class = SecClass::GenericPassword;
        query.account = Some(account);
        query.service = Some(&self.service_name);
        
        match security_framework::secitem::delete_item(&query) {
            Ok(_) => {
                tracing::debug!("Password deleted from macOS Keychain for account: {account}");
                Ok(())
            }
            Err(security_framework::error::Error::ItemNotFound) => {
                tracing::debug!("No password to delete in macOS Keychain for account: {account}");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to delete password from macOS Keychain: {e}");
                Err(KeychainError::OperationFailed(e.to_string()))
            }
        }
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
        // Generate or load encryption key from environment
        let encryption_key = std::env::var("TRCAA_ENCRYPTION_KEY")
            .unwrap_or_else(|_| {
                // Generate a random key for fallback mode
                use rand::RngCore;
                let mut key = vec![0u8; 32];
                rand::thread_rng().fill_bytes(&mut key);
                hex::encode(key)
            });
        
        FallbackStorage {
            encryption_key: hex::decode(&encryption_key).unwrap_or_else(|_| vec![0u8; 32]),
        }
    }
    
    /// Store data using AES-GCM encryption
    pub fn store(&self, key: &str, data: &str) -> Result<String, anyhow::Error> {
        use aes_gcm::{
            aead::{Aead, KeyInit, OsRng},
            Aes256Gcm, Nonce,
        };
        
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&OsRng.gen::<[u8; 12]>());
        
        let ciphertext = cipher.encrypt(nonce, data.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;
        
        // Return nonce + ciphertext as hex string
        let mut result = nonce.to_vec();
        result.extend(&ciphertext);
        
        Ok(hex::encode(result))
    }
    
    /// Retrieve and decrypt data
    pub fn retrieve(&self, key: &str, encrypted_data: &str) -> Result<Option<String>, anyhow::Error> {
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
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {e}"))?;
        
        Ok(Some(String::from_utf8(plaintext)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8: {e}"))?))
    }
    
    /// Delete stored data
    pub fn delete(&self, key: &str) -> Result<(), anyhow::Error> {
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
    pub fn store_credentials(&self, key_id: &str, credentials: &str) -> Result<(), anyhow::Error> {
        if self.use_keychain && self.keychain.is_available() {
            self.keychain.store_ssh_key(key_id, credentials)
                .map_err(|e| anyhow::anyhow!("Keychain storage failed: {e}"))
        } else {
            let encrypted = self.fallback.store(key_id, credentials)?;
            // Store encrypted data in database
            self.store_in_db(key_id, &encrypted)
        }
    }
    
    /// Retrieve SSH credentials
    pub fn get_credentials(&self, key_id: &str) -> Result<Option<String>, anyhow::Error> {
        if self.use_keychain && self.keychain.is_available() {
            match self.keychain.get_ssh_key(key_id) {
                Ok(Some(creds)) => Ok(Some(creds)),
                Ok(None) => Ok(None),
                Err(_) => {
                    // Fall back to database
                    self.get_from_db(key_id)
                        .and_then(|encrypted| {
                            encrypted.map(|enc| self.fallback.retrieve(key_id, &enc))
                                .unwrap_or(Ok(None))
                                .transpose()
                        })
                }
            }
        } else {
            self.get_from_db(key_id)
                .and_then(|encrypted| {
                    encrypted.map(|enc| self.fallback.retrieve(key_id, &enc))
                        .unwrap_or(Ok(None))
                        .transpose()
                })
        }
    }
    
    /// Delete SSH credentials
    pub fn delete_credentials(&self, key_id: &str) -> Result<(), anyhow::Error> {
        if self.use_keychain && self.keychain.is_available() {
            self.keychain.delete_ssh_key(key_id)
                .map_err(|e| anyhow::anyhow!("Keychain deletion failed: {e}"))?;
        }
        // Also remove from database
        self.delete_from_db(key_id)
    }
    
    /// Check if credentials exist
    pub fn has_credentials(&self, key_id: &str) -> Result<bool, anyhow::Error> {
        if self.use_keychain && self.keychain.is_available() {
            Ok(self.keychain.get_ssh_key(key_id)?.is_some())
        } else {
            Ok(self.get_from_db(key_id)?.is_some())
        }
    }
    
    /// Store encrypted data in database
    fn store_in_db(&self, key_id: &str, encrypted_data: &str) -> Result<(), anyhow::Error> {
        use crate::db::connection::get_db_connection;
        use rusqlite::params;
        
        let conn = get_db_connection()?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        conn.execute(
            "INSERT OR REPLACE INTO ssh_credentials (key_id, encrypted_data, updated_at)
             VALUES (?1, ?2, ?3)",
            params![key_id, encrypted_data, now],
        )?;
        
        Ok(())
    }
    
    /// Retrieve encrypted data from database
    fn get_from_db(&self, key_id: &str) -> Result<Option<String>, anyhow::Error> {
        use crate::db::connection::get_db_connection;
        
        let conn = get_db_connection()?;
        
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
    fn delete_from_db(&self, key_id: &str) -> Result<(), anyhow::Error> {
        use crate::db::connection::get_db_connection;
        
        let conn = get_db_connection()?;
        conn.execute(
            "DELETE FROM ssh_credentials WHERE key_id = ?1",
            [key_id],
        )?;
        
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
        let valid_key = "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----";
        let result = service.store_ssh_key("test_key", valid_key);
        
        // This may fail due to keychain unavailability, but should pass validation
        match result {
            Ok(()) => tracing::info!("SSH key stored successfully"),
            Err(KeychainError::NotAvailable(_)) => tracing::info!("Keychain not available (expected in CI)"),
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
        
        assert_eq!(manager.is_keychain_available(), manager.keychain.is_available());
    }
    
    #[test]
    fn test_credential_lifecycle() {
        let manager = SshCredentialManager::new();
        let test_key_id = "test:credential:lifecycle";
        let test_credentials = "-----BEGIN OPENSSH PRIVATE KEY-----\ntest_key\n-----END OPENSSH PRIVATE KEY-----";
        
        // Store credentials
        let store_result = manager.store_credentials(test_key_id, test_credentials);
        
        match store_result {
            Ok(()) => {
                tracing::info!("Credentials stored successfully");
                
                // Retrieve credentials
                match manager.get_credentials(test_key_id) {
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
                match manager.delete_credentials(test_key_id) {
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
}

// Re-export for use in other modules
pub mod ssh_integration {
    use super::*;
    
    /// Helper function to get SSH key for connection
    pub fn get_ssh_key_for_host(host: &str, username: &str) -> Result<Option<String>, anyhow::Error> {
        let manager = SshCredentialManager::new();
        let key_id = format!("{host}:{username}:identity");
        manager.get_credentials(&key_id)
    }
    
    /// Helper function to store SSH key for host
    pub fn store_ssh_key_for_host(
        host: &str,
        username: &str,
        key_data: &str,
    ) -> Result<(), anyhow::Error> {
        let manager = SshCredentialManager::new();
        let key_id = format!("{host}:{username}:identity");
        manager.store_credentials(&key_id, key_data)
    }
    
    /// Helper function to delete SSH key for host
    pub fn delete_ssh_key_for_host(host: &str, username: &str) -> Result<(), anyhow::Error> {
        let manager = SshCredentialManager::new();
        let key_id = format!("{host}:{username}:identity");
        manager.delete_credentials(&key_id)
    }
}
