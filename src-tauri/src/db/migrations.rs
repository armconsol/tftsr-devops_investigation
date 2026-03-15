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
                    tracing::warn!("FTS5 not available, skipping: {}", e);
                } else {
                    return Err(e.into());
                }
            }
            conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])?;
            tracing::info!("Applied migration: {}", name);
        }
    }

    Ok(())
}
