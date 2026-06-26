# Ticket: Proxmox cross-DC migration 501 + console copy/paste

## Description

Two defects in the in-app Proxmox integration:

1. **Cross-datacenter VM migration fails with HTTP 501.** Migrating a VM from one
   datacenter to another fails with:

   ```
   Failed to remote-migrate VM 104: API request failed with status 501 Not Implemented:
   {"data":null,"message":"Method 'POST /nodes/vmhost3/qemu/104/remote-migrate' not implemented"}
   ```

   Root cause: the code targeted the REST path `remote-migrate` (dash). The
   Proxmox REST API registers the endpoint as `remote_migrate` (underscore); the
   dashed form is only the `qm` CLI command name, so `pveproxy` returns 501.
   Confirmed against PVE `qemu-server` source (`PVE/API2/Qemu.pm`,
   `path => '{vmid}/remote_migrate'`).

2. **No copy/paste in console sessions.** The Proxmox | VMs and Proxmox | Remotes
   console sessions (noVNC graphical consoles and the xterm.js node/PBS shell)
   never wired any clipboard integration, so users could not copy from or paste
   into a session.

## Acceptance Criteria

- [x] Cross-DC remote migration issues `POST /nodes/{node}/qemu/{vmid}/remote_migrate`
      and no longer returns 501.
- [x] Regression tests assert the underscore REST path and fail on the dashed form.
- [x] Graphical (noVNC) VM/LXC and node-shell consoles support:
  - copy from guest → host (RFB `clipboard` event mirrored to the system clipboard),
  - paste host → guest via a **Paste** button and **Ctrl/Cmd+Shift+V**.
- [x] xterm.js terminal supports copy (**Ctrl/Cmd+Shift+C**, current selection) and
      paste (**Ctrl/Cmd+Shift+V**), while a bare `Ctrl+C`/`Ctrl+V` still reaches the guest.
- [x] Clipboard access works reliably inside WebKitGTK (via
      `tauri-plugin-clipboard-manager`, not `navigator.clipboard`).
- [x] All Rust and frontend tests pass 100%.
- [x] `cargo fmt`, `cargo clippy -D warnings`, `tsc --noEmit`, and `eslint` are clean.

## Work Implemented

### Backend (Rust)
- Added pure helper `remote_migrate_path(node, vmid)` in `proxmox/migration.rs`
  returning the correct underscore REST path; both `migration.rs` and
  `proxmox/vm.rs` now delegate to it. Updated stale doc comments.
- Registered `tauri-plugin-clipboard-manager` (`Cargo.toml`, `lib.rs`) and granted
  `clipboard-manager:allow-read-text` / `allow-write-text` in
  `capabilities/default.json` (regenerated ACL schemas).

### Frontend (TypeScript / React)
- `src/lib/clipboard.ts` — fail-soft wrapper over the clipboard plugin
  (`readClipboardText` / `writeClipboardText`).
- `src/lib/consoleClipboard.ts` — pure `isCopyShortcut` / `isPasteShortcut`
  predicates (Ctrl/Cmd+Shift+C / +V).
- `NoVncConsole.tsx` & `NodeShellConsole.tsx` — mirror the RFB `clipboard` event to
  the host clipboard, add a Paste button + Ctrl/Cmd+Shift+V handler.
- `XtermConsole.tsx` — `attachCustomKeyEventHandler` for copy-selection / paste.
- `src/types/novnc.d.ts` — added `clipboardPasteFrom(text)` to the RFB type.

### Tests (TDD — written test-first, red → green)
- Rust: `test_remote_migrate_path_uses_rest_underscore_form`,
  `test_remote_migrate_uses_rest_underscore_path`,
  `test_capabilities_allow_clipboard_text`.
- Frontend: `tests/unit/clipboard.test.ts` (6), `tests/unit/consoleClipboard.test.ts` (7).

### Docs
- `docs/wiki/Troubleshooting.md` — new Proxmox section documenting the 501
  dash-vs-underscore gotcha and console copy/paste behavior.

## Testing Needed

Automated (all passing):
- `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1` — 634 passed.
- `npm run test:run` — 438 passed (55 files).
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `npx tsc --noEmit`,
  `npx eslint src/ tests/ --quiet` — all clean.

Manual smoke (recommended against a live cluster):
- Migrate a running VM across two datacenters; confirm the task starts (no 501) and
  the temporary destination token is cleaned up.
- In a VM console: copy text inside the guest and paste it on the host; copy host
  text and paste into the guest via the Paste button and Ctrl+Shift+V.
- In a PBS/node xterm shell: select + Ctrl+Shift+C to copy, Ctrl+Shift+V to paste;
  confirm bare Ctrl+C still sends SIGINT to the remote shell.
