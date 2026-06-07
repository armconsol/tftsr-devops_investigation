# Kubernetes UI PR Review Fixes

## Description

Resolved all findings from the automated PR review (qwen3-coder-next) of the Kubernetes resource discovery and management feature. The review identified two blockers and several warnings across Rust backend and React frontend.

**Root cause of blockers:** All six JSON parsing functions in `kube.rs` imported and used `serde_yaml::Value` / `serde_yaml::from_str` against kubectl's JSON output (`-o json`), causing parse failures or incorrect data at runtime. YAML is a superset of JSON and sometimes parses silently incorrectly; the correct parser is `serde_json`.

**Secondary issues:** `PodInfo` lacked container name data, so the log viewer could only show the pod name as the container selector. The `exec_pod` command had an incorrect kubectl argument order (container `-c` flag placed after `--`, so it was passed to the shell inside the pod rather than to kubectl). The "All Namespaces" filter passed an empty string to kubectl `-n ""` which is invalid.

---

## Acceptance Criteria

- [x] All six `parse_*_json` functions use `serde_json::from_str` and `serde_json::Value` API (`as_array`, `as_object`)
- [x] `PodInfo` struct carries `containers: Vec<String>`; container names parsed from `spec.containers[*].name`
- [x] `PodList.tsx` container selector populates from `selectedPod.containers`
- [x] `exec_pod` container `-c` flag is placed before `--` separator (correct kubectl syntax)
- [x] `exec_pod` accepts optional `shell` parameter with allowlist validation (`sh`, `bash`, `ash`, `dash`)
- [x] Empty namespace string routes to `--all-namespaces` in all five list commands
- [x] Dialog inner div uses `overflow-y-auto` to handle content overflow on small screens
- [x] `getNamespaceOptions` memoized with `useMemo`
- [x] `eslint.config.js` deduplicated (was 272 lines, duplicate blocks removed), global ignore fixed
- [x] Unused imports removed from all Kubernetes list components
- [x] `cargo clippy -- -D warnings`: zero warnings
- [x] `tsc --noEmit`: zero errors
- [x] `eslint . --max-warnings 0`: zero warnings
- [x] 331 Rust tests passing, 98 frontend tests passing

---

## Work Implemented

### `src-tauri/src/commands/kube.rs`
- Replaced `use serde_yaml::Value` with `use serde_json::Value`
- `extract_context` and `extract_server_url`: explicitly typed as `serde_yaml::Value` (these legitimately parse YAML kubeconfig files)
- `PodInfo` struct: added `containers: Vec<String>` field
- `parse_pods_json`: switched to `serde_json::from_str`, `as_array()`; added container name extraction from `spec.containers[].name`
- `parse_namespaces_json`, `parse_services_json`, `parse_deployments_json`, `parse_statefulsets_json`, `parse_daemonsets_json`: switched to `serde_json::from_str`, `as_array()`, `as_object()`; updated mapping iterators (serde_json object keys are `String`, not `Value`)
- `parse_services_json`: fixed `.as_sequence()` → `.as_array()` in `external_ip` ingress chain
- `list_pods`, `list_services`, `list_deployments`, `list_statefulsets`, `list_daemonsets`: handle empty `namespace` with `--all-namespaces`
- `exec_pod`: added optional `shell: Option<String>` parameter; allowlist validates against `["sh","bash","ash","dash","/bin/sh","/bin/bash","/bin/ash","/bin/dash"]`; fixed argument order so `-c container` appears before `--`
- Phase 3 stub commands: added `#[allow(unused_variables)]` to suppress Clippy warnings on unimplemented stubs

### `src/lib/tauriCommands.ts`
- `PodInfo` interface: added `containers: string[]`
- `execPodCmd`: added optional `shell?: string` parameter, passed through to IPC

### `src/components/Kubernetes/PodList.tsx`
- Fixed: `const containers = selectedPod ? [selectedPod.name] : []` → `selectedPod?.containers ?? []`
- Fixed: `overflow-hidden` → `overflow-y-auto` on inner dialog content div
- Removed unused imports: `Card`, `CardContent`, `CardHeader`, `CardTitle`

### `src/components/Kubernetes/ResourceBrowser.tsx`
- Added `useCallback` import; wrapped `loadData` in `useCallback([clusterId, selectedNamespace])`
- `useEffect` deps updated to `[loadData, resourceType]`
- Removed unused `CardTitle` import
- `getNamespaceOptions` converted to memoized `namespaceOptions` via `useMemo`

### `src/components/Kubernetes/DaemonSetList.tsx`, `ServiceList.tsx`, `StatefulSetList.tsx`
- Removed unused `Card`, `CardContent`, `CardHeader`, `CardTitle` imports
- Renamed unused props: `clusterId: _clusterId`, `namespace: _namespace`

### `src/components/Kubernetes/DeploymentList.tsx`
- Removed unused `Card`, `CardContent`, `CardHeader`, `CardTitle` imports

### `src/components/ui/index.tsx`
- `TableRow`: renamed unused `hover` prop to `_hover`

### `src/App.tsx`
- Removed two debug `console.log` calls (auto-testing provider connection)

### `src/pages/Triage/index.tsx`
- `useEffect`: added `addMessage`, `setActiveDomain`, `startSession` to dependency array (stable Zustand store actions)

### `src/pages/LogUpload/index.tsx`
- `handleImagesUpload`: wrapped in `useCallback([id])` and moved before `handleImageDrop` to resolve declaration-order issue
- `handleImageDrop`: updated deps from `[id]` to `[handleImagesUpload]`

### `eslint.config.js`
- Removed duplicate config block (file was doubled to 272 lines)
- Fixed global ignore: moved `ignores` array to a standalone config object (was incorrectly paired with `files`)
- CLI section: added `"log"` to allowed console methods (CLI tool output)

### `.eslintignore`
- Deleted — content migrated to `eslint.config.js` global ignore

---

## Testing Needed

- [ ] Connect a real kubeconfig and verify pod/namespace/service/deployment/statefulset/daemonset lists render correctly with JSON from kubectl
- [ ] Select "All Namespaces" — verify `--all-namespaces` is used and resources from all namespaces appear
- [ ] Open pod log dialog — verify container dropdown shows actual container names (not pod name)
- [ ] Fetch logs for a multi-container pod — verify correct container logs are returned
- [ ] Test `exec_pod` via UI with `sh` (default) and `bash` — verify both work
- [ ] Test `exec_pod` with an invalid shell name (e.g., `zsh`) — verify it returns an error
- [ ] Verify "All Namespaces" view does not trigger empty-namespace kubectl error
- [ ] Smoke test triage and log upload flows to verify `useEffect`/`useCallback` hook changes have no regressions
