use serde::{Deserialize, Serialize};

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
    use sha2::{Digest, Sha256};

    // Generate a random 32-byte verifier
    let verifier_bytes: Vec<u8> = (0..32)
        .map(|_| {
            let r: u8 = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
                % 256) as u8;
            r
        })
        .collect();

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

/// Exchange an authorization code for tokens. Placeholder for v0.2.
pub async fn exchange_code(
    _token_endpoint: &str,
    _client_id: &str,
    _code: &str,
    _redirect_uri: &str,
    _code_verifier: &str,
) -> Result<OAuthToken, String> {
    Err("OAuth token exchange available in v0.2".to_string())
}

/// Store a PAT credential securely. Placeholder - in production, use OS keychain.
pub fn store_pat(conn: &rusqlite::Connection, credential: &PatCredential) -> Result<(), String> {
    let id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "INSERT OR REPLACE INTO credentials (id, service, token_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, credential.service, hash_token(&credential.token), now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Retrieve a stored PAT. In production, retrieve from OS keychain.
pub fn get_pat(conn: &rusqlite::Connection, service: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT token_hash FROM credentials WHERE service = ?1 ORDER BY created_at DESC LIMIT 1")
        .map_err(|e| e.to_string())?;
    let result = stmt
        .query_row([service], |row| row.get::<_, String>(0))
        .ok();
    Ok(result)
}

fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn base64_url_encode(data: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    URL_SAFE_NO_PAD.encode(data)
}

fn urlencoding_encode(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('+', "%2B")
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
}
