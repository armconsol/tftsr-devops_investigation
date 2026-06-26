use rmcp::model::{CallToolRequestParams, Content, RawContent};
use rmcp::{service::RunningService, RoleClient, ServiceExt};
use serde_json::Map;

use crate::mcp::models::{McpResource, McpTool};

/// Live connection to an MCP server.
pub type McpConnection = RunningService<RoleClient, ()>;

/// Connect to a stdio MCP server with optional environment variables.
pub async fn connect_stdio(
    command: &str,
    args: &[String],
    env: std::collections::HashMap<String, String>,
) -> Result<McpConnection, String> {
    let transport = crate::mcp::transport::stdio::build_stdio_transport(command, args, env)?;
    ().serve(transport)
        .await
        .map_err(|e| format!("MCP stdio connection failed: {e}"))
}

/// Connect to an HTTP MCP server with optional custom headers.
pub async fn connect_http(
    url: &str,
    auth_header: Option<&str>,
    custom_headers: std::collections::HashMap<String, String>,
) -> Result<McpConnection, String> {
    let transport =
        crate::mcp::transport::http::build_http_transport(url, auth_header, custom_headers);
    ().serve(transport)
        .await
        .map_err(|e| format!("MCP HTTP connection failed: {e}"))
}

/// List all tools from an active connection, mapped to our McpTool type.
pub async fn list_tools(
    conn: &McpConnection,
    server_id: &str,
    server_name: &str,
) -> Result<Vec<McpTool>, String> {
    let rmcp_tools = conn
        .list_all_tools()
        .await
        .map_err(|e| format!("list_all_tools failed: {e}"))?;

    let tools = rmcp_tools
        .into_iter()
        .map(|t| {
            let tool_key = crate::mcp::adapter::build_tool_key(server_name, &t.name);
            let parameters =
                serde_json::to_string(&*t.input_schema).unwrap_or_else(|_| "{}".to_string());
            McpTool {
                id: uuid::Uuid::now_v7().to_string(),
                server_id: server_id.to_string(),
                name: t.name.to_string(),
                tool_key,
                description: t.description.map(|d| d.to_string()),
                parameters,
            }
        })
        .collect();

    Ok(tools)
}

/// List all resources from an active connection.
pub async fn list_resources(
    conn: &McpConnection,
    server_id: &str,
) -> Result<Vec<McpResource>, String> {
    let rmcp_resources = conn
        .list_all_resources()
        .await
        .map_err(|e| format!("list_all_resources failed: {e}"))?;

    let resources = rmcp_resources
        .into_iter()
        .map(|r| McpResource {
            id: uuid::Uuid::now_v7().to_string(),
            server_id: server_id.to_string(),
            uri: r.raw.uri.clone(),
            name: Some(r.raw.name.clone()),
            description: r.raw.description.clone(),
        })
        .collect();

    Ok(resources)
}

/// Call an MCP tool and return its text result.
/// Enforces a 30-second hard timeout to prevent a misbehaving server from
/// stalling the AI chat loop indefinitely.
pub async fn call_tool(
    conn: &McpConnection,
    tool_name: &str,
    arguments: &serde_json::Value,
) -> Result<String, String> {
    let args: Option<Map<String, serde_json::Value>> = arguments.as_object().cloned();

    let params = match args {
        Some(map) => CallToolRequestParams::new(tool_name.to_string()).with_arguments(map),
        None => CallToolRequestParams::new(tool_name.to_string()),
    };

    let result = tokio::time::timeout(std::time::Duration::from_secs(30), conn.call_tool(params))
        .await
        .map_err(|_| format!("MCP tool '{tool_name}' timed out after 30s"))?
        .map_err(|e| format!("MCP tool call failed: {e}"))?;

    if result.is_error == Some(true) {
        let msg = extract_text_content(&result.content);
        return Err(format!("MCP tool returned error: {msg}"));
    }

    Ok(extract_text_content(&result.content))
}

fn extract_text_content(content: &[Content]) -> String {
    content
        .iter()
        .filter_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}
