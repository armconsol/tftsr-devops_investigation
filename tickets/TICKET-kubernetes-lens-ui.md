# Ticket: Kubernetes Management UI — Lens Desktop v5 Feature Parity

**Branch**: `feature/kubernetes-management-v2`
**PR**: See PR created against `master`
**Date**: 2026-06-07

---

## Description

The Kubernetes page previously showed only a cluster configuration list and port forwarding panel — a fraction of the intended feature set. This ticket implements full Lens Desktop v5-equivalent Kubernetes management UI directly inside the application.

The backend already had 44 Tauri commands and 40+ frontend components built but not properly orchestrated. The core problem was `KubernetesPage.tsx` acting as a simple config page rather than as a Lens-style IDE shell. This work:

1. Rewrites the page as a proper Lens-like shell (collapsible sidebar nav + hotbar + main content panel)
2. Surfaces all 26 resource types through organized navigation
3. Replaces all stub components with real implementations backed by IPC
4. Adds 4 missing resource types (StorageClasses, NetworkPolicies, ResourceQuotas, LimitRanges) with Rust backend + frontend
5. Installs and integrates missing libraries (xterm.js, Monaco editor, recharts)
6. Fixes the pre-existing ESLint 10 incompatibility (`eslint-plugin-react` → `@eslint-react/eslint-plugin`)
7. Achieves 251 passing tests (up from 94) with full TDD methodology

---

## Acceptance Criteria

- [x] Kubernetes page renders a Lens-style layout: collapsible sidebar with 5 navigation categories, top hotbar, namespace selector, cluster context switcher
- [x] All 26 resource types are accessible via sidebar navigation (previously only 5)
- [x] `Ctrl+K` opens Command Palette with navigation commands
- [x] ClusterOverview shows real-time node/pod/deployment/namespace counts from the cluster
- [x] Terminal component uses xterm.js with real `exec_pod` IPC integration and multi-tab support
- [x] YAML editor uses Monaco with syntax highlighting and apply/cancel functionality
- [x] Create/Edit resource modals call `createResourceCmd`/`editResourceCmd` IPC
- [x] RBAC Viewer loads live data; RBAC Editor creates roles via `createResourceCmd`
- [x] Detail panels (Pod, Deployment, Service, ConfigMap, Secret) show real data from IPC — zero hardcoded values
- [x] MetricsChart uses recharts with proper data transformation
- [x] StorageClasses, NetworkPolicies, ResourceQuotas, LimitRanges: Rust commands + TypeScript wrappers + list components
- [x] ESLint passes with zero errors/warnings across entire `src/` directory
- [x] `npx tsc --noEmit` passes with zero errors
- [x] `cargo clippy -- -D warnings` passes with zero warnings
- [x] `cargo fmt --check` passes
- [x] All 251 tests pass

---

## Work Implemented

### New/Rewritten Frontend Files
| File | Change |
|------|--------|
| `src/pages/Kubernetes/KubernetesPage.tsx` | Full rewrite: Lens-like sidebar layout, hotbar, namespace selector, command palette, all 26 resource types |
| `src/components/Kubernetes/Terminal.tsx` | Rewrite: real xterm.js, multi-tab, exec_pod IPC |
| `src/components/Kubernetes/YamlEditor.tsx` | Rewrite: Monaco editor with apply/cancel |
| `src/components/Kubernetes/MetricsChart.tsx` | Rewrite: recharts LineChart/BarChart |
| `src/components/Kubernetes/ClusterOverview.tsx` | Rewrite: real IPC data (nodes, pods, deployments, namespaces) |
| `src/components/Kubernetes/ClusterDetails.tsx` | Rewrite: real kubeconfig + node data |
| `src/components/Kubernetes/PodDetail.tsx` | Rewrite: real logs, real pod metadata, real containers |
| `src/components/Kubernetes/DeploymentDetail.tsx` | Rewrite: real replicas, scale/restart/rollback actions |
| `src/components/Kubernetes/ServiceDetail.tsx` | Rewrite: real service data, port table |
| `src/components/Kubernetes/ConfigMapDetail.tsx` | Rewrite: real configmap data |
| `src/components/Kubernetes/SecretDetail.tsx` | Rewrite: real secret key listing |
| `src/components/Kubernetes/CreateResourceModal.tsx` | Wired: calls `createResourceCmd` |
| `src/components/Kubernetes/EditResourceModal.tsx` | Wired: calls `editResourceCmd` |
| `src/components/Kubernetes/CommandPalette.tsx` | Wired: 12 real navigation commands |
| `src/components/Kubernetes/RbacViewer.tsx` | Rewrite: live RBAC data from 4 IPC commands |
| `src/components/Kubernetes/RbacEditor.tsx` | Rewrite: real create via `createResourceCmd` |
| `src/components/Kubernetes/StorageClassList.tsx` | New component |
| `src/components/Kubernetes/NetworkPolicyList.tsx` | New component |
| `src/components/Kubernetes/ResourceQuotaList.tsx` | New component |
| `src/components/Kubernetes/LimitRangeList.tsx` | New component |
| `src/components/Kubernetes/index.tsx` | Exports for 4 new components |
| `src/lib/eventBus.ts` | Fixed: `any` → `unknown` types |
| `src/pages/Settings/Security.tsx` | Fixed: function hoisting lint issue |

### New Backend (Rust)
| File | Change |
|------|--------|
| `src-tauri/src/commands/kube.rs` | +4 structs, +4 commands: `list_storageclasses`, `list_networkpolicies`, `list_resourcequotas`, `list_limitranges` |
| `src-tauri/src/lib.rs` | +4 entries in `generate_handler![]` |

### TypeScript IPC
| File | Change |
|------|--------|
| `src/lib/tauriCommands.ts` | +4 interfaces, +4 command wrappers for new resource types |

### Tooling
| File | Change |
|------|--------|
| `eslint.config.js` | Replaced incompatible `eslint-plugin-react` with `@eslint-react/eslint-plugin` (ESLint 10 compatible) |
| `package.json` / `package-lock.json` | Added: `xterm`, `xterm-addon-fit`, `xterm-addon-web-links`, `@monaco-editor/react`, `recharts`, `@eslint-react/eslint-plugin` |

### Tests (35 test files, 251 tests — up from 19 files, 94 tests)
New test files:
- `tests/unit/KubernetesPage.test.tsx` — 22 tests
- `tests/unit/Terminal.test.tsx` — 15 tests
- `tests/unit/YamlEditor.test.tsx` — 8 tests
- `tests/unit/CreateResourceModal.test.tsx` — 6 tests
- `tests/unit/EditResourceModal.test.tsx` — 4 tests
- `tests/unit/MetricsChart.test.tsx` — 7 tests
- `tests/unit/ClusterOverview.test.tsx` — 6 tests
- `tests/unit/ClusterDetails.test.tsx` — 5 tests
- `tests/unit/PodDetail.test.tsx` — 7 tests
- `tests/unit/DeploymentDetail.test.tsx` — 6 tests
- `tests/unit/ConfigMapDetail.test.tsx` — 4 tests
- `tests/unit/SecretDetail.test.tsx` — 4 tests
- `tests/unit/RbacViewer.test.tsx` — 9 tests
- `tests/unit/CommandPalette.test.tsx` — 12 tests
- `tests/unit/NewResourceTypes.test.tsx` — 21 tests

### Wiki
- `docs/wiki/Kubernetes-Management.md` — Full rewrite covering all features, layout, backend architecture, dependencies, known limitations

---

## Testing Needed

- [ ] **Manual: Cluster load** — Upload a kubeconfig, activate it, verify sidebar auto-populates namespace dropdown
- [ ] **Manual: Resource browsing** — Navigate to each sidebar section, verify list renders from live cluster
- [ ] **Manual: Pod logs** — Click a pod → Logs tab → verify container dropdown and real log output
- [ ] **Manual: Deployment scale** — Navigate to Deployments → click deployment → Actions tab → scale to N replicas
- [ ] **Manual: Deployment rollback** — Rollback a deployment, verify `kubectl rollout undo` executes
- [ ] **Manual: Terminal** — Exec into a pod, run `ls`, verify output appears in xterm.js
- [ ] **Manual: YAML create** — Create a ConfigMap via YAML editor, verify it appears in the list
- [ ] **Manual: RBAC** — Navigate to Access Control → Roles, verify live data from cluster
- [ ] **Manual: Port forward** — Navigate to Cluster → Port Forwarding, start a forward, verify tunnel is active
- [ ] **Manual: Command Palette** — Press Ctrl+K, type "pod", press Enter, verify navigation to Pods section
- [ ] **Manual: Node drain** — Navigate to Nodes, drain a non-critical node, verify cordon+eviction
- [ ] **Manual: StorageClasses** — Navigate to Config → Storage Classes, verify provisioner column populated
- [ ] **Automated**: `npm run test:run` — 251/251 must pass
- [ ] **Automated**: `npx tsc --noEmit` — zero errors
- [ ] **Automated**: `npx eslint src/ --max-warnings 0` — zero issues
- [ ] **Automated**: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` — zero warnings
