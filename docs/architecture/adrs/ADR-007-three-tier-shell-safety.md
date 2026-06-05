# ADR-007: Three-Tier Shell Command Safety Classification

**Date**: 2026-06-02  
**Status**: Accepted  
**Deciders**: Shaun Arman, Henry Castle, RJ Cooper  
**Context**: Hackathon v1.0.0 — Agentic Shell Execution

---

## Context

TFTSR DevOps Investigation v1.0.0 introduced agentic shell command execution, allowing AI agents to execute kubectl, Proxmox, and general shell commands during troubleshooting conversations. This capability creates a significant security risk: malicious or hallucinated commands could cause data loss, service disruption, or unauthorized system access.

**Requirements**:
- AI agents need shell access for diagnostics (kubectl, pvecm, qm, etc.)
- Read-only operations should execute immediately for fast iteration
- Mutating operations require explicit user approval
- Destructive operations must be blocked entirely
- Classification must handle pipes, chains, and command substitution
- System must be deterministic and testable

**Alternatives Considered**:

1. **Whitelist-only approach**: Maintain a fixed list of allowed commands
   - ✅ Simple to implement
   - ❌ Brittle — breaks with new commands or options
   - ❌ Poor UX — blocks legitimate commands like `kubectl get pods -n custom-namespace`

2. **Blacklist-only approach**: Block known-dangerous commands
   - ✅ Flexible for new commands
   - ❌ Fails-open — unknown dangerous commands execute
   - ❌ False sense of security

3. **LLM-based classification**: Ask another AI to classify command safety
   - ✅ Context-aware decisions
   - ❌ Non-deterministic — same command gets different classifications
   - ❌ Latency — adds 500ms+ per command
   - ❌ Cost — every command requires an AI call
   - ❌ Cannot unit test

4. **Sandbox all commands**: Execute in isolated containers
   - ✅ Maximum safety
   - ❌ Complex infrastructure
   - ❌ Breaks kubectl (needs real cluster access)
   - ❌ High latency

---

## Decision

**Implement a deterministic three-tier safety classification system with static analysis and rule-based tier assignment.**

### Tier Definitions

| Tier | Safety Level | Approval | Examples |
|------|--------------|----------|----------|
| **Tier 1** | Read-only, no side effects | Auto-execute | `kubectl get`, `describe`, `logs`, `cat`, `grep`, `ls`, `pvecm status`, `qm status` |
| **Tier 2** | Mutating, potentially disruptive | User approval required | `kubectl apply`, `delete`, `scale`, `chmod`, `systemctl restart`, `ssh`, `chown` |
| **Tier 3** | Destructive, unrecoverable | Always deny | `rm -rf`, `shutdown`, `reboot`, `mkfs`, `dd if=/dev/zero`, `:(){:\|:&};:` (fork bomb) |

### Classification Rules

1. **Single command**: Classify by command + subcommand pattern
   - `kubectl get` → Tier 1
   - `kubectl apply` → Tier 2
   - `rm -rf` → Tier 3

2. **Piped commands** (`|`): Highest tier wins
   - `kubectl get pods | grep nginx` → max(Tier 1, Tier 1) = Tier 1
   - `cat /etc/passwd | tee /tmp/backup` → max(Tier 1, Tier 2) = Tier 2

3. **Command chains** (`&&`, `||`, `;`): Highest tier wins
   - `ls && cat file` → max(Tier 1, Tier 1) = Tier 1
   - `kubectl delete pod nginx && kubectl get pods` → max(Tier 2, Tier 1) = Tier 2

4. **Command substitution** (`` `...` ``, `$(...)`): Escalate Tier 1 to Tier 2
   - `kubectl get pods $(cat namespace.txt)` → Tier 2 (even if `kubectl get` is Tier 1)
   - Rationale: Command substitution introduces hidden indirection

5. **Any Tier 3 in chain**: Entire command becomes Tier 3
   - `ls && rm -rf /` → Tier 3 (entire command denied)

### Implementation

**Backend**: `src-tauri/src/shell/classifier.rs`

```rust
pub enum CommandTier {
    Tier1, // Auto-execute
    Tier2, // Requires approval
    Tier3, // Always deny
}

impl CommandClassifier {
    pub fn classify(&self, command: &str) -> ClassificationResult {
        // Parse command structure (pipes, chains, substitution)
        let components = Self::parse_command_structure(command);
        
        // Classify each component and find highest tier
        let mut highest_tier = CommandTier::Tier1;
        for component in &components {
            let tier = self.classify_single_command(&component.command, ...);
            if tier > highest_tier {
                highest_tier = tier;
            }
        }
        
        // Escalate if command substitution detected
        if command.contains("$(") || command.contains("`") {
            if highest_tier == CommandTier::Tier1 {
                highest_tier = CommandTier::Tier2;
            }
        }
        
        ClassificationResult { tier: highest_tier, ... }
    }
}
```

**Testing**: 19 unit tests cover all classification rules, edge cases, and escalation logic.

---

## Consequences

### Positive

- **Deterministic**: Same command always gets same classification (unit testable)
- **Fast**: Regex-based classification completes in <1ms (no AI calls)
- **User-friendly**: Read-only commands execute immediately without prompts
- **Safe defaults**: Unknown commands default to Tier 2 (approval required)
- **Transparent**: UI shows tier reasoning ("mutating operation", "contains command substitution")
- **Session memory**: User can "Allow for Session" to approve multiple similar Tier 2 commands

### Negative

- **Maintenance burden**: New commands require manual tier assignment
- **False negatives**: Benign commands may be over-classified (e.g., `kubectl run --dry-run=client` is Tier 2 but harmless)
- **Bypass via arguments**: `cat /etc/shadow` is Tier 1 (read-only) but accesses sensitive data
  - **Mitigation**: Context matters — AI should not ask to read `/etc/shadow` without reason
  - **Mitigation**: Full audit log records all commands for security review

### Trade-offs

We chose **correctness and safety over flexibility**. A false positive (over-restricting a safe command) is acceptable; a false negative (allowing a destructive command) is not.

---

## Related Decisions

- **ADR-008**: MCP Protocol Integration (provides alternative tool integration method)
- **ADR-009**: Bundle kubectl Binary (ensures consistent kubectl version across platforms)

---

## References

- **Implementation PR**: #30 (Hackathon v1.0.0)
- **Test Coverage**: `src-tauri/src/shell/tests.rs` (19 tests)
- **Wiki**: `docs/wiki/Shell-Execution.md`
- **Database Schema**: Migrations 024-027 (shell_commands, kubeconfig_files, command_executions, approval_decisions)
