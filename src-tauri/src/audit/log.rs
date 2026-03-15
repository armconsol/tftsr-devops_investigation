use crate::db::models::AuditEntry;

/// Write an audit event to the audit_log table.
pub fn write_audit_event(
    conn: &rusqlite::Connection,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    details: &str,
) -> anyhow::Result<()> {
    let entry = AuditEntry::new(
        action.to_string(),
        entity_type.to_string(),
        entity_id.to_string(),
        details.to_string(),
    );
    conn.execute(
        "INSERT INTO audit_log (id, timestamp, action, entity_type, entity_id, user_id, details) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            entry.id,
            entry.timestamp,
            entry.action,
            entry.entity_type,
            entry.entity_id,
            entry.user_id,
            entry.details,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                entity_type TEXT NOT NULL DEFAULT '',
                entity_id TEXT NOT NULL DEFAULT '',
                user_id TEXT NOT NULL DEFAULT 'local',
                details TEXT NOT NULL DEFAULT '{}'
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_write_audit_event_inserts_row() {
        let conn = setup_test_db();
        write_audit_event(&conn, "test_action", "issue", "issue-123", r#"{"key":"val"}"#)
            .expect("should insert");

        let count: i64 = conn
            .prepare("SELECT COUNT(*) FROM audit_log")
            .unwrap()
            .query_row([], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_write_audit_event_correct_fields() {
        let conn = setup_test_db();
        write_audit_event(&conn, "create_issue", "issue", "abc-999", "details here")
            .expect("should insert");

        let (action, entity_type, entity_id, user_id): (String, String, String, String) = conn
            .prepare("SELECT action, entity_type, entity_id, user_id FROM audit_log LIMIT 1")
            .unwrap()
            .query_row([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap();

        assert_eq!(action, "create_issue");
        assert_eq!(entity_type, "issue");
        assert_eq!(entity_id, "abc-999");
        assert_eq!(user_id, "local");
    }

    #[test]
    fn test_write_multiple_events() {
        let conn = setup_test_db();
        for i in 0..5 {
            write_audit_event(
                &conn,
                &format!("action_{}", i),
                "test",
                &format!("id_{}", i),
                "{}",
            )
            .unwrap();
        }

        let count: i64 = conn
            .prepare("SELECT COUNT(*) FROM audit_log")
            .unwrap()
            .query_row([], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_write_audit_event_generates_unique_ids() {
        let conn = setup_test_db();
        write_audit_event(&conn, "a", "t", "1", "{}").unwrap();
        write_audit_event(&conn, "b", "t", "2", "{}").unwrap();

        let mut stmt = conn.prepare("SELECT id FROM audit_log").unwrap();
        let ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
    }
}
