use crate::ai::{ParameterProperty, Tool, ToolParameters};
use std::collections::HashMap;

/// Get all available tools for AI function calling
pub fn get_available_tools() -> Vec<Tool> {
    vec![get_add_ado_comment_tool()]
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
