# Shell Execution

**Status**: ✅ Production-ready agentic shell execution with three-tier safety classification (v1.0.0)

## Overview

The Shell Execution feature enables AI-powered autonomous execution of diagnostic commands with intelligent safety controls. The AI can directly execute kubectl, Proxmox tools, and general shell commands to gather troubleshooting data without manual intervention.

**Key Features**:
- Three-tier command safety classification (auto/approve/deny)
- Real-time approval modal for mutating operations
- kubectl integration with bundled binary (v1.30.0)
- Multi-cluster support via multiple kubeconfig files
- AES-256-GCM encrypted kubeconfig storage
- Complete audit trail for all executions
- Pipe/chain command analysis with tier escalation
- Command timeout protection (30s)
- Approval timeout protection (60s)

## Three-Tier Safety Architecture

Commands are automatically classified into three safety tiers based on their potential impact:

### Tier 1: Auto-Execute (Read-Only)
**Behavior**: Execute immediately without user approval

**kubectl commands**:
- `kubectl get [resource]` - List resources
- `kubectl describe [resource]` - Show detailed resource information
- `kubectl logs [pod]` - View pod logs

**General commands**:
- `cat [file]` - Display file contents
- `grep [pattern]` - Search text patterns
- `ls` - List directory contents
- `pwd` - Print working directory
- `whoami` - Display current user
- `date` - Show system date/time
- `uptime` - Show system uptime
- `df -h` - Show disk usage
- `free -m` - Show memory usage
- `ps aux` - List processes

**Proxmox commands**:
- `pvecm status` - Show cluster status
- `pvesh get /cluster/status` - Get cluster status via API

### Tier 2: Require Approval (Mutating)
**Behavior**: Pause execution and display approval modal to user

**kubectl commands**:
- `kubectl apply -f [file]` - Apply configuration
- `kubectl delete [resource]` - Delete resources
- `kubectl scale [deployment]` - Scale deployments
- `kubectl exec -it [pod]` - Execute command in container
- `kubectl port-forward` - Forward ports
- `kubectl patch` - Update resource fields
- `kubectl create` - Create resources
- `kubectl edit` - Edit resources

**System commands**:
- `ssh` - Remote shell access
- `scp` - Secure copy
- `chmod` - Change file permissions
- `chown` - Change file ownership
- `systemctl restart [service]` - Restart services
- `systemctl stop [service]` - Stop services
- `systemctl start [service]` - Start services
- `docker restart [container]` - Restart Docker containers
- `docker stop [container]` - Stop Docker containers
- `reboot` (with confirmation) - System reboot

**Proxmox commands**:
- `qm start [vmid]` - Start virtual machine
- `qm stop [vmid]` - Stop virtual machine
- `qm restart [vmid]` - Restart virtual machine

### Tier 3: Always Deny (Destructive)
**Behavior**: Immediate denial with clear reasoning

**Destructive operations**:
- `rm -rf` - Recursive force delete
- `mkfs` - Format filesystem
- `dd` - Low-level disk operations
- `fdisk` - Partition manipulation
- `parted` - Partition editing
- `shutdown` - System shutdown
- `init 0` - System halt
- `halt` - System halt
- `poweroff` - System power off
- `wipefs` - Wipe filesystem signatures

**Why Tier 3 is Denied**:
These commands can cause irreversible data loss, system downtime, or infrastructure damage. They should only be executed manually by authorized personnel with explicit intent.

## Pipe and Chain Analysis

The classifier analyzes complex command structures and escalates to the highest tier found:

### Piped Commands
```bash
# Tier 1: Both commands are read-only
kubectl get pods | grep nginx

# Tier 2: Second command is mutating (escalates entire chain)
kubectl get pods | kubectl delete -f -

# Tier 3: Contains destructive operation (entire chain denied)
cat /tmp/list.txt | xargs rm -rf
```

### Logical Operators
```bash
# Tier 2: Uses && to chain mutating operations
kubectl apply -f deployment.yaml && kubectl rollout status deployment/nginx

# Tier 2: Uses || for fallback (escalates to highest tier)
ssh server1 || ssh server2

# Tier 3: Contains destructive command (entire chain denied)
cd /tmp && rm -rf *
```

### Command Substitution
Commands using `$()` or backticks are flagged with a risk factor and analyzed recursively:

```bash
# Tier 2: Inner command is read-only, but ssh requires approval
ssh server "$(cat /tmp/script.sh)"

# Tier 3: Inner command is destructive (entire operation denied)
rm -rf $(find / -name "*.tmp")
```

## Approval Workflow

When a Tier 2 command is detected:

1. **Execution Paused**: Command execution stops before running
2. **Modal Displayed**: Real-time modal appears in the UI showing:
   - Full command text
   - Safety tier badge
   - Classification reasoning
   - Risk factors (if any)
   - Safety controls in place
3. **User Decision**: Three options available:
   - **Deny**: Reject the command permanently
   - **Allow Once**: Execute this specific command only
   - **Allow for Session**: Execute this and future similar commands in the current session
4. **Timeout**: If no response within 60 seconds, automatically deny

### Approval Modal Screenshot
```
┌─────────────────────────────────────────────────────────┐
│ 🛡️  Command Approval Required                           │
├─────────────────────────────────────────────────────────┤
│ This command requires your approval before execution    │
│                                                          │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ $ kubectl delete pod nginx-5d5f4c7d9-abcde          │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                          │
│ Safety Tier: [Tier 2]                                   │
│                                                          │
│ ⚠️  Why approval is needed:                             │
│ Mutating operation: kubectl delete                      │
│                                                          │
│ Safety Controls:                                         │
│ • Command execution is logged and auditable             │
│ • 30-second timeout protection                          │
│ • PII detection before execution                        │
│ • Output is captured for review                         │
│                                                          │
│ [Deny]  [Allow Once]  [Allow for Session]              │
└─────────────────────────────────────────────────────────┘
```

## kubectl Integration

### Bundled Binary
kubectl v1.30.0 is bundled with the application for all platforms:
- **Linux**: amd64, arm64
- **macOS**: Intel (x86_64), Apple Silicon (aarch64)
- **Windows**: amd64

The binary is automatically selected based on the runtime platform.

### Kubeconfig Management

**Upload Process**:
1. Navigate to **Settings → Kubeconfig**
2. Click **Upload Kubeconfig**
3. Select your kubeconfig file (.yaml or .yml)
4. Provide a friendly name (e.g., "production-cluster")
5. File is parsed and validated
6. Content is encrypted using AES-256-GCM
7. Stored in `kubeconfig_files` table

**Multiple Clusters**:
- Upload multiple kubeconfig files for different clusters
- Only one can be **active** at a time
- Activate a config by clicking **Activate** button
- Active config is used for all kubectl commands
- Cluster URL and context displayed for each config

**Auto-Detection**:
Kubeconfig auto-detection from `~/.kube/config` is implemented but not enabled at startup due to AppHandle state access limitations. Users must manually upload kubeconfig files via the UI.

### Environment Isolation
When kubectl commands execute:
- `KUBECONFIG` environment variable set to active config path
- Sensitive environment variables cleared (AWS credentials, etc.)
- Working directory isolated if specified
- 30-second timeout per command

## Command Execution Flow

### Full Execution Pipeline

1. **AI Tool Call**: AI invokes `execute_shell_command` tool with command text
2. **PII Detection**: Command text scanned for sensitive data (passwords, tokens, API keys)
3. **Audit Log (Pre-Execution)**: Command logged with hash chain before execution
4. **Classification**: CommandClassifier analyzes command structure and assigns tier
5. **Tier Decision**:
   - **Tier 1**: Proceed directly to execution
   - **Tier 2**: Emit `shell:approval-needed` event, wait for user response
   - **Tier 3**: Return error immediately with reasoning
6. **Execution** (if approved):
   - For kubectl: Use `execute_kubectl()` with active kubeconfig
   - For general: Use `tokio::process::Command` with 30s timeout
7. **Result Capture**: Capture exit code, stdout, stderr, execution time
8. **Database Record**: Store execution in `command_executions` table
9. **Audit Log (Post-Execution)**: Log result with exit code
10. **Return to AI**: Format output as text for AI analysis

### Error Handling
- **Timeout**: 30s command timeout, returns timeout error
- **Approval Timeout**: 60s approval timeout, command denied
- **Execution Failure**: Exit code != 0, stderr captured and returned
- **Classification Error**: Unparseable command, denied with reasoning
- **PII Detected**: Warning logged but execution continues (non-blocking)

## Audit Trail

All command executions are recorded in the `command_executions` table:

**Fields**:
- `id`: Unique UUID
- `command`: Full command text
- `tier`: Safety tier (1, 2, or 3)
- `approval_status`: "auto", "approved", or "denied"
- `kubeconfig_id`: Reference to active kubeconfig (if kubectl)
- `exit_code`: Command exit code
- `stdout`: Command output
- `stderr`: Error output
- `execution_time_ms`: Execution duration
- `executed_at`: Timestamp

**Audit Logging**:
All executions are also written to the audit log (`audit_events` table) with:
- Event type: `shell_command_execution`
- Entity type: `shell_command`
- Entity ID: Command text
- Details JSON: `{"command": "...", "exit_code": 0}`
- Hash chain linkage for tamper detection

**Viewing History**:
Navigate to **Settings → Shell Execution** to view recent command executions:
- Last 10 commands displayed
- Tier badge and approval status
- Exit code (green for 0, red for non-zero)
- Execution time
- Timestamp
- Collapsible stdout output

## Security Controls

### Encryption
- **Kubeconfig Files**: AES-256-GCM encryption at rest
- **Encryption Key**: Derived from `TFTSR_ENCRYPTION_KEY` environment variable
- **Nonce**: Random 12-byte nonce per encryption operation
- **Authentication Tag**: 16-byte tag for integrity verification

### PII Detection
Before execution, commands are scanned for:
- Passwords (e.g., `--password=secret`)
- API keys (patterns like `AKIAIOSFODNN7EXAMPLE`)
- Tokens (e.g., `token=abc123`)
- SSH keys (private key patterns)

If PII is detected:
- Warning logged with span count
- Execution continues (non-blocking)
- Consider sanitizing command history in future enhancement

### Command Injection Prevention
- No shell interpretation of user-provided arguments
- Arguments passed directly to `tokio::process::Command`
- kubectl arguments parsed from command string, not shell-interpreted

### Timeout Protection
- **Command Timeout**: 30 seconds per command
- **Approval Timeout**: 60 seconds for user response
- Prevents indefinite hangs or runaway processes

### Hash-Chained Audit Log
All executions recorded in audit log with:
- Previous event hash
- Current event data hash
- Timestamp
- Tamper detection via hash verification

## Settings

### Shell Execution Settings
**Location**: Settings → Shell Execution

**Features**:
- kubectl installation status and version display
- Link to Kubeconfig Manager
- Three-tier safety architecture visualization
- Recent command execution history (last 10)

### Kubeconfig Manager
**Location**: Settings → Kubeconfig

**Features**:
- Upload kubeconfig files (.yaml, .yml)
- List all uploaded configs with context and cluster URL
- Activate/deactivate configs
- Delete configs with confirmation
- Preview uploaded file content (first 500 chars)

## API Reference

### Backend Commands

#### `upload_kubeconfig`
Upload and encrypt a kubeconfig file.

**Parameters**:
- `name: String` - Friendly name for the config
- `content: String` - Full kubeconfig YAML content

**Returns**: `Result<String, String>` - Config ID on success

#### `list_kubeconfigs`
List all uploaded kubeconfig files.

**Returns**: `Result<Vec<KubeconfigInfo>, String>`

**KubeconfigInfo**:
```rust
pub struct KubeconfigInfo {
    pub id: String,
    pub name: String,
    pub context: String,
    pub cluster_url: Option<String>,
    pub is_active: bool,
}
```

#### `activate_kubeconfig`
Set a kubeconfig as active.

**Parameters**:
- `id: String` - Config ID to activate

**Returns**: `Result<(), String>`

#### `delete_kubeconfig`
Delete a kubeconfig file.

**Parameters**:
- `id: String` - Config ID to delete

**Returns**: `Result<(), String>`

#### `respond_to_shell_approval`
Respond to a shell command approval request.

**Parameters**:
- `approval_id: String` - Unique approval request ID
- `decision: String` - "deny", "allow_once", or "allow_session"

**Returns**: `Result<(), String>`

#### `list_command_executions`
List recent command executions.

**Parameters**:
- `issue_id: Option<String>` - Filter by issue ID (optional)

**Returns**: `Result<Vec<CommandExecution>, String>`

**CommandExecution**:
```rust
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
```

#### `check_kubectl_installed`
Check if kubectl is installed and get version info.

**Returns**: `Result<KubectlStatus, String>`

**KubectlStatus**:
```rust
pub struct KubectlStatus {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}
```

### AI Tool: `execute_shell_command`

**Description**: Execute shell commands with automatic safety classification.

**Parameters**:
- `command: String` (required) - Shell command to execute
- `working_directory: String` (optional) - Working directory for execution
- `kubeconfig_id: String` (optional) - Kubeconfig file ID for kubectl commands

**Returns**: String with formatted output:
```
Exit Code: 0

Stdout:
NAME                     READY   STATUS    RESTARTS   AGE
nginx-5d5f4c7d9-abcde   1/1     Running   0          5m

Stderr:
```

**Usage in AI Context**:
```typescript
{
  "name": "execute_shell_command",
  "arguments": {
    "command": "kubectl get pods -n production",
    "kubeconfig_id": "uuid-of-active-config"
  }
}
```

## Database Schema

### `shell_commands` (Migration 024)
Pre-defined command templates with tier classification.

```sql
CREATE TABLE IF NOT EXISTS shell_commands (
    id TEXT PRIMARY KEY,
    command_template TEXT NOT NULL,
    tier INTEGER NOT NULL CHECK(tier IN (1, 2, 3)),
    description TEXT,
    category TEXT NOT NULL,  -- 'kubectl', 'proxmox', 'general'
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### `kubeconfig_files` (Migration 025)
Encrypted kubeconfig storage.

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

### `command_executions` (Migration 026)
Full audit trail of all command executions.

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

### `approval_decisions` (Migration 027)
Session-based approval preferences.

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

## Testing

### Backend Tests
**Location**: `src-tauri/src/shell/`

**Classifier Tests** (`classifier.rs`):
- `test_tier1_kubectl_get` - Auto-execute kubectl get
- `test_tier2_kubectl_delete` - Require approval for kubectl delete
- `test_tier3_rm_rf` - Deny rm -rf
- `test_pipe_tier_escalation` - Piped command tier analysis
- 19 total tests covering all tier classifications

**kubectl Tests** (`kubectl.rs`):
- `test_locate_kubectl_finds_binary` - Binary location logic
- `test_kubectl_version_check` - Verify binary works
- `test_execute_kubectl_with_timeout` - Timeout implementation
- 3 total tests

**Executor Tests** (`executor.rs`):
- Currently ignored (require full app setup)
- Placeholder tests for approval flow

**Coverage**:
- Classifier: 100% (critical safety component)
- kubectl: 90%
- Executor: Needs integration test environment

### Frontend Tests
**Location**: `src/components/__tests__/`, `src/pages/__tests__/`

**Component Tests**:
- ShellApprovalModal: Event listener, modal rendering, button actions
- All existing tests passing (103 total)

### Integration Testing

**Manual Test Cases**:

1. **Tier 1 Auto-Execution**
   - AI request: "Show me all pods in the default namespace"
   - Expected: Command executes immediately without modal
   - Verify: `command_executions` has `approval_status='auto'`

2. **Tier 2 Approval Flow**
   - AI request: "Scale the nginx deployment to 5 replicas"
   - Expected: Approval modal appears
   - Test: Deny → execution blocked
   - Test: Allow Once → execution proceeds
   - Test: Allow for Session → execution proceeds

3. **Tier 3 Denial**
   - AI request: "Delete all files in /tmp"
   - Expected: No modal, immediate error with reasoning
   - Verify: Command not executed

4. **Piped Command Analysis**
   - Command: `kubectl get pods | grep nginx` → Tier 1 (auto-execute)
   - Command: `kubectl get pods | kubectl delete -f -` → Tier 2 (approval)
   - Command: `cat /tmp/list.txt | xargs rm -rf` → Tier 3 (deny)

5. **Timeout Protection**
   - Command: `sleep 60` → Times out after 30s
   - Approval: Wait 61s → Approval times out, command denied

6. **Audit Trail**
   - Query: `SELECT * FROM command_executions ORDER BY executed_at DESC`
   - Verify: All commands logged with correct tier, status, exit code

## Troubleshooting

### kubectl not found
**Problem**: "kubectl is not installed" message in Shell Execution settings

**Solutions**:
1. Check if kubectl is bundled: Binary should be at `Resources/kubectl` (macOS) or similar platform path
2. Verify PATH: Ensure system PATH includes kubectl location
3. Reinstall: Download latest application bundle with kubectl included

### Kubeconfig upload fails
**Problem**: "Failed to parse kubeconfig" error

**Solutions**:
1. Validate YAML: Ensure kubeconfig is valid YAML format
2. Check contexts: Kubeconfig must have at least one context defined
3. Cluster URL: Ensure cluster URL is accessible
4. File format: Only .yaml or .yml files accepted

### Commands not executing
**Problem**: Commands hang or don't execute

**Solutions**:
1. Check timeout: Commands timeout after 30 seconds
2. Approval timeout: User must respond within 60 seconds for Tier 2
3. Active kubeconfig: Ensure a kubeconfig is activated for kubectl commands
4. Review logs: Check audit log for denial reason

### Approval modal not appearing
**Problem**: Tier 2 command doesn't show approval modal

**Solutions**:
1. Check browser: Ensure JavaScript is enabled
2. Event listener: Modal listens for `shell:approval-needed` event
3. Tauri events: Verify Tauri event system is working
4. Console errors: Check browser console for errors

## Future Enhancements

**Planned Features**:
- Session-based approval preferences (approve all kubectl get for 1 hour)
- Command templating (save frequently used commands)
- Execution rollback (undo kubectl apply operations)
- Tier overrides (admin can override tier classification)
- Command history search and filtering
- Export execution history as CSV/JSON
- Integration with issue timeline (show commands executed during incident)
- Proxmox advanced commands (cluster management, backups)
- Multi-kubeconfig context switching within single file
- Auto-detection of ~/.kube/config on startup (pending AppHandle fix)

**Stretch Goals**:
- Parallel command execution (run multiple commands concurrently)
- Command scheduling (execute command at specific time)
- Command chaining with dependencies (run X, then Y if X succeeds)
- Command output parsing (extract structured data from stdout)
- Integration with monitoring systems (auto-execute commands on alerts)

## Related Documentation

- [[Architecture]] - Overall application architecture
- [[Security-Model]] - Security architecture and threat model
- [[Database]] - Database schema and migrations
- [[IPC-Commands]] - Frontend-backend communication
- [[AI-Providers]] - AI integration and tool use

## Version History

- **v1.0.0** (2026-06-02): Initial release with three-tier safety classification, kubectl bundling, and multi-cluster support
