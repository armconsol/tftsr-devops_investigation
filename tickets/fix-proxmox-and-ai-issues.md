# Fix: Proxmox Dashboard and AI Chat Issues

## Description

Resolved 7 bugs across the Proxmox management dashboard and the AI triage chat that rendered several features non-functional. Issues were verified against a live Proxmox VE host at `172.0.0.18`.

## Acceptance Criteria

- [x] VM action menu items (start, stop, reboot, shutdown, suspend, resume, migrate, clone, delete) execute and close the menu
- [x] Migration dialog presents a "Target Remote" dropdown listing all configured clusters for cross-datacenter migration
- [x] Storage page displays actual storage from PVE (cluster-wide, with correct size/used/available figures)
- [x] Network interface "Type" field is a dropdown of all valid PVE interface types, not a free-text input
- [x] Firewall "New Rule" button opens a dialog with action/protocol/source/dest/port fields that submits to PVE
- [x] Backup page renders all backup jobs configured in PVE with their storage target, schedule, VM list, and next-run time
- [x] AI chat with Qwen 3.5 (via LiteLLM) no longer throws `BadRequestError: System message must be at the beginning`
- [x] `cargo check`, `cargo clippy -D warnings`, `tsc --noEmit`, `eslint --max-warnings 0`, all Rust tests (432), all frontend tests (386) pass

## Work Implemented

### Issue 1 — VM Actions: Tauri param name mismatch + menu state

**Root cause**: `VMList` internally re-fetched cluster data on its own, causing a race condition; all VM action Tauri commands used `node: String` but the frontend sent `nodeId` (Tauri 2.x maps camelCase `nodeId` → snake_case `node_id`, so no value arrived); action menu buttons had no `onClick` handlers.

**Fix**:
- `src/pages/Proxmox/VMsPage.tsx`: pass `clusterId` and `clusters` props to `<VMList>`.
- `src/components/Proxmox/VMList.tsx`: accept those props, remove internal cluster `useEffect`, wire all action buttons through `handleAction()` wrapper (closes menu, stops propagation).
- `src-tauri/src/commands/proxmox.rs`: renamed `node: String` → `node_id: String` (and body usages) in `get_proxmox_vm`, `start_proxmox_vm`, `stop_proxmox_vm`, `reboot_proxmox_vm`, `shutdown_proxmox_vm`, `resume_proxmox_vm`, `suspend_proxmox_vm`, `clone_vm`, `delete_vm`.

### Issue 2 — Migration: no Target Remote option

**Fix**: `VMList.tsx` `MigrationDialog` now receives `clusters` and `currentClusterId` props; when more than one cluster is configured a "Target Remote" `<Select>` appears. Selecting a different cluster switches the node input to free-text (cross-cluster node names can't be enumerated locally). `submitMigration` passes `targetCluster` to `migrate_vm`. Rust `migrate_vm` had `node: String` renamed to `node_id: String` as part of the bulk rename above.

### Issue 3 — Storage: fields mismatch, empty display

**Root cause**: `list_proxmox_datastores` made N per-node requests and returned raw PVE fields (`avail`, `total`, `plugintype`); `StorageList` expected (`available`, `size`, `type`).

**Fix** (`src-tauri/src/commands/proxmox.rs`): replaced the N-request per-node loop with a single `cluster/resources?type=storage` call. Response normalization maps `plugintype` → `type`, `disk`/`maxdisk` → `used`/`size`, computes `available = maxdisk.saturating_sub(disk)`.

### Issue 4 — Network: Interface Type free-text

**Fix** (`src/pages/Proxmox/NetworkPage.tsx`): replaced `<Input>` with a `<Select>` listing all PVE network interface types: `eth`, `bond`, `bridge`, `vlan`, `OVSBridge`, `OVSBond`, `OVSIntPort`, `OVSPort`.

### Issue 5 — Firewall: "New Rule" button did nothing

**Root cause**: Button had no `onClick`. `FirewallRuleListProps` had no `onNewRule` prop.

**Fix**:
- `src/components/Proxmox/FirewallRuleList.tsx`: added `onNewRule?: () => void` to props interface; wired button.
- `src/pages/Proxmox/FirewallPage.tsx`: added full new-rule dialog (action, protocol, source, dest, dport, comment fields); calls `addFirewallRule(clusterId, nodeId, ruleObject)` on submit; refreshes list.
- `src-tauri/src/commands/proxmox.rs` `add_firewall_rule`: rewrote signature from 6 flat params to `rule: serde_json::Value` (matching what the frontend sends as a single object) plus `node_id: String` rename.

### Issue 6 — Backup: empty display

**Root cause**: PVE `cluster/backup` returns `{ id, storage, schedule, enabled, next-run }` but `BackupJobInfo` expected `{ name, node, status, lastRun, nextRun }` — no fields matched.

**Fix**:
- `src/pages/Proxmox/BackupPage.tsx`: added normalizer that maps `id` → `name`, derives `enabled` (0/1/bool), converts `next-run` unix timestamp to locale string.
- `src/components/Proxmox/BackupJobList.tsx`: added `storage`, `vmid`, `mode`, `comment` optional fields to interface; updated table columns to show ID, Storage, VMs, Node, Schedule, Enabled, Next Run, Mode.

### Issue 7 — AI chat: system message ordering / Qwen 3.5 rejection

**Root cause**:
1. `chat_message` in `src-tauri/src/commands/ai.rs` pushed 4–5 consecutive `system` role messages before history. Qwen 3.5 (and LiteLLM's OpenAI compatibility layer) rejects anything but a single system message at position 0.
2. The tool-calling loop pushed `tool` role messages without first emitting the assistant message that contains `tool_calls` — violating the OpenAI API contract.

**Fix**:
1. All system prompt sections (agent prompt, domain prompt, tool instructions, integration context) are now collected into a `Vec<String>` and merged with `"\n\n---\n\n"` into a single `Message { role: "system" }` before history.
2. When tool calls are present, the assistant's response (with `tool_calls` populated) is pushed to the message history before any tool result messages.

## Testing Needed

- [ ] Start the app against Proxmox host `172.0.0.18`; verify all VM action menu items execute on a running VM
- [ ] Trigger a migration from one node to another (same cluster); verify the dialog lists nodes and submits
- [ ] If multiple clusters configured: verify "Target Remote" dropdown appears in Migration dialog
- [ ] Navigate to Storage page; verify all storage volumes appear with correct used/total/available figures
- [ ] Open Network → Add Interface; verify Type field is a dropdown with all interface types
- [ ] Open Firewall → New Rule; fill in action/protocol/port; verify rule is created in PVE
- [ ] Open Backup page; verify backup jobs configured in PVE appear with storage target and next-run time
- [ ] Start an AI chat session using a Qwen 3.5 model via LiteLLM; verify no `BadRequestError` and tool calls work correctly
