# Ticket Summary — Proxmox Ceph Endpoint Parity

## Description

Resolve post-push review findings by enforcing node-scoped Ceph API paths in the backend Ceph module, validating whether stale warnings were real defects, and refreshing README/wiki coverage for current Proxmox behavior.

## Acceptance Criteria

- Ceph operations in `src-tauri/src/proxmox/ceph.rs` use node-scoped endpoint paths (`nodes/{node}/ceph/*`) where applicable.
- No missing `validate_node` usage/import issue remains in Ceph module.
- VM list migration path uses correct `listProxmoxNodes(clusterId)` call shape (single argument).
- README and wiki documents reflect current Proxmox/Ceph behavior and endpoint policy.

## Work Implemented

- Updated Ceph backend operations from cluster-scoped to node-scoped paths in `src-tauri/src/proxmox/ceph.rs`, including:
  - pool create/delete/quota
  - OSD weight/in/out
  - MDS list/status/failover
  - RBD list/create/delete/clone/resize/snapshot
  - monitor status and quorum health retrieval
- Added/maintained node validation (`validate_node`) on node-parameterized Ceph operations.
- Added a test asserting write-operation path construction is node-scoped.
- Updated documentation:
  - `README.md` with Proxmox Management highlights and Ceph node-scoped endpoint note.
  - `docs/wiki/IPC-Commands.md` Ceph section expanded with `list_ceph_pools`, `list_ceph_osd`, `get_ceph_health`, corrected CephFS endpoint to `nodes/{node}/ceph/fs`, and explicit endpoint policy.
  - `docs/wiki/Architecture.md` module table now includes `proxmox/ceph.rs` and node-scoped behavior.
- Per review validation:
  - `validate_node` warning was stale (function is locally defined and used).
  - `VMList.tsx` already uses a single `listProxmoxNodes(clusterId)` argument.

## Testing Needed

- Re-run repository quality gates in CI:
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1`
  - `npx tsc --noEmit`
  - `npx eslint src/ tests/ --quiet`
- Smoke-test Proxmox Ceph UI page against a live cluster:
  - Pools/OSDs/Health/Monitors/Managers/CephFS/Flags load for selected node.
  - Confirm no regressions in existing Ceph read flows.
