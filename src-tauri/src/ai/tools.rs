use crate::ai::{ParameterProperty, Tool, ToolParameters};
use std::collections::HashMap;

/// Get all statically-registered tools for AI function calling.
pub fn get_available_tools() -> Vec<Tool> {
    vec![get_add_ado_comment_tool(), get_execute_shell_command_tool()]
}

/// Fetch tools from all connected, enabled MCP servers.
pub async fn get_enabled_mcp_tools(state: &crate::state::AppState) -> Vec<Tool> {
    crate::mcp::adapter::get_enabled_mcp_tools(state)
        .await
        .unwrap_or_default()
}

/// Tool definition for adding comments to Azure DevOps work items
fn get_add_ado_comment_tool() -> Tool {
    let mut properties = HashMap::new();

    properties.insert(
        "work_item_id".to_string(),
        ParameterProperty {
            prop_type: "integer".to_string(),
            description: "The Azure DevOps work item ID (ticket number) to add the comment to"
                .to_string(),
            enum_values: None,
        },
    );

    properties.insert(
        "comment_text".to_string(),
        ParameterProperty {
            prop_type: "string".to_string(),
            description: "The text content of the comment to add to the work item".to_string(),
            enum_values: None,
        },
    );

    Tool {
        name: "add_ado_comment".to_string(),
        description: "Add a comment to an Azure DevOps work item (ticket). Use this when the user asks you to add a comment, update a ticket, or provide information to a ticket.".to_string(),
        parameters: ToolParameters {
            param_type: "object".to_string(),
            properties,
            required: vec!["work_item_id".to_string(), "comment_text".to_string()],
        },
    }
}

/// Tool definition for executing shell commands with safety classification
fn get_execute_shell_command_tool() -> Tool {
    let mut properties = HashMap::new();

    properties.insert(
        "command".to_string(),
        ParameterProperty {
            prop_type: "string".to_string(),
            description: "Shell command to execute. Supports kubectl, pvesh, qm, and general shell commands. Read-only commands execute automatically. Mutating commands require user approval.".to_string(),
            enum_values: None,
        },
    );

    properties.insert(
        "working_directory".to_string(),
        ParameterProperty {
            prop_type: "string".to_string(),
            description: "Optional working directory for command execution".to_string(),
            enum_values: None,
        },
    );

    properties.insert(
        "kubeconfig_id".to_string(),
        ParameterProperty {
            prop_type: "string".to_string(),
            description: "Optional kubeconfig file ID for kubectl commands".to_string(),
            enum_values: None,
        },
    );

    Tool {
        name: "execute_shell_command".to_string(),
        description: "Execute shell commands with automatic safety classification. Tier 1 (read-only): kubectl get/describe/logs, cat, grep, ls - execute automatically. Tier 2 (mutating): kubectl apply/delete/scale, chmod, systemctl restart - require user approval. Tier 3 (destructive): rm -rf, shutdown, mkfs - always denied.".to_string(),
        parameters: ToolParameters {
            param_type: "object".to_string(),
            properties,
            required: vec!["command".to_string()],
        },
    }
}
