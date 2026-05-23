use std::collections::HashMap;

use crate::ai::{ParameterProperty, Tool, ToolParameters};
use crate::mcp::models::McpTool;

/// Sanitize a string for use as part of a tool key:
/// lowercase → non-alphanumeric to `_` → collapse consecutive `_` → trim `_`.
pub fn sanitize_name(s: &str) -> String {
    let lower = s.to_lowercase();
    let replaced: String = lower
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    // Collapse consecutive underscores
    let mut collapsed = String::with_capacity(replaced.len());
    let mut prev_underscore = false;
    for c in replaced.chars() {
        if c == '_' {
            if !prev_underscore {
                collapsed.push(c);
            }
            prev_underscore = true;
        } else {
            collapsed.push(c);
            prev_underscore = false;
        }
    }

    // Trim leading/trailing underscores
    collapsed.trim_matches('_').to_string()
}

/// Build a unique, AI-safe tool key: `mcp_{server_name}_{tool_name}`.
pub fn build_tool_key(server_name: &str, tool_name: &str) -> String {
    format!("mcp_{}_{}", sanitize_name(server_name), sanitize_name(tool_name))
}

/// Convert stored McpTool records into AI Tool definitions.
pub fn mcp_tools_to_ai_tools(tools: &[McpTool]) -> Vec<Tool> {
    tools
        .iter()
        .map(|t| {
            let parameters = parse_parameters(&t.parameters);
            Tool {
                name: t.tool_key.clone(),
                description: t
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("MCP tool: {}", t.name)),
                parameters,
            }
        })
        .collect()
}

/// Parse a JSON schema string into AI ToolParameters.
/// Falls back to an empty object schema on any parse error.
fn parse_parameters(schema_json: &str) -> ToolParameters {
    let value: serde_json::Value = serde_json::from_str(schema_json).unwrap_or_default();

    let properties = value
        .get("properties")
        .and_then(|p| p.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| {
                    let prop_type = v
                        .get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("string")
                        .to_string();
                    let description = v
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();
                    (
                        k.clone(),
                        ParameterProperty {
                            prop_type,
                            description,
                            enum_values: None,
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();

    let required = value
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    ToolParameters {
        param_type: "object".to_string(),
        properties,
        required,
    }
}

/// Async wrapper — fetch enabled MCP tools from state and convert to AI tools.
pub async fn get_enabled_mcp_tools(
    state: &crate::state::AppState,
) -> Result<Vec<Tool>, String> {
    let tool_records = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        crate::mcp::store::get_enabled_tools(&db)?
    };

    let tools = tool_records
        .iter()
        .map(|(t, _url)| {
            let parameters = parse_parameters(&t.parameters);
            Tool {
                name: t.tool_key.clone(),
                description: t
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("MCP tool: {}", t.name)),
                parameters,
            }
        })
        .collect();

    Ok(tools)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::models::McpTool;

    #[test]
    fn test_tool_name_sanitization() {
        assert_eq!(sanitize_name("My Weather API"), "my_weather_api");
        assert_eq!(sanitize_name("get_forecast"), "get_forecast");
        assert_eq!(sanitize_name("foo--bar"), "foo_bar");
        assert_eq!(sanitize_name("  leading trailing  "), "leading_trailing");
        assert_eq!(sanitize_name("CamelCase"), "camelcase");
        assert_eq!(sanitize_name("v1.0.0"), "v1_0_0");
        assert_eq!(sanitize_name("___underscores___"), "underscores");
        assert_eq!(sanitize_name("hello world"), "hello_world");
    }

    #[test]
    fn test_build_tool_key() {
        assert_eq!(
            build_tool_key("My Weather API", "get_forecast"),
            "mcp_my_weather_api_get_forecast"
        );
        assert_eq!(
            build_tool_key("simple", "ping"),
            "mcp_simple_ping"
        );
        assert_eq!(
            build_tool_key("My Server", "search files"),
            "mcp_my_server_search_files"
        );
    }

    #[test]
    fn test_mcp_tool_to_ai_tool_conversion() {
        let tool = McpTool {
            id: "1".to_string(),
            server_id: "srv".to_string(),
            name: "echo".to_string(),
            tool_key: "mcp_test_echo".to_string(),
            description: Some("Echoes text back".to_string()),
            parameters: r#"{
                "type": "object",
                "properties": {
                    "message": { "type": "string", "description": "The text to echo" }
                },
                "required": ["message"]
            }"#
            .to_string(),
        };

        let ai_tools = mcp_tools_to_ai_tools(&[tool]);
        assert_eq!(ai_tools.len(), 1);

        let ai_tool = &ai_tools[0];
        assert_eq!(ai_tool.name, "mcp_test_echo");
        assert_eq!(ai_tool.description, "Echoes text back");
        assert_eq!(ai_tool.parameters.param_type, "object");
        assert!(ai_tool.parameters.properties.contains_key("message"));
        assert_eq!(ai_tool.parameters.required, vec!["message".to_string()]);

        let msg_prop = &ai_tool.parameters.properties["message"];
        assert_eq!(msg_prop.prop_type, "string");
        assert_eq!(msg_prop.description, "The text to echo");
    }

    #[test]
    fn test_mcp_tool_missing_description_uses_fallback() {
        let tool = McpTool {
            id: "2".to_string(),
            server_id: "srv".to_string(),
            name: "ping".to_string(),
            tool_key: "mcp_test_ping".to_string(),
            description: None,
            parameters: "{}".to_string(),
        };

        let ai_tools = mcp_tools_to_ai_tools(&[tool]);
        assert_eq!(ai_tools[0].description, "MCP tool: ping");
    }

    #[test]
    fn test_parse_parameters_malformed_json() {
        let params = parse_parameters("{invalid json");
        assert_eq!(params.param_type, "object");
        assert!(params.properties.is_empty());
        assert!(params.required.is_empty());
    }
}
