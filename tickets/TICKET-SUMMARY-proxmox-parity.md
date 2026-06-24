# Proxmox Feature-Parity Fixes — Ticket Summary

## Description

Twelve reported Proxmox issues in the TRCAA desktop app were preventing 100%
feature parity with `proxmox/proxmox-datacenter-manager`. This work fixes all
twelve across the Rust (Tauri) backend and the React/TypeScript frontend,
following TDD. All new and pre-existing tests pass.

Issues addressed:

1. Missing **Console (VNC)** action in the VM actions menu.
2. Cross-datacenter **migration reported false success** while Proxmox failed
   ("No route to host" / "Can't connect to destination address").
3. **Storage** action items were not wired up.
4. **Network**: node field was free-text and nothing rendered.
5. **Ceph**: no data, no datacenter selector.
6. **HA Groups**: failed to reload on datacenter change ("Failed to load HA groups").
7. **HA Groups/Resources**: no way to edit; action items did nothing.
8. **Backup**: "Failed to load backup jobs" from remote datacenters.
9. **Tasks**: no datacenter selector.
10. **Views**: dead server call returned a "not implemented" message.
11. **Updates**: "Failed to load APT updates / repositories".
12. **Administration & Node Details**: blank page + lost navigation (required restart).

## Acceptance Criteria

- [x] A working in-app **Console (VNC)** opens a live graphical console for VMs
      and containers.
- [x] Cross-datacenter migration actually migrates (or surfaces the **real**
      Proxmox error) and the UI polls the task instead of reporting instant success.
- [x] Storage Edit/Delete actions are functional and backed by real commands.
- [x] Network and Ceph pages provide datacenter + node selectors and auto-load data.
- [x] HA Groups reload correctly on datacenter change; HA groups and resources
      are editable.
- [x] Backup jobs load from remote datacenters (tolerant parsing) with real errors surfaced.
- [x] Tasks page provides a datacenter selector.
- [x] Views provides a working, locally persisted saved-views experience.
- [x] APT updates/repositories load via node dropdown.
- [x] Administration and Node Details no longer blank the app or lose navigation.
- [x] 100% of tests pass (new + pre-existing); lints/format/type checks clean.

## Work Implemented

**Cross-cutting**
- Route-level `ErrorBoundary` (keyed on pathname) keeps the sidebar/navigation
  alive when a page throws (#12), plus fixes to the Admin/Node Detail render paths.
- Reusable datacenter + node dropdown pattern with auto-select/auto-load (#4, #5,
  #9, #11).

**Backend (Rust / Tauri)**
- `proxmox/console.rs` (new): `vncproxy_vm` / `vncproxy_lxc`, vncproxy response
  parsing, `vncwebsocket` URL + auth-cookie builders, and a local WebSocket proxy
  bridging noVNC to PVE (cookie injection, self-signed TLS). Commands
  `open_vnc_console` / `open_lxc_console` (#1).
- `proxmox/migration.rs`: real `remote-migrate` support, `target-endpoint`
  builder, fingerprint normalization/auto-fetch, and task exit-status
  interpretation; command `start_remote_migration` auto-manages a temporary
  destination API token (#2).
- Storage config commands (`get_proxmox_storage_config`,
  `update_proxmox_storage`, `delete_proxmox_storage`) + param builder (#3).
- Tolerant HA/backup parsing (handles `data: null` from standalone remotes);
  `update_ha_group` / `update_ha_resource` (#6, #7, #8).

**Frontend (React / TypeScript)**
- `@novnc/novnc` RFB renderer (`NoVncConsole`) on a dedicated route
  `/proxmox/console/:clusterId/:node/:vmid/:kind`; Console (VNC) menu item (#1).
- Migration dialog: Target Storage + Target Bridge inputs (cross-DC only), live
  progress/log, task polling, real success/error, temp-token cleanup (#2).
- Storage Edit/Delete dialogs; HA group/resource edit dialogs; datacenter/node
  selectors for Network/Ceph/Tasks/Updates; local persisted saved-views
  (`src/lib/savedViews.ts`) (#3–#11).

## Testing Needed

- **Automated (all passing):** Rust `cargo test` (601 passed, 6 ignored),
  frontend `vitest` (420 passed), `tsc --noEmit`, `cargo fmt --check`,
  `cargo clippy -D warnings`, `eslint`, and `npm run build`.
- **Manual against a live multi-datacenter Proxmox + Ceph cluster:**
  - Open Console (VNC) for a running VM and a container; verify keyboard/mouse,
    Ctrl+Alt+Del, reconnect, and self-signed-cert nodes.
  - Cross-DC migrate a VM; verify live progress, success, and that a genuine
    failure surfaces the real Proxmox error (no false success). Confirm the
    temporary destination token is removed afterward.
  - Verify Storage edit/delete, Network/Ceph/Tasks/Updates datacenter+node
    auto-load, HA reload on DC change and HA group/resource edits, backup job
    loading from remote DCs, saved-views persistence, and that Administration /
    Node Details no longer blank the app.
