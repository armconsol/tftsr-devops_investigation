use rusqlite::Connection;
use std::path::Path;

fn generate_key() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
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
    // Support both TRCAA_DB_KEY (new) and TFTSR_DB_KEY (legacy) for backwards compatibility
    if let Ok(key) = std::env::var("TRCAA_DB_KEY") {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }
    if let Ok(key) = std::env::var("TFTSR_DB_KEY") {
        if !key.trim().is_empty() {
            tracing::warn!("TFTSR_DB_KEY is deprecated, use TRCAA_DB_KEY instead");
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

/// Migrates a plain SQLite database to an encrypted SQLCipher database.
/// Creates a backup of the original file before migration.
fn migrate_plain_to_encrypted(db_path: &Path, key: &str) -> anyhow::Result<Connection> {
    tracing::warn!("Detected plain SQLite database in release build - migrating to encrypted");

    // Create backup of plain database
    let backup_path = db_path.with_extension("db.plain-backup");
    std::fs::copy(db_path, &backup_path)?;
    tracing::info!("Backed up plain database to {:?}", backup_path);

    // Open the plain database
    let plain_conn = Connection::open(db_path)?;

    // Create temporary encrypted database path
    let temp_encrypted = db_path.with_extension("db.encrypted-temp");

    // Attach and migrate to encrypted database using SQLCipher export
    plain_conn.execute_batch(&format!(
        "ATTACH DATABASE '{}' AS encrypted KEY '{}';\
         PRAGMA encrypted.cipher_page_size = 16384;\
         PRAGMA encrypted.kdf_iter = 256000;\
         PRAGMA encrypted.cipher_hmac_algorithm = HMAC_SHA512;\
         PRAGMA encrypted.cipher_kdf_algorithm = PBKDF2_HMAC_SHA512;",
        temp_encrypted.display(),
        key.replace('\'', "''")
    ))?;

    // Export all data to encrypted database
    plain_conn.execute_batch("SELECT sqlcipher_export('encrypted');")?;
    plain_conn.execute_batch("DETACH DATABASE encrypted;")?;
    drop(plain_conn);

    // Replace original with encrypted version
    std::fs::rename(&temp_encrypted, db_path)?;
    tracing::info!("Successfully migrated database to encrypted format");

    // Open and return the encrypted database
    open_encrypted_db(db_path, key)
}

/// Checks if a database file is plain SQLite by reading its header.
fn is_plain_sqlite(path: &Path) -> bool {
    if let Ok(mut file) = std::fs::File::open(path) {
        use std::io::Read;
        let mut header = [0u8; 16];
        if file.read_exact(&mut header).is_ok() {
            // SQLite databases start with "SQLite format 3\0"
            return &header == b"SQLite format 3\0";
        }
    }
    false
}

pub fn init_db(data_dir: &Path) -> anyhow::Result<Connection> {
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("trcaa.db");

    let key = get_db_key(data_dir)?;

    let conn = if cfg!(debug_assertions) {
        open_dev_db(&db_path)?
    } else {
        // In release mode, try encrypted first
        match open_encrypted_db(&db_path, &key) {
            Ok(conn) => conn,
            Err(e) => {
                // Check if error is due to trying to decrypt a plain SQLite database
                if db_path.exists() && is_plain_sqlite(&db_path) {
                    // Auto-migrate from plain to encrypted
                    migrate_plain_to_encrypted(&db_path, &key)?
                } else {
                    // Different error - propagate it
                    return Err(e);
                }
            }
        }
    };

    crate::db::migrations::run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        use std::time::SystemTime;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("trcaa-test-{}-{}", name, timestamp));
        // Clean up if it exists
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_get_db_key_uses_env_var_when_present() {
        // Remove any existing env var first
        std::env::remove_var("TRCAA_DB_KEY");
        let dir = temp_dir("env-var");
        std::env::set_var("TRCAA_DB_KEY", "test-db-key");
        let key = get_db_key(&dir).unwrap();
        assert_eq!(key, "test-db-key");
        std::env::remove_var("TRCAA_DB_KEY");
    }

    #[test]
    fn test_get_db_key_debug_fallback_for_empty_env() {
        // Remove any existing env var first
        std::env::remove_var("TRCAA_DB_KEY");
        let dir = temp_dir("empty-env");
        std::env::set_var("TRCAA_DB_KEY", "   ");
        let key = get_db_key(&dir).unwrap();
        assert_eq!(key, "dev-key-change-in-prod");
        std::env::remove_var("TRCAA_DB_KEY");
    }

    #[test]
    fn test_is_plain_sqlite_detects_plain_database() {
        let dir = temp_dir("plain-detect");
        let db_path = dir.join("test.db");

        // Create a plain SQLite database
        let conn = Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE test (id INTEGER)", []).unwrap();
        drop(conn);

        assert!(is_plain_sqlite(&db_path));
    }

    #[test]
    fn test_is_plain_sqlite_rejects_encrypted() {
        let dir = temp_dir("encrypted-detect");
        let db_path = dir.join("test.db");

        // Create an encrypted database
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "PRAGMA key = 'test-key';\
             PRAGMA cipher_page_size = 16384;",
        )
        .unwrap();
        conn.execute("CREATE TABLE test (id INTEGER)", []).unwrap();
        drop(conn);

        assert!(!is_plain_sqlite(&db_path));
    }
}
