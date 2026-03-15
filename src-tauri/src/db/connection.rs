use rusqlite::Connection;
use std::path::Path;

pub fn open_encrypted_db(path: &Path, key: &str) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    // Set SQLCipher encryption key
    conn.execute_batch(&format!("PRAGMA key = '{}';", key.replace('\'', "''")))?;
    // Verify the key works by running a simple query
    conn.execute_batch("SELECT count(*) FROM sqlite_master;")?;
    // Set SQLCipher settings for AES-256
    conn.execute_batch(
        "PRAGMA cipher_page_size = 4096; \
         PRAGMA kdf_iter = 256000; \
         PRAGMA cipher_hmac_algorithm = HMAC_SHA512; \
         PRAGMA cipher_kdf_algorithm = PBKDF2_HMAC_SHA512;",
    )?;
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
    let key =
        std::env::var("TFTSR_DB_KEY").unwrap_or_else(|_| "dev-key-change-in-prod".to_string());

    let conn = if cfg!(debug_assertions) {
        open_dev_db(&db_path)?
    } else {
        open_encrypted_db(&db_path, &key)?
    };

    crate::db::migrations::run_migrations(&conn)?;
    Ok(conn)
}
