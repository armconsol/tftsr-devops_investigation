use rusqlite::Connection;

/// Run all database migrations in order, tracking which have been applied.
pub fn run_migrations(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let migrations: &[(&str, &str)] = &[
        (
            "001_create_issues",
            "CREATE TABLE IF NOT EXISTS issues (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                severity TEXT NOT NULL DEFAULT 'medium',
                status TEXT NOT NULL DEFAULT 'open',
                category TEXT NOT NULL DEFAULT 'general',
                source TEXT NOT NULL DEFAULT 'manual',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                resolved_at TEXT,
                assigned_to TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '[]'
            );",
        ),
        (
            "002_create_log_files",
            "CREATE TABLE IF NOT EXISTS log_files (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL DEFAULT '',
                file_size INTEGER NOT NULL DEFAULT 0,
                mime_type TEXT NOT NULL DEFAULT 'text/plain',
                content_hash TEXT NOT NULL DEFAULT '',
                uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),
                redacted INTEGER NOT NULL DEFAULT 0
            );",
        ),
        (
            "003_create_pii_spans",
            "CREATE TABLE IF NOT EXISTS pii_spans (
                id TEXT PRIMARY KEY,
                log_file_id TEXT NOT NULL REFERENCES log_files(id) ON DELETE CASCADE,
                pii_type TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                original_value TEXT NOT NULL,
                replacement TEXT NOT NULL
            );",
        ),
        (
            "004_create_ai_conversations",
            "CREATE TABLE IF NOT EXISTS ai_conversations (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                title TEXT NOT NULL DEFAULT 'Untitled'
            );",
        ),
        (
            "005_create_ai_messages",
            "CREATE TABLE IF NOT EXISTS ai_messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
                role TEXT NOT NULL CHECK(role IN ('system','user','assistant')),
                content TEXT NOT NULL,
                token_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        ),
        (
            "006_create_resolution_steps",
            "CREATE TABLE IF NOT EXISTS resolution_steps (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
                step_order INTEGER NOT NULL DEFAULT 0,
                why_question TEXT NOT NULL DEFAULT '',
                answer TEXT NOT NULL DEFAULT '',
                evidence TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        ),
        (
            "007_create_documents",
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
                doc_type TEXT NOT NULL,
                title TEXT NOT NULL,
                content_md TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        ),
        (
            "008_create_audit_log",
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                action TEXT NOT NULL,
                entity_type TEXT NOT NULL DEFAULT '',
                entity_id TEXT NOT NULL DEFAULT '',
                user_id TEXT NOT NULL DEFAULT 'local',
                details TEXT NOT NULL DEFAULT '{}'
            );",
        ),
        (
            "009_create_settings",
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        ),
        (
            "010_issues_fts",
            "CREATE VIRTUAL TABLE IF NOT EXISTS issues_fts USING fts5(
                id UNINDEXED, title, description,
                content='issues', content_rowid='rowid'
            );",
        ),
        (
            "011_create_integrations",
            "CREATE TABLE IF NOT EXISTS credentials (
                id TEXT PRIMARY KEY,
                service TEXT NOT NULL CHECK(service IN ('confluence','servicenow','azuredevops')),
                token_hash TEXT NOT NULL,
                encrypted_token TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at TEXT,
                UNIQUE(service)
            );
            CREATE TABLE IF NOT EXISTS integration_config (
                id TEXT PRIMARY KEY,
                service TEXT NOT NULL CHECK(service IN ('confluence','servicenow','azuredevops')),
                base_url TEXT NOT NULL,
                username TEXT,
                project_name TEXT,
                space_key TEXT,
                auto_create_enabled INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(service)
            );",
        ),
        (
            "012_audit_hash_chain",
            "ALTER TABLE audit_log ADD COLUMN prev_hash TEXT NOT NULL DEFAULT '';
             ALTER TABLE audit_log ADD COLUMN entry_hash TEXT NOT NULL DEFAULT '';",
        ),
        (
            "013_image_attachments",
            "CREATE TABLE IF NOT EXISTS image_attachments (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL DEFAULT '',
                file_size INTEGER NOT NULL DEFAULT 0,
                mime_type TEXT NOT NULL DEFAULT 'image/png',
                upload_hash TEXT NOT NULL DEFAULT '',
                uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),
                pii_warning_acknowledged INTEGER NOT NULL DEFAULT 1,
                is_paste INTEGER NOT NULL DEFAULT 0
            );",
        ),
    ];

    for (name, sql) in migrations {
        let already_applied: bool = conn
            .prepare("SELECT COUNT(*) FROM _migrations WHERE name = ?1")?
            .query_row([name], |row| row.get::<_, i64>(0))
            .map(|count| count > 0)?;

        if !already_applied {
            // FTS5 virtual table creation can be skipped if FTS5 is not compiled in
            if let Err(e) = conn.execute_batch(sql) {
                if name.contains("fts") {
                    tracing::warn!("FTS5 not available, skipping: {e}");
                } else {
                    return Err(e.into());
                }
            }
            conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])?;
            tracing::info!("Applied migration: {name}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_image_attachments_table() {
        let conn = setup_test_db();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='image_attachments'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let mut stmt = conn
            .prepare("PRAGMA table_info(image_attachments)")
            .unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"issue_id".to_string()));
        assert!(columns.contains(&"file_name".to_string()));
        assert!(columns.contains(&"file_path".to_string()));
        assert!(columns.contains(&"file_size".to_string()));
        assert!(columns.contains(&"mime_type".to_string()));
        assert!(columns.contains(&"upload_hash".to_string()));
        assert!(columns.contains(&"uploaded_at".to_string()));
        assert!(columns.contains(&"pii_warning_acknowledged".to_string()));
        assert!(columns.contains(&"is_paste".to_string()));
    }

    #[test]
    fn test_create_integration_config_table() {
        let conn = setup_test_db();

        // Verify table exists
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='integration_config'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify columns
        let mut stmt = conn
            .prepare("PRAGMA table_info(integration_config)")
            .unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"service".to_string()));
        assert!(columns.contains(&"base_url".to_string()));
        assert!(columns.contains(&"username".to_string()));
        assert!(columns.contains(&"project_name".to_string()));
        assert!(columns.contains(&"space_key".to_string()));
        assert!(columns.contains(&"auto_create_enabled".to_string()));
        assert!(columns.contains(&"updated_at".to_string()));
    }

    #[test]
    fn test_store_and_retrieve_credential() {
        let conn = setup_test_db();

        // Insert credential
        conn.execute(
            "INSERT INTO credentials (id, service, token_hash, encrypted_token, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                "test-id",
                "confluence",
                "test_hash",
                "encrypted_test",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
            ],
        )
        .unwrap();

        // Retrieve
        let (service, token_hash): (String, String) = conn
            .query_row(
                "SELECT service, token_hash FROM credentials WHERE service = ?1",
                ["confluence"],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();

        assert_eq!(service, "confluence");
        assert_eq!(token_hash, "test_hash");
    }

    #[test]
    fn test_store_and_retrieve_integration_config() {
        let conn = setup_test_db();

        // Insert config
        conn.execute(
            "INSERT INTO integration_config (id, service, base_url, space_key, auto_create_enabled, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                "test-config-id",
                "confluence",
                "https://example.atlassian.net",
                "DEV",
                1,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
            ],
        )
        .unwrap();

        // Retrieve
        let (service, base_url, space_key, auto_create): (String, String, String, i32) = conn
            .query_row(
                "SELECT service, base_url, space_key, auto_create_enabled FROM integration_config WHERE service = ?1",
                ["confluence"],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();

        assert_eq!(service, "confluence");
        assert_eq!(base_url, "https://example.atlassian.net");
        assert_eq!(space_key, "DEV");
        assert_eq!(auto_create, 1);
    }

    #[test]
    fn test_service_uniqueness_constraint() {
        let conn = setup_test_db();

        // Insert first credential
        conn.execute(
            "INSERT INTO credentials (id, service, token_hash, encrypted_token, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                "test-id-1",
                "confluence",
                "hash1",
                "token1",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
            ],
        )
        .unwrap();

        // Try to insert duplicate service - should fail
        let result = conn.execute(
            "INSERT INTO credentials (id, service, token_hash, encrypted_token, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                "test-id-2",
                "confluence",
                "hash2",
                "token2",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
            ],
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_migration_tracking() {
        let conn = setup_test_db();

        // Verify migration 011 was applied
        let applied: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
                ["011_create_integrations"],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(applied, 1);
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations twice
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        // Verify migration was only recorded once
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
                ["011_create_integrations"],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_store_and_retrieve_image_attachment() {
        let conn = setup_test_db();

        // Create an issue first (required for foreign key)
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO issues (id, title, description, severity, status, category, source, created_at, updated_at, resolved_at, assigned_to, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                "test-issue-1",
                "Test Issue",
                "Test description",
                "medium",
                "open",
                "test",
                "manual",
                now.clone(),
                now.clone(),
                None::<Option<String>>,
                "",
                "[]",
            ],
        )
        .unwrap();

        // Now insert the image attachment
        conn.execute(
            "INSERT INTO image_attachments (id, issue_id, file_name, file_path, file_size, mime_type, upload_hash, uploaded_at, pii_warning_acknowledged, is_paste)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                "test-img-1",
                "test-issue-1",
                "screenshot.png",
                "/path/to/screenshot.png",
                102400,
                "image/png",
                "abc123hash",
                now,
                1,
                0,
            ],
        )
        .unwrap();

        let (id, issue_id, file_name, mime_type, is_paste): (String, String, String, String, i32) = conn
            .query_row(
                "SELECT id, issue_id, file_name, mime_type, is_paste FROM image_attachments WHERE id = ?1",
                ["test-img-1"],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();

        assert_eq!(id, "test-img-1");
        assert_eq!(issue_id, "test-issue-1");
        assert_eq!(file_name, "screenshot.png");
        assert_eq!(mime_type, "image/png");
        assert_eq!(is_paste, 0);
    }
}
