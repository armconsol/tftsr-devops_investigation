# Ticket Summary — Kubernetes Action Namespace & Stability Fixes

**Branch**: `fix/kube-action-namespace-and-stability`
**PR**: https://gogs.tftsr.com/sarman/tftsr-devops_investigation/pulls/86

---

## Description

Seven bugs in the Kubernetes management interface were identified via systematic debugging and resolved across 6 commits.

The most severe was a **temp kubeconfig race condition** in the Rust backend: every kubectl-based IPC command wrote a temp file to a static path derived only from `cluster_id`. Concurrent calls — triggered by rapid section or namespace switching — shared identical paths. `TempFileCleanup::drop()` on the first-to-finish call deleted the file while a concurrent kubectl process was still reading it. Errors were silently swallowed, leaving the UI showing stale/empty data. This was the root cause of "things stop loading after a few selection changes."

The second major class of bugs was **namespace `"all"` passed to targeted kubectl commands**. When the user selects "All Namespaces", `KubernetesPage` stores `selectedNamespace = "all"` and passes it as a prop to all list components. `loadResourceData` correctly converts `"all" → ""` for list fetching (which becomes `--all-namespaces` in Rust). However, action handlers inside list components (edit, delete, scale, logs, shell, attach) used the raw prop and forwarded `"all"` to `kubectl -n all`, producing "namespaces 'all' not found" errors.

---

## Acceptance Criteria

- [x] Rapid section/namespace switching no longer causes data to stop loading
- [x] Pod Logs loads successfully when "All Namespaces" is selected
- [x] Pod Shell, Attach, and Edit open and target the pod's actual namespace
- [x] Deployment, StatefulSet, DaemonSet, and all other workload action commands work under "All Namespaces"
- [x] Network, Config, Storage, and Access Control action commands work under "All Namespaces"
- [x] Workloads → Overview shows actual resource counts (not all-zero)
- [x] Cluster connection errors display a visible banner instead of failing silently
- [x] `connectClusterFromKubeconfigCmd` is only called once on mount, not twice
- [x] Dark mode — all text is readable; status indicators are visible

---

## Work Implemented

### Commit 1 — `fix(kube): unique temp kubeconfig paths`
**File**: `src-tauri/src/commands/kube.rs`

Added `KUBECONFIG_COUNTER: AtomicU64` and `unique_kubeconfig_path(cluster_id)` helper. Replaced all 74 static `temp_dir.join(format!("kubeconfig-{}-*.yaml"))` calls with the helper. Each invocation now gets a globally unique path, eliminating the race.

### Commit 2 — `fix(ui): replace hardcoded colors with semantic Tailwind vars`
**Files**: `src/components/Kubernetes/PortForwardList.tsx`, `src/components/Kubernetes/WorkloadOverview.tsx`

Replaced non-adaptive `text-gray-*` / `bg-gray-*` classes with `text-muted-foreground`, `bg-muted`, `border-border` — Tailwind CSS vars that correctly invert in dark mode.

### Commit 3 — `fix(kube): WorkloadOverview loads data; single connect; visible error`
**Files**: `src/pages/Kubernetes/KubernetesPage.tsx`, `tests/unit/KubernetesPage.test.tsx`

- Added `case "workloads_overview"` in `loadResourceData` that fetches pods + deployments + statefulsets + daemonsets + jobs + cronjobs via `Promise.allSettled` in parallel.
- Added `initializedRef` guard in `loadInitialData` to prevent double-connect when `selectedClusterId` changes.
- Connection errors now captured and shown as a dismissible banner.

### Commit 4 — `fix(kube): add namespace to PodInfo; pod actions use pod.namespace`
**Files**: `src-tauri/src/commands/kube.rs`, `src/lib/tauriCommands.ts`, `src/components/Kubernetes/PodList.tsx`, `tests/unit/PodList.test.tsx`

Added `namespace: String` to `PodInfo` Rust struct, extracted from `metadata.namespace` in `parse_pods_json`. Added `namespace: string` to TypeScript `PodInfo` interface. Updated all 6 action call sites in `PodList` to use `pod.namespace`.

### Commit 5 — `fix(kube): network/config/storage list actions use item.namespace`
**Files**: `ServiceList`, `IngressList`, `ConfigMapList`, `SecretList`, `HPAList`, `PVCList`, `ServiceAccountList`, `RoleList`, `RoleBindingList`, `NetworkPolicyList`, `ResourceQuotaList`, `LimitRangeList` + `tests/unit/NamespaceActionFix.test.tsx`

12 components fixed. 24 new tests (2 per component).

### Commit 6 — `fix(kube): workload list actions use item.namespace not filter prop`
**Files**: `DeploymentList`, `StatefulSetList`, `DaemonSetList`, `ReplicaSetList`, `JobList`, `CronJobList` + `tests/unit/WorkloadListActions.test.tsx`

6 components fixed. 21 new tests.

---

## Testing Needed

1. **Automated**: `cargo test` → 364 pass; `npm run test:run` → 325 pass; `npx tsc --noEmit` → 0; `npx eslint . --max-warnings 0` → 0; `cargo clippy -- -D warnings` → 0; `cargo fmt --check` → clean
2. **Manual — race condition**: With a live cluster, rapidly switch between Pods → Deployments → Services → ConfigMaps several times. Data should load reliably every time.
3. **Manual — pod actions**: Select "All Namespaces". Open pod action menu → Logs → should fetch without error. Shell/Attach → modals open, exec targets correct namespace. Edit → YAML editor opens.
4. **Manual — overview**: Navigate to Workloads → Overview. Cards should show actual pod/deployment/etc. counts.
5. **Manual — error banner**: Configure an invalid kubeconfig. Navigate to Kubernetes page. A red banner should appear with the connection error. Clicking Dismiss hides it.
6. **Manual — dark mode**: Switch to dark theme. All text in Kubernetes pages (sidebar, tables, status indicators) should be readable with good contrast.
