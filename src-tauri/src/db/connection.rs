use rusqlite::Connection;
use std::path::Path;

fn get_db_key() -> anyhow::Result<String> {
    if let Ok(key) = std::env::var("TFTSR_DB_KEY") {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }

    if cfg!(debug_assertions) {
        return Ok("dev-key-change-in-prod".to_string());
    }

    Err(anyhow::anyhow!(
        "TFTSR_DB_KEY must be set in release builds"
    ))
}

pub fn open_encrypted_db(path: &Path, key: &str) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    // ALL cipher settings MUST be set before the first database access.
    // cipher_page_size in particular must precede any read/write so it takes
    // effect for both creation (new DB) and reopening (existing DB).
    // 16384 matches 16KB kernel page size (Asahi Linux / Apple Silicon aarch64).
    conn.execute_batch(&format!(
        "PRAGMA key = '{}';\
         PRAGMA cipher_page_size = 16384;\
         PRAGMA kdf_iter = 256000;\
         PRAGMA cipher_hmac_algorithm = HMAC_SHA512;\
         PRAGMA cipher_kdf_algorithm = PBKDF2_HMAC_SHA512;",
        key.replace('\'', "''")
    ))?;
    // Verify the key and settings work
    conn.execute_batch("SELECT count(*) FROM sqlite_master;")?;
    Ok(conn)
}

pub fn open_dev_db(path: &Path) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    Ok(conn)
}

pub fn init_db(data_dir: &Path) -> anyhow::Result<Connection> {
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("tftsr.db");

    // In dev/test mode use unencrypted DB; in production use encryption
    let key = get_db_key()?;

    let conn = if cfg!(debug_assertions) {
        open_dev_db(&db_path)?
    } else {
        open_encrypted_db(&db_path, &key)?
    };

    crate::db::migrations::run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_key_uses_env_var_when_present() {
        std::env::set_var("TFTSR_DB_KEY", "test-db-key");
        let key = get_db_key().unwrap();
        assert_eq!(key, "test-db-key");
        std::env::remove_var("TFTSR_DB_KEY");
    }

    #[test]
    fn test_get_db_key_debug_fallback_for_empty_env() {
        std::env::set_var("TFTSR_DB_KEY", "   ");
        let key = get_db_key().unwrap();
        assert_eq!(key, "dev-key-change-in-prod");
        std::env::remove_var("TFTSR_DB_KEY");
    }
}
