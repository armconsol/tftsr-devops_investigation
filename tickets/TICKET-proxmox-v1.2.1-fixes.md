# Proxmox PDM v1.2.1 — Bug Fixes & 100% Feature Parity

## Description

This ticket tracks the v1.2.1 release of the Proxmox integration in TRCAA, which delivers 100% feature parity with upstream Proxmox Datacenter Manager (PDM) and resolves four reported UX issues.

The implementation was cross-referenced against the PDM source at https://github.com/proxmox/proxmox-datacenter-manager/tree/master.

## Acceptance Criteria

- [ ] Auto-updater is in Settings > Updater, not under Proxmox settings
- [ ] Proxmox sidebar section is collapsed by default
- [ ] No dummy/hardcoded data visible anywhere in the Proxmox section
- [ ] Adding and saving a Proxmox remote (VE or PBS) works end-to-end
- [ ] All 17 PDM feature phases implemented or marked out-of-scope with justification
- [ ] TypeScript: 0 errors
- [ ] ESLint: 0 warnings
- [ ] Rust: `cargo check` clean

## Work Implemented

### Bug Fixes
1. Auto-updater relocated to Settings > Updater page
2. Proxmox settings persist via localStorage (port, timeout, retry, SSL, caching, debug)
3. ACL page dummy data removed; loads from live cluster
4. EditRemoteForm: added missing password field; Refresh button functional
5. Proxmox nav section collapsed by default (accordion)

### Feature Phases (PDM Parity)
- **Phase 8**: HA Groups Manager (HAGroupsList, HAResourcesList, real backend)
- **Phase 9**: User Management (AclList, UserList, RealmList, multi-tab ACL page)
- **Phase 10**: Certificate Manager (CertificateList with expiry coloring, ACME, upload)
- **Phase 11**: Subscription Registry (per-cluster status, key management)
- **Phase 12**: Notes System (view/edit cluster notes)
- **Phase 13**: Resource Search (cross-cluster full-text search)
- **Phase 14**: Custom Views (CRUD for named resource views)
- **Phase 15**: Connection Health (connected/disconnected status per cluster)
- Administration Panel (Node Status, APT Updates, Repos, Syslog, Tasks)
- Network Management (interface list with type/status/addressing)
- Tasks page (live cluster task log, status badges)
- 20 new TypeScript client functions + 20 Rust command stubs

### Version
- `package.json`, `tauri.conf.json`, `Cargo.toml`: bumped to 1.2.1

## Testing Needed

- [ ] Settings > Updater loads and shows correct channel
- [ ] Settings > Proxmox: Save button persists values; Reset restores defaults
- [ ] Proxmox nav collapsed on app start; click to expand
- [ ] Remotes: Add a PVE remote — fills form, submits, appears in list
- [ ] Remotes: Edit a remote — password field visible, save works
- [ ] Remotes: Refresh button reloads the list
- [ ] Access Control: No dummy data; ACL/Users/Realms tabs load from backend
- [ ] HA Groups: Creates and lists HA groups
- [ ] Certificates: Loads certs, shows expiry colors
- [ ] Subscription: Shows per-cluster subscription status
- [ ] Notes: View and edit cluster notes
- [ ] Search: Returns results across clusters
- [ ] Admin: Node Status shows CPU/memory; Syslog scrolls entries
- [ ] Network: Lists network interfaces per node
- [ ] Tasks: Lists recent cluster tasks
- [ ] Views: Create and delete a custom view
