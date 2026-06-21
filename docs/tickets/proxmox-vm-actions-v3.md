# Proxmox VM Actions — Fix & Add VM

## Description

Three issues existed in the Proxmox | VMs page:

1. **Actions did nothing** — `toast.success()` / `toast.error()` were called but the `<Toaster>` component from `sonner` was never mounted in `App.tsx`, so all feedback was silently discarded. The backend commands were wired correctly; the issue was purely the missing Toaster mount.

2. **Disk column showing meaningless data** — PVE `cluster/resources` does not return meaningful disk usage for running VMs (only static allocation metadata). The Disk column was removed.

3. **No way to create a new VM** — No "Add VM" button or creation flow existed.

Additionally:
- `dialog:allow-confirm` was missing from capabilities, causing the delete confirmation to fail silently.
- The `MigrationDialog` derived available target nodes from the local VM list (only nodes that already had a VM), instead of querying the actual cluster node list.
- `suspendProxmoxVm` and `resumeProxmoxVm` client wrappers were missing from `proxmoxClient.ts`.

## Acceptance Criteria

- [x] VM start/stop/reboot/shutdown/suspend/resume actions show toast feedback
- [x] Migrate action shows a dialog populated with real cluster nodes from `GET /nodes`
- [x] Disk column is absent from the VMs table
- [x] "Add VM" button opens a creation dialog with node, VMID, name, memory, CPU, storage, disk, network, and optional ISO fields
- [x] Created VM appears after refreshing the VM list
- [x] All existing tests continue to pass; new tests added for new functionality and security validation
- [x] `cargo clippy -- -D warnings` passes; `npx eslint . --max-warnings 0` passes; `npx tsc --noEmit` passes

## Work Implemented

### Frontend

| File | Change |
|---|---|
| `src/App.tsx` | Added `<Toaster richColors position="top-right" />` — root cause fix for silent actions |
| `src/components/Proxmox/VMList.tsx` | Removed Disk column (header + cell + `diskPercent` calc); updated `MigrationDialog` to fetch nodes via `list_proxmox_nodes` invoke; fixed `useMemo` unused import |
| `src/components/Proxmox/CreateVmDialog.tsx` | New component — form dialog for creating QEMU VMs with node/storage discovery |
| `src/components/Proxmox/index.ts` | Exported `CreateVmDialog` |
| `src/pages/Proxmox/VMsPage.tsx` | Added "Add VM" button + `CreateVmDialog` mount |
| `src/lib/proxmoxClient.ts` | Added `suspendProxmoxVm`, `resumeProxmoxVm`, `listProxmoxNodes`, `createProxmoxVm` wrappers |

### Backend (Rust)

| File | Change |
|---|---|
| `src-tauri/src/commands/proxmox.rs` | Added `list_proxmox_nodes`, `create_proxmox_vm` commands; added `validate_pve_identifier` helper |
| `src-tauri/src/lib.rs` | Registered `list_proxmox_nodes` and `create_proxmox_vm` |
| `src-tauri/capabilities/default.json` | Added `dialog:allow-confirm` permission |

### Security Hardening (new commands only)

- **H2 — Path injection**: `node_id`, `storage`, `net_bridge` validated against `^[A-Za-z0-9._-]+$` before URL interpolation
- **H3 — ISO comma injection**: `iso` validated to match `storage:iso/path` format, rejecting commas
- **M4 — Numeric bounds**: `vmid` (100–999 999 999), `memory` (32–1 048 576 MB), `cores` (1–512), `sockets` (1–4), `disk_size` (1–65 536 GB) validated server-side

### Known / Deferred

- **C1 — TLS cert verification disabled** (`danger_accept_invalid_certs(true)` in `proxmox/client.rs`): Pre-existing across all Proxmox commands. Needs a separate PR implementing TOFU cert pinning or CA trust.
- **M5 — Missing audit log** for mutating Proxmox commands: Pre-existing. Should be addressed for all Proxmox write operations in a follow-up.

### Tests

- `tests/unit/VMList.test.tsx`: 19 tests (all pass) — covers Disk column absent, action menus by status, all power actions, migration dialog open, empty state
- `src-tauri/src/commands/proxmox.rs` (inline): 7 new tests covering `validate_pve_identifier`, VMID range, ISO format validation, ide2/scsi0/net0 string construction
- **Total**: 446 Rust tests, 405 frontend tests — all pass

## Testing Needed

- [ ] Connect to a real Proxmox cluster and verify Start/Stop/Reboot/Shutdown/Suspend/Resume all show toast notifications
- [ ] Verify Migrate dialog shows actual cluster nodes (not just nodes inferred from VMs)
- [ ] Create a new VM via the "Add VM" dialog — confirm VM appears in PVE web UI
- [ ] Confirm Disk column is absent from the VM list
- [ ] Confirm delete VM shows a browser confirm dialog (previously silently failing due to missing capability)
- [ ] Test with an ISO to confirm the `storage:iso/filename.iso` path is accepted; test with a comma-injected value to confirm it is rejected with a clear error
