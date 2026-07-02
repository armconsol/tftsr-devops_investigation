# Ticket: K8s logs dock, pty stdin, Proxmox Ceph parsing

**Branch:** `fix/kube-logs-pty-proxmox-ceph` (off `beta`)
**PR:** [#142](https://gogs.tftsr.com/sarman/tftsr-devops_investigation/pulls/142) → `beta`

## Description
Four defects in the TFTSR DevOps investigation app were resolved on a single branch off `beta`, using TDD:

1. **K8s pod logs opened in a small centered popup**, making logs hard to read. They now open in the freelens-style bottom dock (full width, resizable lower half) with live streaming.
2. **Shelling/attaching to a K8s pod was broken** — every keystroke failed with `invalid args 'data' for command 'send_pty_stdin': invalid type: string "l", expected a sequence`. The backend expects `Vec<u8>`; the frontend sent a JS string.
3. **K8s Deployments (and other workloads) logs opened in a popup and lacked tail/stream** — no live output. They now use the streaming bottom dock with a pod picker, follow and tail.
4. **Proxmox › Ceph threw "Failed to load Ceph OSDs"** (and the same for Monitors, Managers, CephFS, Flags) because the Rust parsers expected the wrong PVE API response shapes and the structs did not match the frontend types.

## Acceptance Criteria
- [x] Pod logs open in the bottom dock (not a modal) with follow/tail/streaming.
- [x] Workload logs (Deployment, StatefulSet, DaemonSet, Job, CronJob, ReplicaSet, ReplicationController) open in the dock with a pod selector defaulting to the first matching pod, with streaming + tail.
- [x] Shelling/attaching to a pod accepts keystrokes without serde errors.
- [x] Proxmox Ceph page loads OSDs, Monitors, Managers, CephFS and Flags without errors.
- [x] All Rust and frontend tests pass 100%; `cargo fmt`, `cargo clippy -D warnings`, `tsc` and `eslint` are clean.

## Work Implemented
- **Issue #2 (pty stdin):** `sendPtyStdinCmd` now encodes `data` as `Array.from(new TextEncoder().encode(data))`. Added `tests/unit/sendPtyStdin.test.ts`. (commit `e6a3269d`)
- **Issue #4 (Ceph):** Rewrote `src-tauri/src/proxmox/ceph.rs` with pure, fixture-tested parsers — `parse_osds` (CRUSH-tree walk), `parse_monitors` (numeric quorum), `parse_managers`, `parse_cephfs`, `parse_pools`; `get_ceph_flags` now targets the cluster-scoped `cluster/ceph/flags`. Aligned structs to camelCase, updated `src/lib/proxmoxClient.ts`, `src/pages/Proxmox/CephPage.tsx`, and `docs/wiki/IPC-Commands.md`. Added `tests/unit/proxmoxCeph.test.ts`. (commit `4d648646`)
- **Issues #1 & #3 (logs dock):** `src/components/dock/LogsTab.tsx` gained an optional workload mode (name-prefix pod resolution + pod picker) alongside single-pod mode, stopping the active stream when switching pods. New `src/lib/logsDock.ts` helpers (`openPodLogsTab`, `openWorkloadLogsTab`) open a de-duplicated `POD_LOGS` dock tab. `PodList` and all 7 workload lists route to the dock. Removed the superseded non-streaming `WorkloadLogsModal`. Added `tests/unit/logsDock.test.ts`; updated `PodList` and `criticalUIFixes` tests. (commit `14e2188f`)

## Testing Needed
- **Manual:** Open pod logs and workload logs from the K8s pages — confirm dock placement, streaming, follow, tail and pod selection.
- **Manual:** Shell/attach into a pod and type — confirm input works.
- **Manual:** Open Proxmox › Ceph — confirm OSDs/Monitors/Managers/CephFS/Flags render.
- **Automated (all green):**
  - Rust: `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1` → 641 passed
  - Frontend: `npm run test:run` → 451 passed
  - `cargo fmt --check`, `cargo clippy -- -D warnings`, `npx tsc --noEmit`, `npx eslint src/ tests/ --quiet` → clean
