use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkceChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatCredential {
    pub service: String,
    pub token: String,
}

/// Generate a PKCE code verifier and challenge for OAuth flows.
pub fn generate_pkce() -> PkceChallenge {
    use rand::{thread_rng, RngCore};

    // Generate a random 32-byte verifier
    let mut verifier_bytes = [0u8; 32];
    thread_rng().fill_bytes(&mut verifier_bytes);

    let code_verifier = base64_url_encode(&verifier_bytes);
    let challenge_hash = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = base64_url_encode(&challenge_hash);

    PkceChallenge {
        code_verifier,
        code_challenge,
    }
}

/// Build an OAuth2 authorization URL with PKCE parameters.
pub fn build_auth_url(
    auth_endpoint: &str,
    client_id: &str,
    redirect_uri: &str,
    scope: &str,
    pkce: &PkceChallenge,
) -> String {
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256",
        auth_endpoint,
        urlencoding_encode(client_id),
        urlencoding_encode(redirect_uri),
        urlencoding_encode(scope),
        &pkce.code_challenge,
    )
}

/// Exchange an authorization code for tokens using PKCE.
pub async fn exchange_code(
    token_endpoint: &str,
    client_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<OAuthToken, String> {
    let client = reqwest::Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("code_verifier", code_verifier),
    ];

    let resp = client
        .post(token_endpoint)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send token exchange request: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Token exchange failed with status {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))?;

    let access_token = body["access_token"]
        .as_str()
        .ok_or_else(|| "No access_token in response".to_string())?
        .to_string();

    let refresh_token = body["refresh_token"].as_str().map(|s| s.to_string());

    let expires_in = body["expires_in"].as_i64().unwrap_or(3600);
    let expires_at = chrono::Utc::now().timestamp() + expires_in;

    let token_type = body["token_type"].as_str().unwrap_or("Bearer").to_string();

    Ok(OAuthToken {
        access_token,
        refresh_token,
        expires_at,
        token_type,
    })
}

/// Store a PAT credential securely with AES-256-GCM encryption.
pub fn store_pat(conn: &rusqlite::Connection, credential: &PatCredential) -> Result<(), String> {
    let id = uuid::Uuid::now_v7().to_string();
    let token_hash = hash_token(&credential.token);
    let encrypted_token = encrypt_token(&credential.token)?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT OR REPLACE INTO credentials (id, service, token_hash, encrypted_token, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, credential.service, token_hash, encrypted_token, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Retrieve and decrypt a stored PAT.
pub fn get_pat(conn: &rusqlite::Connection, service: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT encrypted_token FROM credentials WHERE service = ?1 ORDER BY created_at DESC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let encrypted = stmt
        .query_row([service], |row| row.get::<_, String>(0))
        .optional()
        .map_err(|e| e.to_string())?;

    match encrypted {
        Some(enc) => {
            let decrypted = decrypt_token(&enc)?;
            Ok(Some(decrypted))
        }
        None => Ok(None),
    }
}

fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn base64_url_encode(data: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    URL_SAFE_NO_PAD.encode(data)
}

fn urlencoding_encode(s: &str) -> String {
    urlencoding::encode(s).into_owned()
}

fn get_encryption_key_material() -> Result<String, String> {
    if let Ok(key) = std::env::var("TFTSR_ENCRYPTION_KEY") {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }

    if cfg!(debug_assertions) {
        return Ok("dev-key-change-me-in-production-32b".to_string());
    }

    // Release: load or auto-generate a per-installation encryption key
    // stored in the app data directory, similar to the database key.
    if let Some(app_data_dir) = crate::state::get_app_data_dir() {
        let key_path = app_data_dir.join(".enckey");

        // Try to load existing key
        if key_path.exists() {
            if let Ok(key) = std::fs::read_to_string(&key_path) {
                let key = key.trim().to_string();
                if !key.is_empty() {
                    return Ok(key);
                }
            }
        }

        // Generate and store new key
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let key = hex::encode(bytes);

        // Ensure directory exists
        if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
            tracing::warn!("Failed to create app data directory: {e}");
            return Err(format!("Failed to create app data directory: {e}"));
        }

        // Write key with restricted permissions
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&key_path)
                .map_err(|e| format!("Failed to write encryption key: {e}"))?;
            f.write_all(key.as_bytes())
                .map_err(|e| format!("Failed to write encryption key: {e}"))?;
        }

        #[cfg(not(unix))]
        {
            std::fs::write(&key_path, &key)
                .map_err(|e| format!("Failed to write encryption key: {e}"))?;
        }

        tracing::info!("Generated new encryption key at {:?}", key_path);
        return Ok(key);
    }

    Err("Failed to determine app data directory for encryption key storage".to_string())
}

fn derive_aes_key() -> Result<[u8; 32], String> {
    let key_material = get_encryption_key_material()?;
    let digest = Sha256::digest(key_material.as_bytes());
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&digest);
    Ok(key_bytes)
}

/// Encrypt a token using AES-256-GCM.
/// Key is derived from TFTSR_ENCRYPTION_KEY env var or a default dev key.
/// Returns base64-encoded ciphertext with nonce prepended.
pub fn encrypt_token(token: &str) -> Result<String, String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use rand::{thread_rng, RngCore};

    let key_bytes = derive_aes_key()?;

    let cipher = Aes256Gcm::new(&key_bytes.into());

    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, token.as_bytes())
        .map_err(|e| format!("Encryption failed: {e}"))?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    // Base64 encode
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    Ok(STANDARD.encode(&result))
}

/// Decrypt a token that was encrypted with encrypt_token().
pub fn decrypt_token(encrypted: &str) -> Result<String, String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };

    // Base64 decode
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    let data = STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Base64 decode failed: {e}"))?;

    if data.len() < 12 {
        return Err("Invalid encrypted data: too short".to_string());
    }

    // Extract nonce (first 12 bytes) and ciphertext (rest)
    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];

    let key_bytes = derive_aes_key()?;

    let cipher = Aes256Gcm::new(&key_bytes.into());

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {e}"))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pkce_produces_valid_challenge() {
        let pkce = generate_pkce();
        assert!(!pkce.code_verifier.is_empty());
        assert!(!pkce.code_challenge.is_empty());
        // Verifier and challenge should be different
        assert_ne!(pkce.code_verifier, pkce.code_challenge);
    }

    #[test]
    fn test_build_auth_url_contains_required_params() {
        let pkce = PkceChallenge {
            code_verifier: "test_verifier".to_string(),
            code_challenge: "test_challenge".to_string(),
        };
        let url = build_auth_url(
            "https://auth.example.com/authorize",
            "my-client",
            "http://localhost:8080/callback",
            "read write",
            &pkce,
        );
        assert!(url.starts_with("https://auth.example.com/authorize?"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=my-client"));
        assert!(url.contains("code_challenge=test_challenge"));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn test_build_auth_url_encodes_special_chars() {
        let pkce = PkceChallenge {
            code_verifier: "v".to_string(),
            code_challenge: "c".to_string(),
        };
        let url = build_auth_url(
            "https://auth.example.com",
            "client id",
            "http://localhost",
            "read+write",
            &pkce,
        );
        assert!(url.contains("client%20id"));
        assert!(url.contains("read%2Bwrite"));
    }

    #[test]
    fn test_hash_token_deterministic() {
        let h1 = hash_token("my-secret-token");
        let h2 = hash_token("my-secret-token");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_token_different_for_different_inputs() {
        let h1 = hash_token("token-a");
        let h2 = hash_token("token-b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_token_is_hex_string() {
        let h = hash_token("test");
        assert!(h.len() == 64); // SHA-256 = 32 bytes = 64 hex chars
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_urlencoding_encode() {
        assert_eq!(urlencoding_encode("hello world"), "hello%20world");
        assert_eq!(urlencoding_encode("a&b=c+d"), "a%26b%3Dc%2Bd");
    }

    #[test]
    fn test_base64_url_encode_no_padding() {
        let encoded = base64_url_encode(b"test data");
        assert!(!encoded.contains('='));
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
    }

    #[tokio::test]
    async fn test_exchange_code_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/oauth/token")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "access_token": "test_access_token_123",
                    "refresh_token": "test_refresh_token_456",
                    "expires_in": 3600,
                    "token_type": "Bearer"
                }"#,
            )
            .create_async()
            .await;

        let token_endpoint = format!("{server_url}/oauth/token", server_url = server.url());
        let result = exchange_code(
            &token_endpoint,
            "test-client-id",
            "auth_code_xyz",
            "http://localhost:8765/callback",
            "code_verifier_abc",
        )
        .await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let token = result.unwrap();
        assert_eq!(token.access_token, "test_access_token_123");
        assert_eq!(
            token.refresh_token,
            Some("test_refresh_token_456".to_string())
        );
        assert_eq!(token.token_type, "Bearer");
    }

    #[tokio::test]
    async fn test_exchange_code_missing_access_token() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_body(r#"{"expires_in": 3600}"#)
            .create_async()
            .await;

        let token_endpoint = format!("{server_url}/oauth/token", server_url = server.url());
        let result = exchange_code(
            &token_endpoint,
            "test-client-id",
            "code",
            "http://localhost:8765/callback",
            "verifier",
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("access_token"));
    }

    #[tokio::test]
    async fn test_exchange_code_http_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/oauth/token")
            .with_status(401)
            .with_body(r#"{"error": "invalid_grant"}"#)
            .create_async()
            .await;

        let token_endpoint = format!("{server_url}/oauth/token", server_url = server.url());
        let result = exchange_code(
            &token_endpoint,
            "test-client-id",
            "invalid_code",
            "http://localhost:8765/callback",
            "verifier",
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("401") || err.contains("Unauthorized") || err.contains("failed"));
    }

    #[tokio::test]
    async fn test_exchange_code_network_error() {
        // Use an unreachable endpoint
        let result = exchange_code(
            "http://localhost:9999/token",
            "client",
            "code",
            "http://localhost/callback",
            "verifier",
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "my-secret-token-12345";
        let encrypted = encrypt_token(original).unwrap();
        let decrypted = decrypt_token(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_encrypt_produces_different_output_each_time() {
        // Ensure env var is not set from other tests
        std::env::remove_var("TFTSR_ENCRYPTION_KEY");

        let token = "same-token";
        let enc1 = encrypt_token(token).unwrap();
        let enc2 = encrypt_token(token).unwrap();
        // Different nonces mean different ciphertext
        assert_ne!(enc1, enc2);
        // But both decrypt to the same value
        assert_eq!(decrypt_token(&enc1).unwrap(), token);
        assert_eq!(decrypt_token(&enc2).unwrap(), token);
    }

    #[test]
    fn test_decrypt_invalid_data_fails() {
        let result = decrypt_token("invalid-base64-!!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_too_short_fails() {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        let short_data = STANDARD.encode(b"short");
        let result = decrypt_token(&short_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too short"));
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        // Encrypt with one key
        std::env::set_var(
            "TFTSR_ENCRYPTION_KEY",
            "key-1-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        );
        let encrypted = encrypt_token("secret").unwrap();

        // Try to decrypt with different key
        std::env::set_var(
            "TFTSR_ENCRYPTION_KEY",
            "key-2-yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy",
        );
        let result = decrypt_token(&encrypted);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Decryption failed"));

        // Reset env var
        std::env::remove_var("TFTSR_ENCRYPTION_KEY");
    }

    #[test]
    fn test_store_and_retrieve_pat() {
        // Set up test DB
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();

        // Store credential
        let credential = PatCredential {
            service: "confluence".to_string(),
            token: "my-secret-pat-token-12345".to_string(),
        };
        store_pat(&conn, &credential).unwrap();

        // Retrieve and verify
        let retrieved = get_pat(&conn, "confluence").unwrap();
        assert_eq!(retrieved, Some("my-secret-pat-token-12345".to_string()));
    }

    #[test]
    fn test_get_pat_nonexistent_service() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();

        let result = get_pat(&conn, "nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_store_pat_replaces_existing() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();

        // Store first token
        let cred1 = PatCredential {
            service: "servicenow".to_string(),
            token: "token-v1".to_string(),
        };
        store_pat(&conn, &cred1).unwrap();

        // Store second token for same service
        let cred2 = PatCredential {
            service: "servicenow".to_string(),
            token: "token-v2".to_string(),
        };
        store_pat(&conn, &cred2).unwrap();

        // Should retrieve the most recent token
        let retrieved = get_pat(&conn, "servicenow").unwrap();
        assert_eq!(retrieved, Some("token-v2".to_string()));
    }

    #[test]
    fn test_generate_pkce_is_not_deterministic() {
        let a = generate_pkce();
        let b = generate_pkce();
        assert_ne!(a.code_verifier, b.code_verifier);
    }

    #[test]
    fn test_derive_aes_key_is_stable_for_same_input() {
        std::env::set_var("TFTSR_ENCRYPTION_KEY", "stable-test-key");
        let k1 = derive_aes_key().unwrap();
        let k2 = derive_aes_key().unwrap();
        assert_eq!(k1, k2);
        std::env::remove_var("TFTSR_ENCRYPTION_KEY");
    }
}
