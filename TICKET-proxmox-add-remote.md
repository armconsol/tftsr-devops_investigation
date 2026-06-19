# Ticket: Fix Proxmox Add Remote Failure

## Description

Clicking "Add Remote" in the Proxmox Remotes page always produces the error dialog **"Add New Remote / Error / Failed to add remote"** regardless of what credentials are entered. The root cause is a chain of three bugs:

1. **`password: &str` in async Tauri v2 command** (`src-tauri/src/commands/proxmox.rs`): Tauri v2 requires command parameters to implement `DeserializeOwned + Send`. `&str` is lifetime-bound and does not implement `DeserializeOwned`, so the IPC layer fails to deserialize the payload at runtime — the command body never runs.

2. **Generic error fallback in frontend** (`src/components/Proxmox/AddRemoteForm.tsx`): The catch block uses `err instanceof Error ? err.message : 'Failed to add remote'`. Tauri errors arrive as plain strings, not `Error` objects, so the fallback branch always fires and the real error is hidden.

3. **Missing protocol/port in `ProxmoxClient` URL builder** (`src-tauri/src/proxmox/client.rs`): The frontend strips the protocol via `parseRemoteUrl` before sending just the bare hostname. `get_api_url()` and `authenticate()` were constructing URLs like `"172.0.0.18/api2/json/..."` — no scheme, no port — meaning all subsequent Proxmox API calls (VMs, containers, etc.) would silently fail even after a successful add.

## Acceptance Criteria

- [ ] Submitting the Add Remote form with valid credentials stores the remote and closes the dialog.
- [ ] Submitting with invalid credentials shows the actual backend error string, not the generic fallback.
- [ ] All existing Rust tests pass (416).
- [ ] `cargo clippy -- -D warnings`, `tsc --noEmit`, `eslint --max-warnings 0` all pass.
- [ ] Proxmox API calls (VMs, containers) use correctly formed `https://{host}:{port}/api2/json/...` URLs.

## Work Implemented

**Branch**: `fix/proxmox-add-remote` → `beta` | **PR**: https://gogs.tftsr.com/sarman/tftsr-devops_investigation/pulls/122

| File | Change |
|------|--------|
| `src-tauri/src/commands/proxmox.rs` | `password: &str` → `password: String` (line 41) |
| `src-tauri/src/proxmox/client.rs` | `get_api_url()` now produces `https://{host}:{port}/api2/json/...`; same for `authenticate()`; tests updated to reflect bare-hostname input |
| `src/components/Proxmox/AddRemoteForm.tsx` | `err instanceof Error ? err.message : 'Failed to add remote'` → `String(err)` |

## Testing Needed

1. Build and run the app (`cargo tauri dev` or install release binary).
2. Navigate to **Proxmox → Remotes**.
3. Click **Add Remote**, fill in a valid hostname (e.g., `https://172.0.1.42:8006`), username `root@pam`, password, type PVE.
4. Confirm the dialog closes and the remote appears in the list.
5. Repeat with a wrong password — confirm the actual Tauri error message is shown in the dialog (not the generic fallback).
6. Navigate to a cluster's VM or container list to confirm Proxmox API calls form correct URLs.
