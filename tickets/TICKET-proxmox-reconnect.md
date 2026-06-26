# fix(proxmox): Reliable connect/reconnect after app restart

## Description

Adding a new Proxmox host reported "connected" immediately, but after closing
the app or clicking Disconnect the host could not be reconnected.

Three compounding bugs caused this:

1. **`authenticate()` could never succeed on reconnect** — `ProxmoxClient::authenticate()`
   called `response.json::<AuthResponse>()` but the Proxmox API wraps every
   response in `{"data": {...}}`.  The deserialiser always failed with "missing
   field `ticket`", so `get_proxmox_client_for_cluster` (and the new
   `connect_proxmox_cluster`) threw an error on every app-restart reconnect.

2. **False "connected" indicator on add** — `add_proxmox_cluster` inserted an
   *unauthenticated* client (no ticket) into the in-memory pool.
   `list_proxmox_clusters` reported `connected: true` just because the HashMap
   key existed, even though any API call would have failed.

3. **Double-unwrap of `data` in 10 commands** — `handle_response` already
   strips the `{"data": ...}` envelope before returning to callers, but
   `list_acls`, `list_users`, `get_cluster_notes`, `search_proxmox_resources`,
   `get_node_status`, `get_syslog`, `list_network_interfaces`,
   `get_subscription_status`, `list_cluster_tasks`, and
   `list_proxmox_containers` all called `.get("data")` on the already-unwrapped
   value, causing them to always return "Invalid response format".

4. **VM list always empty** — `list_vms` in `vm.rs` used `POST` on
   `cluster/resources` (a GET-only endpoint). The Proxmox API ignores the
   POST body and the function also had the same double-unwrap bug, meaning
   the resource list always came back empty. Additionally, VMs with no
   `cpu` field (e.g. stopped VMs) were silently dropped by `filter_map`
   using `?` — fixed to `unwrap_or(0.0)`.

5. **Double-unwrap in all other proxmox modules** — the same `.get("data")`
   double-unwrap was present across 18 module files (ceph, ceph_cluster,
   certificates, acme, firewall, sdn, ha, apt, updates, updates_ext, tasks,
   migration, metrics, shell, auth_realm, views, backup, vm). All 19 affected
   functions fixed in a single follow-up commit.

Additional gaps addressed:
- `CSRFPreventionToken` was never sent on POST/PUT/DELETE, so all mutating
  operations (VM start/stop, firewall rules, etc.) would fail with
  "CSRF check failed".
- Disconnect was UI-only with no backend call — the session stayed in the pool.
- `reqwest::Client` rejected self-signed Proxmox certificates.
- No `connect_proxmox_cluster` / `disconnect_proxmox_cluster` backend commands
  existed; the Connect/Disconnect buttons in the Remotes page were wired to a
  non-existent `ping_proxmox_cluster` or nothing at all.

## Acceptance Criteria

- [ ] Adding a new Proxmox host authenticates immediately; the UI shows
      "connected" only when a real ticket has been obtained.
- [ ] Closing the app and re-opening it allows reconnecting via the Connect
      button without requiring the host to be removed and re-added.
- [ ] Clicking Disconnect removes the session from the backend pool; subsequent
      API calls fail with "Cluster not found" until Connect is clicked.
- [ ] All existing Rust tests (426) and frontend tests (386) continue to pass.
- [ ] `cargo clippy -- -D warnings` and `npx eslint . --max-warnings 0` report
      zero issues.

## Work Implemented

| File | Change |
|------|--------|
| `src-tauri/src/proxmox/client.rs` | Added `ProxmoxEnvelope<T>` wrapper; fixed `authenticate()` to use `&mut self`, parse the envelope, and store both ticket and CSRF token; added `set_csrf_token()`; `build_headers()` now takes `include_csrf` flag; POST/PUT/DELETE pass `true`; `danger_accept_invalid_certs(true)` on the reqwest client |
| `src-tauri/src/commands/proxmox.rs` | `add_proxmox_cluster` authenticates before inserting into pool; fixed double-unwrap in 10 commands; added `connect_proxmox_cluster` and `disconnect_proxmox_cluster` Tauri commands |
| `src-tauri/src/lib.rs` | Registered `connect_proxmox_cluster` and `disconnect_proxmox_cluster` |
| `src-tauri/src/cli/mod.rs` | `let mut client` — `authenticate` is now `&mut self` |
| `src/lib/proxmoxClient.ts` | Added `connectProxmoxCluster` and `disconnectProxmoxCluster` wrappers |
| `src/pages/Proxmox/RemotesPage.tsx` | Added `handleConnectRemote` / `handleDisconnectRemote` handlers calling real backend commands; wired them to `RemotesList` `onConnect` / `onDisconnect` props |
| `src-tauri/src/proxmox/vm.rs` | `list_vms`: changed from `client.post("cluster/resources", body)` to `client.get("cluster/resources?type=vm")`; removed double-unwrap; cpu uses `unwrap_or(0.0)`. `get_vm`: removed double-unwrap. `list_snapshots`: removed double-unwrap. |
| `src-tauri/src/proxmox/{acme,apt,auth_realm,backup,ceph,ceph_cluster,certificates,firewall,ha,metrics,migration,sdn,shell,tasks,updates,updates_ext,views}.rs` | Removed `.get("data")` double-unwrap from all 19 functions across 17 files. |

New tests added:
- `client.rs` — envelope deserialization, no-CSRF path, `build_headers` GET omits / POST includes CSRF token, `set_ticket`/`set_csrf_token`
- `commands/proxmox.rs` — already-unwrapped array/object/notes response handling, `connect_proxmox_cluster` not-found error message

## Testing Needed

- [ ] Add a new Proxmox host with correct credentials → verify "connected" status
- [ ] Add a host with wrong password → verify immediate auth error, host not saved
- [ ] Restart the app → verify all previously-connected hosts show "disconnected"
- [ ] Click Connect on a disconnected host → verify it re-authenticates and shows "connected"
- [ ] Click Disconnect → verify status changes to "disconnected" and subsequent API calls fail
- [ ] Verify a VM start/stop operation succeeds (exercises CSRF token flow)
- [ ] Verify `get_node_status`, `list_acls`, `get_syslog` return real data (exercises double-unwrap fix)
- [ ] Verify a Proxmox host with a self-signed certificate connects successfully
