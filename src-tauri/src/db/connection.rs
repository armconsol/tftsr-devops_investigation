use rusqlite::Connection;
use std::path::Path;

fn generate_key() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[cfg(unix)]
fn write_key_file(path: &Path, key: &str) -> anyhow::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(key.as_bytes())?;
    Ok(())
}

#[cfg(not(unix))]
fn write_key_file(path: &Path, key: &str) -> anyhow::Result<()> {
    std::fs::write(path, key)?;
    Ok(())
}

fn get_db_key(data_dir: &Path) -> anyhow::Result<String> {
    if let Ok(key) = std::env::var("TFTSR_DB_KEY") {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }

    if cfg!(debug_assertions) {
        return Ok("dev-key-change-in-prod".to_string());
    }

    // Release: load or auto-generate a per-installation key stored in the
    // app data directory. This lets the app work out of the box without
    // requiring users to set an environment variable.
    let key_path = data_dir.join(".dbkey");
    if key_path.exists() {
        let key = std::fs::read_to_string(&key_path)?;
        let key = key.trim().to_string();
        if !key.is_empty() {
            return Ok(key);
        }
    }

    let key = generate_key();
    std::fs::create_dir_all(data_dir)?;
    write_key_file(&key_path, &key)?;
    Ok(key)
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

    let key = get_db_key(data_dir)?;

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

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tftsr-test-{}", name));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_get_db_key_uses_env_var_when_present() {
        let dir = temp_dir("env-var");
        std::env::set_var("TFTSR_DB_KEY", "test-db-key");
        let key = get_db_key(&dir).unwrap();
        assert_eq!(key, "test-db-key");
        std::env::remove_var("TFTSR_DB_KEY");
    }

    #[test]
    fn test_get_db_key_debug_fallback_for_empty_env() {
        let dir = temp_dir("empty-env");
        std::env::set_var("TFTSR_DB_KEY", "   ");
        let key = get_db_key(&dir).unwrap();
        assert_eq!(key, "dev-key-change-in-prod");
        std::env::remove_var("TFTSR_DB_KEY");
    }
}
