// Shell Command Execution Tauri Commands
//
// This module provides Tauri commands for the frontend to:
// - Manage kubeconfig files (upload, list, activate, delete)
// - Respond to shell command approval requests
// - List command execution history
// - Check kubectl installation status

use crate::shell::KubeconfigInfo;
use crate::state::{AppState, ApprovalResponse};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    pub id: String,
    pub command: String,
    pub tier: i32,
    pub approval_status: String,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub executed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubectlStatus {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[tauri::command]
pub async fn upload_kubeconfig(
    name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Generate ID
    let id = uuid::Uuid::now_v7().to_string();

    // Parse kubeconfig to extract context
    let contexts = crate::shell::kubeconfig::parse_kubeconfig_contexts(&content)?;
    let context = contexts
        .first()
        .ok_or_else(|| "No contexts found in kubeconfig".to_string())?;

    // Encrypt content
    let encrypted_content = crate::integrations::auth::encrypt_token(&content)?;

    // Store in database
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "INSERT INTO kubeconfig_files (id, name, encrypted_content, context, cluster_url, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            params![&id, &name, &encrypted_content, &context.name, &context.cluster_url],
        ).map_err(|e| format!("Failed to store kubeconfig: {e}"))?;
    }

    Ok(id)
}

#[tauri::command]
pub fn list_kubeconfigs(state: State<'_, AppState>) -> Result<Vec<KubeconfigInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare("SELECT id, name, context, cluster_url, is_active FROM kubeconfig_files ORDER BY uploaded_at DESC")
        .map_err(|e| format!("Failed to prepare statement: {e}"))?;

    let configs = stmt
        .query_map([], |row| {
            Ok(KubeconfigInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                context: row.get(2)?,
                cluster_url: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
            })
        })
        .map_err(|e| format!("Failed to query kubeconfigs: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {e}"))?;

    Ok(configs)
}

#[tauri::command]
pub fn activate_kubeconfig(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Deactivate all configs
    db.execute("UPDATE kubeconfig_files SET is_active = 0", [])
        .map_err(|e| format!("Failed to deactivate configs: {e}"))?;

    // Activate the specified config
    db.execute(
        "UPDATE kubeconfig_files SET is_active = 1 WHERE id = ?1",
        params![&id],
    )
    .map_err(|e| format!("Failed to activate config: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn delete_kubeconfig(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.execute("DELETE FROM kubeconfig_files WHERE id = ?1", params![&id])
        .map_err(|e| format!("Failed to delete kubeconfig: {e}"))?;

    Ok(())
}

#[tauri::command]
pub async fn respond_to_shell_approval(
    approval_id: String,
    decision: String, // "deny", "allow_once", "allow_session"
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Retrieve the pending approval channel
    let sender = {
        let mut approvals = state.pending_approvals.lock().await;
        approvals.remove(&approval_id)
    };

    if let Some(sender) = sender {
        let approved = decision != "deny";
        let response = ApprovalResponse { approved, decision };

        // Send response
        sender
            .send(response)
            .map_err(|_| "Failed to send approval response".to_string())?;

        Ok(())
    } else {
        Err("Approval request not found or already responded to".to_string())
    }
}

#[tauri::command]
pub fn list_command_executions(
    issue_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<CommandExecution>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let (query, params_vec): (String, Vec<String>) = if let Some(issue_id) = issue_id {
        (
            "SELECT id, command, tier, approval_status, exit_code, stdout, stderr, execution_time_ms, executed_at
             FROM command_executions
             WHERE issue_id = ?1
             ORDER BY executed_at DESC
             LIMIT 100".to_string(),
            vec![issue_id],
        )
    } else {
        (
            "SELECT id, command, tier, approval_status, exit_code, stdout, stderr, execution_time_ms, executed_at
             FROM command_executions
             ORDER BY executed_at DESC
             LIMIT 100".to_string(),
            vec![],
        )
    };

    let mut stmt = db
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare statement: {e}"))?;

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec
        .iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();

    let executions = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(CommandExecution {
                id: row.get(0)?,
                command: row.get(1)?,
                tier: row.get(2)?,
                approval_status: row.get(3)?,
                exit_code: row.get(4)?,
                stdout: row.get(5)?,
                stderr: row.get(6)?,
                execution_time_ms: row.get(7)?,
                executed_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("Failed to query executions: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {e}"))?;

    Ok(executions)
}

#[tauri::command]
pub async fn check_kubectl_installed(_state: State<'_, AppState>) -> Result<KubectlStatus, String> {
    match crate::shell::kubectl::locate_kubectl() {
        Ok(path) => {
            // Try to get version
            let version = tokio::process::Command::new(&path)
                .arg("version")
                .arg("--client")
                .arg("--output=json")
                .output()
                .await
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                });

            Ok(KubectlStatus {
                installed: true,
                path: Some(path.to_string_lossy().to_string()),
                version,
            })
        }
        Err(_) => Ok(KubectlStatus {
            installed: false,
            path: None,
            version: None,
        }),
    }
}

/// Return the live classifier rule lists so the UI can render them dynamically.
/// The data derives directly from the module-level const arrays in classifier.rs,
/// so any addition or removal there is automatically reflected in the UI.
#[tauri::command]
pub fn get_classifier_rules() -> crate::shell::classifier::ClassifierRules {
    crate::shell::classifier::CommandClassifier::get_rules()
}
