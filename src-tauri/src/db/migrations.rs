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
        (
            "014_create_ai_providers",
            "CREATE TABLE IF NOT EXISTS ai_providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                provider_type TEXT NOT NULL,
                api_url TEXT NOT NULL,
                encrypted_api_key TEXT NOT NULL,
                model TEXT NOT NULL,
                max_tokens INTEGER,
                temperature REAL,
                custom_endpoint_path TEXT,
                custom_auth_header TEXT,
                custom_auth_prefix TEXT,
                api_format TEXT,
                user_id TEXT,
                use_datastore_upload INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        ),
        (
            "015_add_use_datastore_upload",
            "ALTER TABLE ai_providers ADD COLUMN use_datastore_upload INTEGER DEFAULT 0",
        ),
        (
            "016_add_created_at",
            "ALTER TABLE ai_providers ADD COLUMN created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))",
        ),
        (
            "017_create_timeline_events",
            "CREATE TABLE IF NOT EXISTS timeline_events (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
            );
            CREATE INDEX idx_timeline_events_issue ON timeline_events(issue_id);
            CREATE INDEX idx_timeline_events_time ON timeline_events(created_at);",
        ),
        (
            "018_mcp_servers",
            "CREATE TABLE IF NOT EXISTS mcp_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                transport_type TEXT NOT NULL CHECK(transport_type IN ('stdio', 'http')),
                transport_config TEXT NOT NULL DEFAULT '{}',
                auth_type TEXT NOT NULL CHECK(auth_type IN ('none', 'api_key', 'bearer', 'oauth2')),
                auth_value TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_discovered_at TEXT,
                discovery_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK(discovery_status IN ('pending','connected','unreachable','error')),
                discovery_error TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS mcp_tools (
                id TEXT PRIMARY KEY,
                server_id TEXT NOT NULL,
                name TEXT NOT NULL,
                tool_key TEXT NOT NULL,
                description TEXT,
                parameters TEXT NOT NULL DEFAULT '{}',
                FOREIGN KEY(server_id) REFERENCES mcp_servers(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS mcp_resources (
                id TEXT PRIMARY KEY,
                server_id TEXT NOT NULL,
                uri TEXT NOT NULL,
                name TEXT,
                description TEXT,
                FOREIGN KEY(server_id) REFERENCES mcp_servers(id) ON DELETE CASCADE
            );",
        ),
        (
            "019_create_sudo_config",
            "CREATE TABLE IF NOT EXISTS sudo_config (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL DEFAULT '',
                encrypted_password TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        ),
        (
            "020_add_log_content_compressed",
            "ALTER TABLE log_files ADD COLUMN content_compressed BLOB",
        ),
        (
            "021_add_image_data",
            "ALTER TABLE image_attachments ADD COLUMN image_data BLOB",
        ),
        (
            "022_attachment_views",
            "CREATE VIEW IF NOT EXISTS v_log_files_with_issue AS
                SELECT lf.id, lf.issue_id, lf.file_name, lf.file_path, lf.file_size,
                       lf.mime_type, lf.content_hash, lf.uploaded_at, lf.redacted,
                       i.title AS issue_title
                FROM log_files lf
                JOIN issues i ON i.id = lf.issue_id;
             CREATE VIEW IF NOT EXISTS v_image_attachments_with_issue AS
                SELECT ia.id, ia.issue_id, ia.file_name, ia.file_path, ia.file_size,
                       ia.mime_type, ia.upload_hash, ia.uploaded_at,
                       ia.pii_warning_acknowledged, ia.is_paste,
                       i.title AS issue_title
                FROM image_attachments ia
                JOIN issues i ON i.id = ia.issue_id;",
        ),
        (
            "023_add_mcp_env_config",
            "ALTER TABLE mcp_servers ADD COLUMN env_config TEXT",
        ),
        (
            "024_create_shell_commands",
            "CREATE TABLE IF NOT EXISTS shell_commands (
                id TEXT PRIMARY KEY,
                command_template TEXT NOT NULL,
                tier INTEGER NOT NULL CHECK(tier IN (1, 2, 3)),
                description TEXT,
                category TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            INSERT INTO shell_commands (id, command_template, tier, description, category) VALUES
            ('kubectl_get', 'kubectl get', 1, 'Read Kubernetes resources', 'kubectl'),
            ('kubectl_describe', 'kubectl describe', 1, 'Describe Kubernetes resources', 'kubectl'),
            ('kubectl_logs', 'kubectl logs', 1, 'View pod logs', 'kubectl'),
            ('kubectl_apply', 'kubectl apply', 2, 'Apply configuration', 'kubectl'),
            ('kubectl_delete', 'kubectl delete', 2, 'Delete resources', 'kubectl'),
            ('pvecm_status', 'pvecm status', 1, 'Check Proxmox cluster status', 'proxmox'),
            ('qm_status', 'qm status', 1, 'Check VM status', 'proxmox');",
        ),
        (
            "025_create_kubeconfig_files",
            "CREATE TABLE IF NOT EXISTS kubeconfig_files (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                encrypted_content TEXT NOT NULL,
                context TEXT NOT NULL,
                cluster_url TEXT,
                is_active INTEGER NOT NULL DEFAULT 0,
                uploaded_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_kubeconfig_active ON kubeconfig_files(is_active);",
        ),
        (
            "026_create_command_executions",
            "CREATE TABLE IF NOT EXISTS command_executions (
                id TEXT PRIMARY KEY,
                issue_id TEXT,
                command TEXT NOT NULL,
                tier INTEGER NOT NULL,
                approval_status TEXT NOT NULL,
                kubeconfig_id TEXT,
                exit_code INTEGER,
                stdout TEXT,
                stderr TEXT,
                execution_time_ms INTEGER,
                executed_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
                FOREIGN KEY (kubeconfig_id) REFERENCES kubeconfig_files(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_command_executions_issue ON command_executions(issue_id);
            CREATE INDEX IF NOT EXISTS idx_command_executions_executed ON command_executions(executed_at);",
        ),
        (
            "027_create_approval_decisions",
            "CREATE TABLE IF NOT EXISTS approval_decisions (
                id TEXT PRIMARY KEY,
                command_pattern TEXT NOT NULL,
                decision TEXT NOT NULL CHECK(decision IN ('allow_once', 'allow_session', 'deny')),
                session_id TEXT,
                decided_at TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_approval_decisions_session ON approval_decisions(session_id);",
        ),
        (
            "028_add_supports_tool_calling",
            "ALTER TABLE ai_providers ADD COLUMN supports_tool_calling INTEGER DEFAULT 1;
             -- Default to true for existing providers to maintain backward compatibility",
        ),
        (
            "029_create_clusters",
            "CREATE TABLE IF NOT EXISTS clusters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                context TEXT NOT NULL,
                server_url TEXT,
                kubeconfig_content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_clusters_name ON clusters(name);
            CREATE INDEX IF NOT EXISTS idx_clusters_context ON clusters(context);",
        ),
        (
            "030_create_port_forwards",
            "CREATE TABLE IF NOT EXISTS port_forwards (
                id TEXT PRIMARY KEY,
                cluster_id TEXT NOT NULL,
                namespace TEXT NOT NULL,
                pod TEXT NOT NULL,
                container TEXT,
                ports TEXT NOT NULL,
                local_ports TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'stopped', 'error')),
                error_message TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_port_forwards_cluster ON port_forwards(cluster_id);
            CREATE INDEX IF NOT EXISTS idx_port_forwards_status ON port_forwards(status);
            CREATE INDEX IF NOT EXISTS idx_port_forwards_namespace ON port_forwards(namespace);",
        ),
        (
            "031_create_proxmox_clusters",
            "CREATE TABLE IF NOT EXISTS proxmox_clusters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                cluster_type TEXT NOT NULL CHECK(cluster_type IN ('ve', 'pbs')),
                url TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 8006,
                auth_method TEXT NOT NULL DEFAULT 'root',
                encrypted_credentials TEXT NOT NULL,
                ssl_fingerprint TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_proxmox_clusters_name ON proxmox_clusters(name);
            CREATE INDEX IF NOT EXISTS idx_proxmox_clusters_type ON proxmox_clusters(cluster_type);",
        ),
        (
            "032_create_proxmox_resources",
            "CREATE TABLE IF NOT EXISTS proxmox_resources (
                id TEXT PRIMARY KEY,
                cluster_id TEXT NOT NULL REFERENCES proxmox_clusters(id) ON DELETE CASCADE,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                resource_data TEXT NOT NULL DEFAULT '{}',
                last_updated TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(cluster_id, resource_type, resource_id)
            );
            CREATE INDEX IF NOT EXISTS idx_proxmox_resources_cluster ON proxmox_resources(cluster_id);
            CREATE INDEX IF NOT EXISTS idx_proxmox_resources_type ON proxmox_resources(resource_type);
            CREATE INDEX IF NOT EXISTS idx_proxmox_resources_updated ON proxmox_resources(last_updated);",
        ),
        (
            "033_cleanup_old_dummy_data",
            "DELETE FROM proxmox_clusters WHERE name LIKE '%example%' OR name LIKE '%test%' OR name LIKE '%dummy%' OR name LIKE '%sample%';
            DELETE FROM proxmox_resources WHERE cluster_id IN (
                SELECT id FROM proxmox_clusters WHERE name LIKE '%example%' OR name LIKE '%test%' OR name LIKE '%dummy%' OR name LIKE '%sample%'
            );",
        ),
        (
            "034_add_proxmox_username_column",
            "ALTER TABLE proxmox_clusters ADD COLUMN username TEXT NOT NULL DEFAULT '';",
        ),
        (
            "035_create_remote_connections",
            "CREATE TABLE IF NOT EXISTS remote_connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                protocol TEXT NOT NULL CHECK(protocol IN ('rdp', 'vnc')),
                hostname TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 3389,
                username TEXT,
                password_encrypted TEXT NOT NULL,
                domain TEXT,
                resolution TEXT NOT NULL DEFAULT '1280x800',
                color_depth INTEGER NOT NULL DEFAULT 32,
                clipboard_sync INTEGER NOT NULL DEFAULT 1,
                drive_redirect INTEGER NOT NULL DEFAULT 0,
                multi_monitor INTEGER NOT NULL DEFAULT 0,
                compression INTEGER NOT NULL DEFAULT 1,
                quality INTEGER NOT NULL DEFAULT 80,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_connected_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_remote_connections_protocol ON remote_connections(protocol);
            CREATE INDEX IF NOT EXISTS idx_remote_connections_last_connected ON remote_connections(last_connected_at);",
        ),
        (
            "035a_upgrade_remote_connections_ssh",
            "DROP TABLE IF EXISTS remote_connections;
            CREATE TABLE IF NOT EXISTS remote_connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                protocol TEXT NOT NULL CHECK(protocol IN ('rdp', 'vnc')),
                hostname TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT,
                domain TEXT,
                resolution TEXT NOT NULL DEFAULT 'auto',
                color_depth INTEGER NOT NULL DEFAULT 32,
                clipboard_sync INTEGER NOT NULL DEFAULT 1,
                drive_redirect INTEGER NOT NULL DEFAULT 0,
                multi_monitor INTEGER NOT NULL DEFAULT 0,
                compression INTEGER NOT NULL DEFAULT 1,
                quality INTEGER NOT NULL DEFAULT 80,
                ssh_enabled INTEGER NOT NULL DEFAULT 0,
                ssh_hostname TEXT,
                ssh_port INTEGER DEFAULT 22,
                ssh_username TEXT,
                auto_resize INTEGER NOT NULL DEFAULT 1,
                stretch_to_fill INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_connected_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_remote_connections_protocol ON remote_connections(protocol);
            CREATE INDEX IF NOT EXISTS idx_remote_connections_ssh ON remote_connections(ssh_enabled);",
        ),
        (
            "036_create_remote_credentials",
            "CREATE TABLE IF NOT EXISTS remote_credentials (
                id TEXT PRIMARY KEY,
                connection_id TEXT NOT NULL REFERENCES remote_connections(id) ON DELETE CASCADE,
                rdp_password_encrypted TEXT,
                ssh_password_encrypted TEXT,
                ssh_key_encrypted TEXT,
                ssh_key_passphrase_encrypted TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(connection_id)
            );",
        ),
        (
            "037_create_ssh_credentials",
            "CREATE TABLE IF NOT EXISTS ssh_credentials (
                key_id TEXT PRIMARY KEY,
                encrypted_data TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        ),
        (
            "038_create_database_connections",
            "CREATE TABLE IF NOT EXISTS database_connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                db_type TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                database_name TEXT,
                username TEXT,
                encrypted_password TEXT,
                ssl_enabled INTEGER DEFAULT 0,
                ssl_ca_cert_path TEXT,
                ssl_client_cert_path TEXT,
                ssl_client_key_path TEXT,
                connection_options TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_db_connections_type ON database_connections(db_type);",
        ),
        (
            "039_create_query_history",
            "CREATE TABLE IF NOT EXISTS query_history (
                id TEXT PRIMARY KEY,
                connection_id TEXT NOT NULL,
                query_text TEXT NOT NULL,
                row_count INTEGER,
                execution_time_ms INTEGER,
                status TEXT NOT NULL,
                error_message TEXT,
                executed_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (connection_id) REFERENCES database_connections(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_query_history_connection ON query_history(connection_id);
            CREATE INDEX IF NOT EXISTS idx_query_history_executed ON query_history(executed_at DESC);",
        ),
        (
            "040_create_query_bookmarks",
            "CREATE TABLE IF NOT EXISTS query_bookmarks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                query_text TEXT NOT NULL,
                connection_id TEXT,
                tags TEXT,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (connection_id) REFERENCES database_connections(id) ON DELETE SET NULL
            );
            CREATE INDEX IF NOT EXISTS idx_query_bookmarks_connection ON query_bookmarks(connection_id);",
        ),
        (
            "041_add_database_ssh_tunnel_columns",
            "ALTER TABLE database_connections ADD COLUMN ssh_enabled INTEGER DEFAULT 0;
             ALTER TABLE database_connections ADD COLUMN ssh_hostname TEXT;
             ALTER TABLE database_connections ADD COLUMN ssh_port INTEGER;
             ALTER TABLE database_connections ADD COLUMN ssh_username TEXT;
             ALTER TABLE database_connections ADD COLUMN ssh_auth_method TEXT;",
        ),
        (
            // 041 added the SSH tunnel config columns but not the encrypted
            // credential columns referenced by create/update_database_connection.
            "042_add_database_ssh_credential_columns",
            "ALTER TABLE database_connections ADD COLUMN ssh_password_encrypted TEXT;
             ALTER TABLE database_connections ADD COLUMN ssh_private_key_encrypted TEXT;
             ALTER TABLE database_connections ADD COLUMN ssh_key_passphrase_encrypted TEXT;",
        ),
    ];

    for (name, sql) in migrations {
        let already_applied: bool = conn
            .prepare("SELECT COUNT(*) FROM _migrations WHERE name = ?1")?
            .query_row([name], |row| row.get::<_, i64>(0))
            .map(|count| count > 0)?;

        if !already_applied {
            // FTS5 virtual table creation can be skipped if FTS5 is not compiled in
            // Also handle column-already-exists errors for migrations 015-016
            if name.contains("fts") {
                if let Err(e) = conn.execute_batch(sql) {
                    tracing::warn!("FTS5 not available, skipping: {e}");
                }
            } else if name.ends_with("_add_use_datastore_upload")
                || name.ends_with("_add_created_at")
                || name.ends_with("_add_log_content_compressed")
                || name.ends_with("_add_image_data")
                || name.ends_with("_add_supports_tool_calling")
                || name.ends_with("_add_proxmox_username_column")
                || name.ends_with("_add_database_ssh_tunnel_columns")
                || name.ends_with("_add_database_ssh_credential_columns")
            {
                // Use execute for ALTER TABLE (SQLite only allows one statement per command)
                // Skip error if column already exists (SQLITE_ERROR with "duplicate column name")
                if let Err(e) = conn.execute_batch(sql) {
                    let err_str = e.to_string();
                    if err_str.contains("duplicate column name") {
                        tracing::info!("Column may already exist, skipping migration {name}: {e}");
                    } else {
                        return Err(e.into());
                    }
                }
            } else {
                // Use execute_batch for other migrations (FTS5, CREATE TABLE, etc.)
                if let Err(e) = conn.execute_batch(sql) {
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

    #[test]
    fn test_create_ai_providers_table() {
        let conn = setup_test_db();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ai_providers'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let mut stmt = conn.prepare("PRAGMA table_info(ai_providers)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"name".to_string()));
        assert!(columns.contains(&"provider_type".to_string()));
        assert!(columns.contains(&"api_url".to_string()));
        assert!(columns.contains(&"encrypted_api_key".to_string()));
        assert!(columns.contains(&"model".to_string()));
        assert!(columns.contains(&"max_tokens".to_string()));
        assert!(columns.contains(&"temperature".to_string()));
        assert!(columns.contains(&"custom_endpoint_path".to_string()));
        assert!(columns.contains(&"custom_auth_header".to_string()));
        assert!(columns.contains(&"custom_auth_prefix".to_string()));
        assert!(columns.contains(&"api_format".to_string()));
        assert!(columns.contains(&"user_id".to_string()));
        assert!(columns.contains(&"use_datastore_upload".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
        assert!(columns.contains(&"updated_at".to_string()));
    }

    #[test]
    fn test_store_and_retrieve_ai_provider() {
        let conn = setup_test_db();

        conn.execute(
            "INSERT INTO ai_providers (id, name, provider_type, api_url, encrypted_api_key, model)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                "test-provider-1",
                "My OpenAI",
                "openai",
                "https://api.openai.com/v1",
                "encrypted_key_123",
                "gpt-4o"
            ],
        )
        .unwrap();

        let (name, provider_type, api_url, encrypted_key, model): (String, String, String, String, String) = conn
            .query_row(
                "SELECT name, provider_type, api_url, encrypted_api_key, model FROM ai_providers WHERE name = ?1",
                ["My OpenAI"],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();

        assert_eq!(name, "My OpenAI");
        assert_eq!(provider_type, "openai");
        assert_eq!(api_url, "https://api.openai.com/v1");
        assert_eq!(encrypted_key, "encrypted_key_123");
        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_add_missing_columns_to_existing_table() {
        let conn = Connection::open_in_memory().unwrap();

        // Simulate existing table without use_datastore_upload and created_at
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS ai_providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                provider_type TEXT NOT NULL,
                api_url TEXT NOT NULL,
                encrypted_api_key TEXT NOT NULL,
                model TEXT NOT NULL,
                max_tokens INTEGER,
                temperature REAL,
                custom_endpoint_path TEXT,
                custom_auth_header TEXT,
                custom_auth_prefix TEXT,
                api_format TEXT,
                user_id TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();

        // Verify columns BEFORE migration
        let mut stmt = conn.prepare("PRAGMA table_info(ai_providers)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"name".to_string()));
        assert!(columns.contains(&"model".to_string()));
        assert!(!columns.contains(&"use_datastore_upload".to_string()));
        assert!(!columns.contains(&"created_at".to_string()));

        // Run migrations (should apply 015 to add missing columns)
        run_migrations(&conn).unwrap();

        // Verify columns AFTER migration
        let mut stmt = conn.prepare("PRAGMA table_info(ai_providers)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"name".to_string()));
        assert!(columns.contains(&"model".to_string()));
        assert!(columns.contains(&"use_datastore_upload".to_string()));
        assert!(columns.contains(&"created_at".to_string()));

        // Verify data integrity - existing rows should have default values
        conn.execute(
            "INSERT INTO ai_providers (id, name, provider_type, api_url, encrypted_api_key, model)
             VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "test-provider-2",
                "Test Provider",
                "openai",
                "https://api.example.com",
                "encrypted_key_456",
                "gpt-3.5-turbo"
            ],
        )
        .unwrap();

        let (name, use_datastore_upload, created_at): (String, bool, String) = conn
            .query_row(
                "SELECT name, use_datastore_upload, created_at FROM ai_providers WHERE name = ?1",
                ["Test Provider"],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();

        assert_eq!(name, "Test Provider");
        assert!(!use_datastore_upload);
        assert!(created_at.len() > 0);
    }

    #[test]
    fn test_idempotent_add_missing_columns() {
        let conn = Connection::open_in_memory().unwrap();

        // Create table with both columns already present (simulating prior migration run)
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS ai_providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                provider_type TEXT NOT NULL,
                api_url TEXT NOT NULL,
                encrypted_api_key TEXT NOT NULL,
                model TEXT NOT NULL,
                max_tokens INTEGER,
                temperature REAL,
                custom_endpoint_path TEXT,
                custom_auth_header TEXT,
                custom_auth_prefix TEXT,
                api_format TEXT,
                user_id TEXT,
                use_datastore_upload INTEGER DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();

        // Should not fail even though columns already exist
        run_migrations(&conn).unwrap();
    }

    #[test]
    fn test_timeline_events_table_exists() {
        let conn = setup_test_db();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='timeline_events'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let mut stmt = conn.prepare("PRAGMA table_info(timeline_events)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"issue_id".to_string()));
        assert!(columns.contains(&"event_type".to_string()));
        assert!(columns.contains(&"description".to_string()));
        assert!(columns.contains(&"metadata".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
    }

    #[test]
    fn test_timeline_events_cascade_delete() {
        let conn = setup_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO issues (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["issue-1", "Test Issue", now, now],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO timeline_events (id, issue_id, event_type, description, metadata, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["te-1", "issue-1", "triage_started", "Started triage", "{}", "2025-01-15 10:00:00 UTC"],
        )
        .unwrap();

        // Verify event exists
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM timeline_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Delete issue — cascade should remove timeline event
        conn.execute("DELETE FROM issues WHERE id = 'issue-1'", [])
            .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM timeline_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_timeline_events_indexes() {
        let conn = setup_test_db();
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='timeline_events'",
            )
            .unwrap();
        let indexes: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(indexes.contains(&"idx_timeline_events_issue".to_string()));
        assert!(indexes.contains(&"idx_timeline_events_time".to_string()));
    }

    // ─── Migration 018: mcp_servers / mcp_tools / mcp_resources ─────────────

    #[test]
    fn test_018_migration_mcp_tables() {
        let conn = setup_test_db();

        for table in &["mcp_servers", "mcp_tools", "mcp_resources"] {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "table {table} should exist");
        }

        let mut stmt = conn.prepare("PRAGMA table_info(mcp_servers)").unwrap();
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for col in &[
            "id",
            "name",
            "url",
            "transport_type",
            "transport_config",
            "auth_type",
            "auth_value",
            "enabled",
            "last_discovered_at",
            "discovery_status",
            "discovery_error",
            "created_at",
            "updated_at",
        ] {
            assert!(
                cols.contains(&col.to_string()),
                "mcp_servers missing column {col}"
            );
        }

        let mut stmt = conn.prepare("PRAGMA table_info(mcp_tools)").unwrap();
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for col in &[
            "id",
            "server_id",
            "name",
            "tool_key",
            "description",
            "parameters",
        ] {
            assert!(
                cols.contains(&col.to_string()),
                "mcp_tools missing column {col}"
            );
        }

        let mut stmt = conn.prepare("PRAGMA table_info(mcp_resources)").unwrap();
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for col in &["id", "server_id", "uri", "name", "description"] {
            assert!(
                cols.contains(&col.to_string()),
                "mcp_resources missing column {col}"
            );
        }
    }

    #[test]
    fn test_018_mcp_servers_check_constraints() {
        let conn = setup_test_db();

        // Valid insert should succeed
        conn.execute(
            "INSERT INTO mcp_servers (id, name, url, transport_type, auth_type)
             VALUES ('s1', 'My Server', 'http://localhost:8080/mcp', 'http', 'none')",
            [],
        )
        .unwrap();

        // Invalid transport_type must fail
        let err = conn.execute(
            "INSERT INTO mcp_servers (id, name, url, transport_type, auth_type)
             VALUES ('s2', 'Bad Transport', '', 'websocket', 'none')",
            [],
        );
        assert!(err.is_err(), "invalid transport_type should be rejected");

        // Invalid auth_type must fail
        let err = conn.execute(
            "INSERT INTO mcp_servers (id, name, url, transport_type, auth_type)
             VALUES ('s3', 'Bad Auth', '', 'stdio', 'password')",
            [],
        );
        assert!(err.is_err(), "invalid auth_type should be rejected");

        // Invalid discovery_status must fail
        let err = conn.execute(
            "INSERT INTO mcp_servers (id, name, url, transport_type, auth_type, discovery_status)
             VALUES ('s4', 'Bad Status', '', 'stdio', 'none', 'unknown')",
            [],
        );
        assert!(err.is_err(), "invalid discovery_status should be rejected");
    }

    #[test]
    fn test_018_mcp_tools_cascade_delete() {
        let conn = setup_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        conn.execute(
            "INSERT INTO mcp_servers (id, name, url, transport_type, auth_type)
             VALUES ('srv-1', 'Test', 'http://localhost/mcp', 'http', 'none')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO mcp_tools (id, server_id, name, tool_key)
             VALUES ('tool-1', 'srv-1', 'echo', 'mcp_test_echo')",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM mcp_tools", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);

        conn.execute("DELETE FROM mcp_servers WHERE id = 'srv-1'", [])
            .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM mcp_tools", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "cascade delete should remove mcp_tools");
    }

    #[test]
    fn test_018_mcp_resources_cascade_delete() {
        let conn = setup_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        conn.execute(
            "INSERT INTO mcp_servers (id, name, url, transport_type, auth_type)
             VALUES ('srv-2', 'Test', 'http://localhost/mcp', 'http', 'none')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO mcp_resources (id, server_id, uri)
             VALUES ('res-1', 'srv-2', 'file:///tmp/data.txt')",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM mcp_resources", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);

        conn.execute("DELETE FROM mcp_servers WHERE id = 'srv-2'", [])
            .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM mcp_resources", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "cascade delete should remove mcp_resources");
    }

    #[test]
    fn test_018_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        for table in &["mcp_servers", "mcp_tools", "mcp_resources"] {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "table {table} should exist after double-run");
        }

        let applied: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = '018_mcp_servers'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(applied, 1, "018 should only be recorded once");
    }

    // ─── Migration 019: sudo_config ─────────────────────────────────────────────

    #[test]
    fn test_019_sudo_config_table_exists() {
        let conn = setup_test_db();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sudo_config'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_019_sudo_config_columns() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("PRAGMA table_info(sudo_config)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"username".to_string()));
        assert!(columns.contains(&"encrypted_password".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
        assert!(columns.contains(&"updated_at".to_string()));
    }

    #[test]
    fn test_019_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sudo_config'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "sudo_config table should exist after double-run");

        let applied: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = '019_create_sudo_config'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(applied, 1, "019 should only be recorded once");
    }

    // ─── Migration 020-022: attachment content storage ──────────────────────────

    #[test]
    fn test_020_log_content_compressed_column() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("PRAGMA table_info(log_files)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(
            columns.contains(&"content_compressed".to_string()),
            "log_files should have content_compressed column"
        );
    }

    #[test]
    fn test_021_image_data_column() {
        let conn = setup_test_db();
        let mut stmt = conn
            .prepare("PRAGMA table_info(image_attachments)")
            .unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(
            columns.contains(&"image_data".to_string()),
            "image_attachments should have image_data column"
        );
    }

    #[test]
    fn test_022_attachment_views_exist() {
        let conn = setup_test_db();
        for view in &["v_log_files_with_issue", "v_image_attachments_with_issue"] {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name=?1",
                    [view],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "view {view} should exist");
        }
    }

    #[test]
    fn test_022_views_join_issue_title() {
        let conn = setup_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            "INSERT INTO issues (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["issue-view-1", "Disk Full Alert", now.clone(), now.clone()],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO log_files (id, issue_id, file_name, file_path, uploaded_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                "lf-1",
                "issue-view-1",
                "syslog.log",
                "/var/log/syslog",
                now.clone()
            ],
        )
        .unwrap();

        let issue_title: String = conn
            .query_row(
                "SELECT issue_title FROM v_log_files_with_issue WHERE id = 'lf-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(issue_title, "Disk Full Alert");
    }

    #[test]
    fn test_020_021_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        for migration in &[
            "020_add_log_content_compressed",
            "021_add_image_data",
            "022_attachment_views",
        ] {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
                    [migration],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "{migration} should be recorded exactly once");
        }
    }

    // ─── Migration 023: MCP env_config ──────────────────────────────────────────

    #[test]
    fn test_023_mcp_env_config_column() {
        let conn = setup_test_db();

        let mut stmt = conn.prepare("PRAGMA table_info(mcp_servers)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(
            columns.contains(&"env_config".to_string()),
            "mcp_servers table should have env_config column after migration 023"
        );
    }

    #[test]
    fn test_023_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let applied: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = '023_add_mcp_env_config'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(applied, 1, "023 should only be recorded once");
    }

    // ─── Migration 029-030: Kubernetes clusters and port_forwards ───────────────

    #[test]
    fn test_029_clusters_table_exists() {
        let conn = setup_test_db();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='clusters'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_029_clusters_columns() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("PRAGMA table_info(clusters)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"name".to_string()));
        assert!(columns.contains(&"context".to_string()));
        assert!(columns.contains(&"server_url".to_string()));
        assert!(columns.contains(&"kubeconfig_content".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
        assert!(columns.contains(&"updated_at".to_string()));
    }

    #[test]
    fn test_029_clusters_foreign_key() {
        let conn = setup_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Create cluster with embedded kubeconfig
        let kubeconfig = "apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://k8s.example.com
  name: cluster-1
contexts:
- context:
    cluster: cluster-1
    user: user-1
  name: context-1
users:
- name: user-1
  user:
    token: test-token
";
        conn.execute(
            "INSERT INTO clusters (id, name, context, server_url, kubeconfig_content)
             VALUES ('cluster-1', 'Production', 'context-1', 'https://k8s.example.com', ?1)",
            [kubeconfig],
        )
        .unwrap();

        // Verify insertion
        let (name, context, server_url, kubeconfig_content): (String, String, String, String) = conn
            .query_row(
                "SELECT name, context, server_url, kubeconfig_content FROM clusters WHERE id = 'cluster-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();

        assert_eq!(name, "Production");
        assert_eq!(context, "context-1");
        assert_eq!(server_url, "https://k8s.example.com");
        assert!(kubeconfig_content.contains("k8s.example.com"));
    }

    #[test]
    fn test_030_port_forwards_table_exists() {
        let conn = setup_test_db();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='port_forwards'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_030_port_forwards_columns() {
        let conn = setup_test_db();
        let mut stmt = conn.prepare("PRAGMA table_info(port_forwards)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"cluster_id".to_string()));
        assert!(columns.contains(&"namespace".to_string()));
        assert!(columns.contains(&"pod".to_string()));
        assert!(columns.contains(&"container".to_string()));
        assert!(columns.contains(&"ports".to_string()));
        assert!(columns.contains(&"local_ports".to_string()));
        assert!(columns.contains(&"status".to_string()));
        assert!(columns.contains(&"error_message".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
        assert!(columns.contains(&"updated_at".to_string()));
    }

    #[test]
    fn test_030_port_forwards_status_constraint() {
        let conn = setup_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Create kubeconfig first
        conn.execute(
            "INSERT INTO kubeconfig_files (id, name, encrypted_content, context)
             VALUES ('k8s-test', 'Test Cluster', 'encrypted', 'test-context')",
            [],
        )
        .unwrap();

        // Create cluster
        conn.execute(
            "INSERT INTO clusters (id, name, context, kubeconfig_content)
             VALUES ('cluster-1', 'Test', 'test-context', 'k8s-test')",
            [],
        )
        .unwrap();

        // Valid status should succeed
        conn.execute(
            "INSERT INTO port_forwards (id, cluster_id, namespace, pod, ports, local_ports, status)
             VALUES ('pf-1', 'cluster-1', 'default', 'pod-1', '[8080]', '[0]', 'active')",
            [],
        )
        .unwrap();

        // Invalid status must fail
        let err = conn.execute(
            "INSERT INTO port_forwards (id, cluster_id, namespace, pod, ports, local_ports, status)
             VALUES ('pf-2', 'cluster-1', 'default', 'pod-2', '[8080]', '[0]', 'unknown')",
            [],
        );
        assert!(err.is_err(), "invalid status should be rejected");
    }

    #[test]
    fn test_030_port_forwards_cascade_delete() {
        let conn = setup_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Create kubeconfig first
        conn.execute(
            "INSERT INTO kubeconfig_files (id, name, encrypted_content, context)
             VALUES ('k8s-3', 'Test Cluster', 'encrypted', 'ctx')",
            [],
        )
        .unwrap();

        // Create cluster
        conn.execute(
            "INSERT INTO clusters (id, name, context, kubeconfig_content)
             VALUES ('cluster-3', 'Test', 'ctx', 'k8s-3')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO port_forwards (id, cluster_id, namespace, pod, ports, local_ports)
             VALUES ('pf-3', 'cluster-3', 'default', 'pod-3', '[8080]', '[0]')",
            [],
        )
        .unwrap();

        // Verify port forward exists
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM port_forwards", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Delete cluster — cascade should remove port forward
        conn.execute("DELETE FROM clusters WHERE id = 'cluster-3'", [])
            .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM port_forwards", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "cascade delete should remove port_forwards");
    }

    #[test]
    fn test_029_030_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        for migration in &["029_create_clusters", "030_create_port_forwards"] {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
                    [migration],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "{migration} should be recorded exactly once");
        }
    }

    fn database_connections_columns(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("PRAGMA table_info(database_connections)")
            .unwrap();
        stmt.query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    #[test]
    fn test_042_database_connections_has_ssh_credential_columns() {
        let conn = setup_test_db();
        let columns = database_connections_columns(&conn);

        for col in [
            "ssh_password_encrypted",
            "ssh_private_key_encrypted",
            "ssh_key_passphrase_encrypted",
        ] {
            assert!(
                columns.contains(&col.to_string()),
                "database_connections should have column {col}, got: {columns:?}"
            );
        }
    }

    #[test]
    fn test_042_idempotent_rerun() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
                ["042_add_database_ssh_credential_columns"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "migration 042 should be recorded exactly once");
    }

    #[test]
    fn test_042_applies_to_existing_v041_database() {
        // Simulate an existing installation: table created by 038 + columns from
        // 041, with both migrations recorded, then run_migrations must add the
        // credential columns without error.
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();
        run_migrations(&conn).unwrap();
        // Drop the credential columns to mimic a DB that predates migration 042
        conn.execute_batch(
            "ALTER TABLE database_connections DROP COLUMN ssh_password_encrypted;
             ALTER TABLE database_connections DROP COLUMN ssh_private_key_encrypted;
             ALTER TABLE database_connections DROP COLUMN ssh_key_passphrase_encrypted;
             DELETE FROM _migrations WHERE name = '042_add_database_ssh_credential_columns';",
        )
        .unwrap();
        assert!(
            !database_connections_columns(&conn).contains(&"ssh_password_encrypted".to_string())
        );

        run_migrations(&conn).unwrap();

        let columns = database_connections_columns(&conn);
        for col in [
            "ssh_password_encrypted",
            "ssh_private_key_encrypted",
            "ssh_key_passphrase_encrypted",
        ] {
            assert!(
                columns.contains(&col.to_string()),
                "missing {col} after re-migration"
            );
        }
    }

    #[test]
    fn test_042_insert_with_ssh_credentials_succeeds() {
        // Mirrors the INSERT in commands/database.rs::create_database_connection
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO database_connections (
                id, name, db_type, host, port, database_name, username, encrypted_password,
                ssl_enabled, ssl_ca_cert_path, ssl_client_cert_path, ssl_client_key_path, connection_options,
                ssh_enabled, ssh_hostname, ssh_port, ssh_username, ssh_auth_method,
                ssh_password_encrypted, ssh_private_key_encrypted, ssh_key_passphrase_encrypted,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                ?9, ?10, ?11, ?12, ?13,
                ?14, ?15, ?16, ?17, ?18,
                ?19, ?20, ?21,
                ?22, ?23
            )",
            rusqlite::params![
                "conn-1",
                "Test PSQL",
                "postgresql",
                "db.example.com",
                5432,
                "appdb",
                "app",
                "enc:password",
                0,
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                1,
                "bastion.example.com",
                22,
                "tunnel",
                "password",
                "enc:sshpw",
                Option::<String>::None,
                Option::<String>::None,
                "2026-07-04 00:00:00",
                "2026-07-04 00:00:00",
            ],
        )
        .expect("INSERT with ssh credential columns must succeed");

        let stored: String = conn
            .query_row(
                "SELECT ssh_password_encrypted FROM database_connections WHERE id = 'conn-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stored, "enc:sshpw");
    }
}
