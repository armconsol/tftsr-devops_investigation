use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::integrations::auth::{decrypt_token, encrypt_token};
use crate::mcp::models::{
    CreateMcpServerRequest, McpResource, McpServer, McpTool, UpdateMcpServerRequest,
};

pub fn create_server(conn: &Connection, req: &CreateMcpServerRequest) -> Result<McpServer, String> {
    let id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let encrypted_auth = match &req.auth_value {
        Some(v) if !v.is_empty() => Some(encrypt_token(v)?),
        _ => None,
    };

    let encrypted_env = match &req.env_config {
        Some(env_json) if !env_json.trim().is_empty() => Some(encrypt_token(env_json)?),
        _ => None,
    };

    conn.execute(
        "INSERT INTO mcp_servers
            (id, name, url, transport_type, transport_config, auth_type, auth_value, enabled, env_config, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        rusqlite::params![
            id,
            req.name,
            req.url,
            req.transport_type,
            req.transport_config,
            req.auth_type,
            encrypted_auth,
            req.enabled as i32,
            encrypted_env,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    get_server(conn, &id)?.ok_or_else(|| "Server not found after insert".to_string())
}

pub fn get_server(conn: &Connection, id: &str) -> Result<Option<McpServer>, String> {
    let row = conn
        .query_row(
            "SELECT id, name, url, transport_type, transport_config, auth_type, auth_value,
                    enabled, last_discovered_at, discovery_status, discovery_error,
                    created_at, updated_at, env_config
             FROM mcp_servers WHERE id = ?1",
            [id],
            |row| {
                Ok(McpServer {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    transport_type: row.get(3)?,
                    transport_config: row.get(4)?,
                    auth_type: row.get(5)?,
                    auth_value: row.get(6)?,
                    enabled: row.get::<_, i32>(7)? != 0,
                    last_discovered_at: row.get(8)?,
                    discovery_status: row.get(9)?,
                    discovery_error: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    env_config: row.get(13)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(row)
}

pub fn list_servers(conn: &Connection) -> Result<Vec<McpServer>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, url, transport_type, transport_config, auth_type, auth_value,
                    enabled, last_discovered_at, discovery_status, discovery_error,
                    created_at, updated_at, env_config
             FROM mcp_servers ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(McpServer {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                transport_type: row.get(3)?,
                transport_config: row.get(4)?,
                auth_type: row.get(5)?,
                auth_value: row.get(6)?,
                enabled: row.get::<_, i32>(7)? != 0,
                last_discovered_at: row.get(8)?,
                discovery_status: row.get(9)?,
                discovery_error: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                env_config: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

pub fn update_server(
    conn: &Connection,
    id: &str,
    req: &UpdateMcpServerRequest,
) -> Result<McpServer, String> {
    let existing = get_server(conn, id)?.ok_or_else(|| format!("Server {id} not found"))?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let new_encrypted_auth = match &req.auth_value {
        Some(v) if !v.is_empty() => Some(encrypt_token(v)?),
        Some(_) => None,
        None => existing.auth_value.clone(),
    };

    let new_encrypted_env = match &req.env_config {
        Some(env_json) if !env_json.trim().is_empty() => Some(encrypt_token(env_json)?),
        Some(_) => None,  // Empty string = clear env_config
        None => existing.env_config.clone(),  // No update requested
    };

    conn.execute(
        "UPDATE mcp_servers SET
            name = ?1, url = ?2, transport_type = ?3, transport_config = ?4,
            auth_type = ?5, auth_value = ?6, enabled = ?7, env_config = ?8, updated_at = ?9
         WHERE id = ?10",
        rusqlite::params![
            req.name.as_deref().unwrap_or(&existing.name),
            req.url.as_deref().unwrap_or(&existing.url),
            req.transport_type
                .as_deref()
                .unwrap_or(&existing.transport_type),
            req.transport_config
                .as_deref()
                .unwrap_or(&existing.transport_config),
            req.auth_type.as_deref().unwrap_or(&existing.auth_type),
            new_encrypted_auth,
            req.enabled
                .map(|b| b as i32)
                .unwrap_or(existing.enabled as i32),
            new_encrypted_env,
            now,
            id,
        ],
    )
    .map_err(|e| e.to_string())?;

    get_server(conn, id)?.ok_or_else(|| "Server not found after update".to_string())
}

pub fn delete_server(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM mcp_servers WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn toggle_server(conn: &Connection, id: &str, enabled: bool) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE mcp_servers SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![enabled as i32, now, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_discovery_status(
    conn: &Connection,
    id: &str,
    status: &str,
    error: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE mcp_servers SET discovery_status = ?1, discovery_error = ?2,
         last_discovered_at = ?3, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![status, error, now, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn replace_tools(conn: &Connection, server_id: &str, tools: &[McpTool]) -> Result<(), String> {
    conn.execute("DELETE FROM mcp_tools WHERE server_id = ?1", [server_id])
        .map_err(|e| e.to_string())?;

    for tool in tools {
        conn.execute(
            "INSERT INTO mcp_tools (id, server_id, name, tool_key, description, parameters)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                tool.id,
                tool.server_id,
                tool.name,
                tool.tool_key,
                tool.description,
                tool.parameters,
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn replace_resources(
    conn: &Connection,
    server_id: &str,
    resources: &[McpResource],
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM mcp_resources WHERE server_id = ?1",
        [server_id],
    )
    .map_err(|e| e.to_string())?;

    for res in resources {
        conn.execute(
            "INSERT INTO mcp_resources (id, server_id, uri, name, description)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![res.id, res.server_id, res.uri, res.name, res.description],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn get_enabled_tools(conn: &Connection) -> Result<Vec<(McpTool, String)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.server_id, t.name, t.tool_key, t.description, t.parameters, s.url
             FROM mcp_tools t
             JOIN mcp_servers s ON t.server_id = s.id
             WHERE s.enabled = 1 AND s.discovery_status = 'connected'",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                McpTool {
                    id: row.get(0)?,
                    server_id: row.get(1)?,
                    name: row.get(2)?,
                    tool_key: row.get(3)?,
                    description: row.get(4)?,
                    parameters: row.get(5)?,
                },
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

pub fn get_tool_by_key(conn: &Connection, tool_key: &str) -> Result<Option<McpTool>, String> {
    conn.query_row(
        "SELECT id, server_id, name, tool_key, description, parameters
         FROM mcp_tools WHERE tool_key = ?1",
        [tool_key],
        |row| {
            Ok(McpTool {
                id: row.get(0)?,
                server_id: row.get(1)?,
                name: row.get(2)?,
                tool_key: row.get(3)?,
                description: row.get(4)?,
                parameters: row.get(5)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub fn get_server_auth_value(conn: &Connection, server_id: &str) -> Result<Option<String>, String> {
    let encrypted: Option<String> = conn
        .query_row(
            "SELECT auth_value FROM mcp_servers WHERE id = ?1",
            [server_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .flatten();

    match encrypted {
        Some(enc) => Ok(Some(decrypt_token(&enc)?)),
        None => Ok(None),
    }
}

pub fn get_tool_count(conn: &Connection, server_id: &str) -> Result<usize, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM mcp_tools WHERE server_id = ?1",
        [server_id],
        |r| r.get::<_, i64>(0),
    )
    .map(|n| n as usize)
    .map_err(|e| e.to_string())
}

pub fn get_resource_count(conn: &Connection, server_id: &str) -> Result<usize, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM mcp_resources WHERE server_id = ?1",
        [server_id],
        |r| r.get::<_, i64>(0),
    )
    .map(|n| n as usize)
    .map_err(|e| e.to_string())
}

/// Decrypt and parse env_config from database, returning a HashMap.
/// Returns None if env_config is NULL, or an error if decryption/parsing fails.
pub fn get_server_env_config(
    conn: &Connection,
    server_id: &str,
) -> Result<Option<std::collections::HashMap<String, String>>, String> {
    let encrypted: Option<String> = conn
        .query_row(
            "SELECT env_config FROM mcp_servers WHERE id = ?1",
            [server_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .flatten();

    match encrypted {
        Some(enc) => {
            let decrypted = decrypt_token(&enc)?;
            let parsed: std::collections::HashMap<String, String> = serde_json::from_str(&decrypted)
                .map_err(|e| format!("Failed to parse env_config JSON: {e}"))?;
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn make_req(name: &str) -> CreateMcpServerRequest {
        CreateMcpServerRequest {
            name: name.to_string(),
            url: "http://localhost:8080/mcp".to_string(),
            transport_type: "http".to_string(),
            transport_config: "{}".to_string(),
            auth_type: "none".to_string(),
            auth_value: None,
            enabled: true,
            env_config: None,
        }
    }

    #[test]
    fn test_mcp_server_crud() {
        let conn = setup();

        // Create
        let server = create_server(&conn, &make_req("Test Server")).unwrap();
        assert_eq!(server.name, "Test Server");
        assert_eq!(server.transport_type, "http");
        assert_eq!(server.discovery_status, "pending");
        assert!(server.enabled);

        // Read
        let fetched = get_server(&conn, &server.id).unwrap().unwrap();
        assert_eq!(fetched.id, server.id);

        // List
        let all = list_servers(&conn).unwrap();
        assert_eq!(all.len(), 1);

        // Update
        let updated = update_server(
            &conn,
            &server.id,
            &UpdateMcpServerRequest {
                name: Some("Renamed".to_string()),
                url: None,
                transport_type: None,
                transport_config: None,
                auth_type: None,
                auth_value: None,
                enabled: None,
                env_config: None,
            },
        )
        .unwrap();
        assert_eq!(updated.name, "Renamed");

        // Delete
        delete_server(&conn, &server.id).unwrap();
        assert!(get_server(&conn, &server.id).unwrap().is_none());
        assert!(list_servers(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_auth_value_encrypted_at_rest() {
        let conn = setup();

        let req = CreateMcpServerRequest {
            name: "Secured".to_string(),
            url: "http://localhost/mcp".to_string(),
            transport_type: "http".to_string(),
            transport_config: "{}".to_string(),
            auth_type: "bearer".to_string(),
            auth_value: Some("super-secret-token".to_string()),
            enabled: true,
            env_config: None,
        };
        let server = create_server(&conn, &req).unwrap();

        // Raw DB value must NOT equal the plaintext token
        let raw: Option<String> = conn
            .query_row(
                "SELECT auth_value FROM mcp_servers WHERE id = ?1",
                [&server.id],
                |r| r.get(0),
            )
            .unwrap();
        let raw = raw.unwrap();
        assert_ne!(
            raw, "super-secret-token",
            "auth_value must be encrypted in DB"
        );

        // Decrypted value must match original
        let decrypted = get_server_auth_value(&conn, &server.id).unwrap().unwrap();
        assert_eq!(decrypted, "super-secret-token");
    }

    #[test]
    fn test_disabled_server_excluded_from_tools() {
        let conn = setup();

        let mut req = make_req("Disabled Server");
        req.enabled = false;
        let server = create_server(&conn, &req).unwrap();

        // Mark connected and add a tool
        update_discovery_status(&conn, &server.id, "connected", None).unwrap();
        let tool = McpTool {
            id: Uuid::now_v7().to_string(),
            server_id: server.id.clone(),
            name: "echo".to_string(),
            tool_key: "mcp_disabled_server_echo".to_string(),
            description: None,
            parameters: "{}".to_string(),
        };
        replace_tools(&conn, &server.id, &[tool]).unwrap();

        let tools = get_enabled_tools(&conn).unwrap();
        assert!(tools.is_empty(), "disabled server tools should be excluded");
    }

    #[test]
    fn test_update_discovery_status() {
        let conn = setup();

        let server = create_server(&conn, &make_req("Status Test")).unwrap();
        assert_eq!(server.discovery_status, "pending");

        update_discovery_status(&conn, &server.id, "connected", None).unwrap();
        let updated = get_server(&conn, &server.id).unwrap().unwrap();
        assert_eq!(updated.discovery_status, "connected");
        assert!(updated.last_discovered_at.is_some());

        update_discovery_status(&conn, &server.id, "error", Some("connection refused")).unwrap();
        let errored = get_server(&conn, &server.id).unwrap().unwrap();
        assert_eq!(errored.discovery_status, "error");
        assert_eq!(
            errored.discovery_error.as_deref(),
            Some("connection refused")
        );
    }

    #[test]
    fn test_cascade_delete_clears_tools() {
        let conn = setup();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        let server = create_server(&conn, &make_req("Cascade Test")).unwrap();
        let tool = McpTool {
            id: Uuid::now_v7().to_string(),
            server_id: server.id.clone(),
            name: "ping".to_string(),
            tool_key: "mcp_cascade_test_ping".to_string(),
            description: None,
            parameters: "{}".to_string(),
        };
        replace_tools(&conn, &server.id, &[tool]).unwrap();

        let count = get_tool_count(&conn, &server.id).unwrap();
        assert_eq!(count, 1);

        delete_server(&conn, &server.id).unwrap();

        // After delete, count should be 0 (cascade)
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM mcp_tools", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "cascade delete should clear mcp_tools");
    }

    #[test]
    fn test_env_config_encrypted_at_rest() {
        let conn = setup();

        let req = CreateMcpServerRequest {
            name: "Env Test".to_string(),
            url: "".to_string(),
            transport_type: "stdio".to_string(),
            transport_config: r#"{"command":"/usr/bin/test","args":[]}"#.to_string(),
            auth_type: "none".to_string(),
            auth_value: None,
            enabled: true,
            env_config: Some(r#"{"API_KEY":"secret123","DEBUG":"1"}"#.to_string()),
        };
        let server = create_server(&conn, &req).unwrap();

        // Raw DB value must be encrypted (not equal to plaintext)
        let raw: Option<String> = conn
            .query_row(
                "SELECT env_config FROM mcp_servers WHERE id = ?1",
                [&server.id],
                |r| r.get(0),
            )
            .unwrap();
        let raw = raw.unwrap();
        assert_ne!(
            raw,
            r#"{"API_KEY":"secret123","DEBUG":"1"}"#,
            "env_config should be encrypted at rest"
        );

        // Decrypted value must match original
        let env_map = get_server_env_config(&conn, &server.id).unwrap().unwrap();
        assert_eq!(env_map.get("API_KEY").unwrap(), "secret123");
        assert_eq!(env_map.get("DEBUG").unwrap(), "1");
    }

    #[test]
    fn test_update_env_config() {
        let conn = setup();

        let server = create_server(&conn, &make_req("Env Update")).unwrap();
        assert!(server.env_config.is_none());

        let updated = update_server(
            &conn,
            &server.id,
            &UpdateMcpServerRequest {
                name: None,
                url: None,
                transport_type: None,
                transport_config: None,
                auth_type: None,
                auth_value: None,
                enabled: None,
                env_config: Some(r#"{"NEW_VAR":"value"}"#.to_string()),
            },
        )
        .unwrap();

        assert!(updated.env_config.is_some());
        let env_map = get_server_env_config(&conn, &server.id).unwrap().unwrap();
        assert_eq!(env_map.get("NEW_VAR").unwrap(), "value");
    }

    #[test]
    fn test_clear_env_config_with_empty_string() {
        let conn = setup();

        let mut req = make_req("Clear Env");
        req.env_config = Some(r#"{"KEY":"val"}"#.to_string());
        let server = create_server(&conn, &req).unwrap();
        assert!(server.env_config.is_some());

        let updated = update_server(
            &conn,
            &server.id,
            &UpdateMcpServerRequest {
                name: None,
                url: None,
                transport_type: None,
                transport_config: None,
                auth_type: None,
                auth_value: None,
                enabled: None,
                env_config: Some("".to_string()), // Clear
            },
        )
        .unwrap();

        assert!(updated.env_config.is_none());
    }

    #[test]
    fn test_env_config_none_preserves_existing() {
        let conn = setup();

        let mut req = make_req("Preserve Env");
        req.env_config = Some(r#"{"ORIGINAL":"value"}"#.to_string());
        let server = create_server(&conn, &req).unwrap();

        // Update without touching env_config
        let updated = update_server(
            &conn,
            &server.id,
            &UpdateMcpServerRequest {
                name: Some("New Name".to_string()),
                url: None,
                transport_type: None,
                transport_config: None,
                auth_type: None,
                auth_value: None,
                enabled: None,
                env_config: None, // Don't update
            },
        )
        .unwrap();

        // env_config should still be there
        assert!(updated.env_config.is_some());
        let env_map = get_server_env_config(&conn, &server.id).unwrap().unwrap();
        assert_eq!(env_map.get("ORIGINAL").unwrap(), "value");
    }
}
