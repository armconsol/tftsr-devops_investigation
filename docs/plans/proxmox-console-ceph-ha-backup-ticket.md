# Ticket: fix(proxmox) — Console, Ceph, HA Groups & Backup actions

## Description

Five defects were reported against the Proxmox integration in the beta build:

1. **Remotes → host console** failed with
   `Shell error — Failed to render console: SecurityError: The operation is insecure.`
2. **VMs → console** failed with
   `Console error — Failed to open console: SecurityError: The operation is insecure.`
3. **Ceph** page flashed data then went blank when switching to a Ceph-enabled host.
4. **HA Groups** failed on PVE 9 with
   `API request failed with status 500 … cannot index groups: ha groups have been migrated to rules`.
5. **Backup** action buttons (Trigger / Edit) did nothing.

### Root causes

1 & 2. **CSP blocked the local console WebSocket.** The backend starts a local
   bridge and hands the webview `ws://127.0.0.1:<port>`, but the bundled CSP
   `connect-src` listed only `http(s)` origins. WebKitGTK raises
   `SecurityError: The operation is insecure` when a WebSocket violates
   `connect-src`. (Loopback `ws://` is spec "potentially trustworthy", so this is
   a CSP rejection, not mixed content.)

3. **Double `data` unwrap.** `ProxmoxClient::get` already strips the Proxmox
   `{ "data": … }` envelope, but `proxmox::ceph::get_ceph_health` unwrapped
   `data` a second time, so health always errored and collapsed the page.

4. **PVE 9 removed `cluster/ha/groups`.** HA groups were migrated to HA rules.
   `list_ha_groups` had no fallback and surfaced the raw 500.

5. **Actions not wired + wrong trigger endpoint.** `BackupPage` never passed
   `onTrigger`/`onEdit` to `BackupJobList`, and `trigger_proxmox_backup_job`
   targeted a non-existent `nodes/{node}/backup/jobs/{id}/run` path using a `u32`
   id (real job ids are strings).

## Changes

- **Console (CSP):** add `ws://127.0.0.1:* ws://localhost:*` to `connect-src` in
  `src-tauri/tauri.conf.json`. Loopback-only, port-wildcarded; no remote origin
  is permitted. Guard test in `proxmox/console.rs` asserts the allowance exists.
- **Ceph:** extract pure `parse_ceph_health()` and remove the extra `data`
  unwrap; tolerate string/array `summary`. Harden `CephHealthWidget` against a
  missing `details`/`status`/`health` so a partial payload can never blank the
  page.
- **HA:** add `parse_ha_rules_as_groups()` and fall back from
  `cluster/ha/groups` to `cluster/ha/rules` when PVE reports the migration,
  mapping `node-affinity` rules to the existing `HaGroup` shape.
- **Backup:** rework `trigger_proxmox_backup_job` to take a `String` job id,
  read the job config, choose a node (`select_backup_node`), and run it via
  `POST nodes/{node}/vzdump` with params from `build_vzdump_params`. Wire
  `onTrigger` and a prefilled `onEdit` dialog in `BackupPage`; update the TS
  wrapper.
- **Security:** add npm `overrides` pinning patched versions of
  `js-yaml`, `undici`, `esbuild`, `serialize-javascript`, `dompurify`,
  `@babel/core` → `npm audit` now reports **0 vulnerabilities**.

## Acceptance criteria

- [ ] Remotes host console and VM console open without the SecurityError.
- [ ] Ceph page renders health/pools/OSDs on a Ceph-enabled node (no blank).
- [ ] HA Groups loads on PVE 9 (rules shown as groups) and on older PVE.
- [ ] Backup Trigger runs a backup; Edit opens prefilled and saves.
- [ ] `npm audit` reports 0 vulnerabilities.

## Testing

- Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`
  (631 passed) — new tests for CSP guard, `parse_ceph_health`,
  `parse_ha_rules_as_groups` + migration detection, `build_vzdump_params` /
  `select_backup_node`.
- Frontend: `npx tsc --noEmit`, `npx eslint`, `npm run test:run` (425 passed)
  — new `CephHealthWidget` defensive-render and `BackupPage` action-wiring tests.
- `npm run build` succeeds with the dependency overrides.

### Manual verification still required (needs a live PVE/PBS host)

- Console rendering on the Linux/WebKitGTK target (CSP fix is platform-specific).
- "Trigger now" against a real `cluster/backup` job.

### Notes

- HA create/edit/delete still target the legacy `cluster/ha/groups` endpoints;
  on PVE 9 these may need migrating to `cluster/ha/rules` in a follow-up. The
  reported defect (list/load) is resolved.
- E2E (WebdriverIO) not run here; requires a compiled binary.
