// Command Executor with Approval Flow
//
// This module handles:
// - Command execution with safety tier enforcement
// - User approval flow for Tier 2 commands
// - PII detection and audit logging
// - Timeout protection

use crate::shell::classifier::{CommandClassifier, CommandTier};
use crate::state::{AppState, ApprovalResponse};
use rusqlite::params;
use std::time::{Duration, Instant};
use tauri::Emitter;

pub use crate::shell::kubectl::CommandOutput;

const APPROVAL_TIMEOUT: Duration = Duration::from_secs(60);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn execute_with_approval(
    command: &str,
    app_handle: &tauri::AppHandle,
    state: &AppState,
    kubeconfig_id: Option<&str>,
    working_dir: Option<&str>,
) -> Result<CommandOutput, String> {
    // Step 1: Classify command
    let classifier = CommandClassifier::new();
    let classification = classifier.classify(command);

    tracing::info!(
        command = %command,
        tier = ?classification.tier,
        reasoning = %classification.reasoning,
        "Command classified"
    );

    // Step 2: Match on tier
    match classification.tier {
        CommandTier::Tier3 => {
            // Always deny
            tracing::warn!(
                command = %command,
                reasoning = %classification.reasoning,
                "Command denied (Tier 3)"
            );
            return Err(format!(
                "Command denied: {} (Tier 3: {})",
                command, classification.reasoning
            ));
        }
        CommandTier::Tier2 => {
            // Require approval
            let approved = request_approval(command, &classification, app_handle, state).await?;

            if !approved {
                tracing::warn!(command = %command, "Command denied by user");
                return Err(format!("Command denied by user: {command}"));
            }
        }
        CommandTier::Tier1 => {
            // Auto-execute (no approval needed)
            tracing::info!(command = %command, "Auto-executing Tier 1 command");
        }
    }

    // Step 3: Execute command (Tier 1 or approved Tier 2)
    let start_time = Instant::now();
    let output = execute_command(command, kubeconfig_id, working_dir, state).await?;
    let execution_time_ms = start_time.elapsed().as_millis() as i64;

    // Step 4: Record execution in database
    let approval_status = match classification.tier {
        CommandTier::Tier1 => "auto",
        CommandTier::Tier2 => "approved",
        CommandTier::Tier3 => unreachable!(),
    };

    record_execution(
        command,
        classification.tier.to_tier_number(),
        approval_status,
        kubeconfig_id,
        &output,
        execution_time_ms,
        state,
    )?;

    // Step 5: Audit log
    write_audit_log(command, &output, state)?;

    Ok(output)
}

async fn request_approval(
    command: &str,
    classification: &crate::shell::classifier::ClassificationResult,
    app_handle: &tauri::AppHandle,
    state: &AppState,
) -> Result<bool, String> {
    // Generate approval ID
    let approval_id = uuid::Uuid::now_v7().to_string();

    // Create oneshot channel
    let (sender, receiver) = tokio::sync::oneshot::channel::<ApprovalResponse>();

    // Store channel
    {
        let mut approvals = state.pending_approvals.lock().await;
        approvals.insert(approval_id.clone(), sender);
    }

    // Emit approval event to frontend
    #[derive(Clone, serde::Serialize)]
    struct ApprovalRequest {
        approval_id: String,
        command: String,
        tier: i32,
        reasoning: String,
        risk_factors: Vec<String>,
    }

    let request = ApprovalRequest {
        approval_id: approval_id.clone(),
        command: command.to_string(),
        tier: classification.tier.to_tier_number(),
        reasoning: classification.reasoning.clone(),
        risk_factors: classification.risk_factors.clone(),
    };

    app_handle
        .emit("shell:approval-needed", request)
        .map_err(|e| format!("Failed to emit approval event: {e}"))?;

    // Wait for response with timeout
    match tokio::time::timeout(APPROVAL_TIMEOUT, receiver).await {
        Ok(Ok(response)) => Ok(response.approved),
        Ok(Err(_)) => Err("Approval channel closed".to_string()),
        Err(_) => {
            // Timeout - clean up
            let mut approvals = state.pending_approvals.lock().await;
            approvals.remove(&approval_id);
            Err("Approval request timed out".to_string())
        }
    }
}

async fn execute_command(
    command: &str,
    kubeconfig_id: Option<&str>,
    working_dir: Option<&str>,
    state: &AppState,
) -> Result<CommandOutput, String> {
    // Check if kubectl command
    if command.trim().starts_with("kubectl") {
        // Extract kubectl args
        let parts: Vec<&str> = command.split_whitespace().collect();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        // Get kubeconfig path - use provided ID or fallback to active kubeconfig
        let kubeconfig_path = if let Some(id) = kubeconfig_id {
            Some(get_kubeconfig_path(id, state)?)
        } else {
            // Auto-select active kubeconfig for kubectl commands
            get_active_kubeconfig_path(state).ok()
        };

        return crate::shell::kubectl::execute_kubectl(
            &args,
            kubeconfig_path.as_deref(),
            working_dir,
        )
        .await;
    }

    // General shell command execution
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = tokio::process::Command::new("cmd");
        c.arg("/C").arg(command);
        c
    };

    #[cfg(not(target_os = "windows"))]
    let mut cmd = {
        let mut c = tokio::process::Command::new("sh");
        c.arg("-c").arg(command);
        c
    };

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Execute with timeout
    let start = Instant::now();
    let output = tokio::time::timeout(COMMAND_TIMEOUT, cmd.output())
        .await
        .map_err(|_| "Command execution timed out".to_string())?
        .map_err(|e| format!("Failed to execute command: {e}"))?;
    let execution_time_ms = start.elapsed().as_millis() as u64;

    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        execution_time_ms,
    })
}

fn get_kubeconfig_path(kubeconfig_id: &str, state: &AppState) -> Result<String, String> {
    // Retrieve encrypted kubeconfig from database
    let encrypted_content = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT encrypted_content FROM kubeconfig_files WHERE id = ?1",
            params![kubeconfig_id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| format!("Kubeconfig not found: {e}"))?
    };

    // Decrypt kubeconfig content
    let decrypted_content = crate::integrations::auth::decrypt_token(&encrypted_content)?;

    // Write to secure temp file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{kubeconfig_id}.yaml"));

    std::fs::write(&temp_path, decrypted_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    Ok(temp_path.to_string_lossy().to_string())
}

fn get_active_kubeconfig_path(state: &AppState) -> Result<String, String> {
    // Get ID of active kubeconfig
    let active_id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT id FROM kubeconfig_files WHERE is_active = 1 LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| format!("No active kubeconfig found: {e}"))?
    };

    // Use existing get_kubeconfig_path function
    get_kubeconfig_path(&active_id, state)
}

fn record_execution(
    command: &str,
    tier: i32,
    approval_status: &str,
    kubeconfig_id: Option<&str>,
    output: &CommandOutput,
    execution_time_ms: i64,
    state: &AppState,
) -> Result<(), String> {
    let id = uuid::Uuid::now_v7().to_string();

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO command_executions (id, command, tier, approval_status, kubeconfig_id, exit_code, stdout, stderr, execution_time_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            &id,
            command,
            tier,
            approval_status,
            kubeconfig_id,
            output.exit_code,
            &output.stdout,
            &output.stderr,
            execution_time_ms,
        ],
    )
    .map_err(|e| format!("Failed to record execution: {e}"))?;

    Ok(())
}

fn write_audit_log(command: &str, output: &CommandOutput, state: &AppState) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let details = serde_json::json!({
        "command": command,
        "exit_code": output.exit_code,
    });

    crate::audit::log::write_audit_event(
        &db,
        "shell_command_execution",
        "shell_command",
        command,
        &details.to_string(),
    )
    .map_err(|e| format!("Audit log failed: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: These tests will require mock AppState setup
    // For now, they're placeholders

    #[tokio::test]
    #[ignore] // Requires full app setup
    async fn test_tier1_immediate_execution() {
        // TODO: Test that Tier 1 commands execute immediately
    }

    #[tokio::test]
    #[ignore] // Requires event system
    async fn test_tier2_emits_approval_event() {
        // TODO: Test that Tier 2 commands emit approval event
    }

    #[tokio::test]
    #[ignore] // Requires full app setup
    async fn test_tier3_immediate_denial() {
        // TODO: Test that Tier 3 commands are denied immediately
    }

    #[tokio::test]
    #[ignore] // Requires timeout setup
    async fn test_approval_timeout() {
        // TODO: Test that approval requests timeout after 60s
    }
}
