# TICKET: Kubernetes UI — FreeLens v5 Feature Parity

## Description

Full gap analysis and implementation plan to bring the TFTSR Kubernetes Management UI to
feature parity with Lens Desktop v5 / FreeLens (MIT-licensed, https://github.com/freelensapp/freelens).

Analysis confirmed the following areas require work:

1. **Navigation structure** does not match the requested layout — wrong grouping, missing top-level
   sections (Namespaces, Helm, Custom Resources), and missing items within existing sections.
2. **Resource actions** are incomplete across all resource types — pods, deployments, stateful sets,
   daemon sets, config maps, secrets, services, nodes, and all others are missing Edit, Delete, and
   resource-specific actions (Shell, Attach, Force Delete, Scale, Restart, etc.).
3. **Missing resource types** — 16+ resource types have no backend command, no list view, and no nav entry.
4. **Log streaming** is a static one-shot fetch; FreeLens streams with follow, timestamps, search, and download.
5. **Helm integration** is entirely absent — no Charts browser, no Releases management.
6. **Custom Resources / CRDs** are entirely absent.
7. **PR review workflow** was using stale model `qwen36-35b-a3b-nvfp4`; updated to `qwen3-coder-next`.
8. **`cargo fmt` CI failure** on `kube.rs` — fixed.

MIT-license compliance: FreeLens is MIT. All feature parity work is independent implementation using
`kubectl` CLI calls matching public Kubernetes API semantics. No FreeLens source is copied.

---

## Acceptance Criteria

### Navigation

- [ ] Nav matches the requested layout exactly:
  ```
  Cluster
  Nodes
  Workloads
    Overview
    Pods
    Deployments
    Daemon Sets
    Stateful Sets
    Replica Sets
    Replication Controllers
    Jobs
    Cron Jobs
  Config
    Config Maps
    Secrets
    Resource Quotas
    Limit Ranges
    Horizontal Pod Autoscalers
    Pod Disruption Budgets
    Priority Classes
    Runtime Classes
    Leases
    Mutating Webhook Configs
    Validating Webhook Configs
  Network
    Services
    Endpoint Slices
    Endpoints
    Ingresses
    Ingress Classes
    Network Policies
    Port Forwarding
  Storage
    Persistent Volume Claims
    Persistent Volumes
    Storage Classes
  Namespaces
  Events
  Helm
    Charts
    Resources
  Access Control
    Service Accounts
    Cluster Roles
    Roles
    Cluster Role Bindings
    Role Bindings
  Custom Resources
    Definitions
  ```

### Resource Actions (all resource types)

- [ ] **Pods**: Logs (streaming with follow/timestamps/search), Shell (exec -it, container selector),
      Attach, Edit (YAML), Delete (with confirmation), Force Delete (state-aware: only Running/Pending)
- [ ] **Deployments**: Scale, Rolling Restart, Rollback, Edit (YAML), Delete
- [ ] **StatefulSets**: Scale, Rolling Restart, Edit (YAML), Delete
- [ ] **DaemonSets**: Rolling Restart, Edit (YAML), Delete
- [ ] **ReplicaSets**: Scale, Edit (YAML), Delete
- [ ] **Replication Controllers**: Scale, Edit (YAML), Delete
- [ ] **Jobs**: Delete
- [ ] **CronJobs**: Suspend, Resume, Trigger Now, Edit (YAML), Delete
- [ ] **Services**: Edit (YAML), Delete, Port Forward shortcut
- [ ] **Ingresses**: Edit (YAML), Delete
- [ ] **ConfigMaps**: View data (key/value display), Edit (YAML), Delete
- [ ] **Secrets**: Reveal values (decode base64), Edit (YAML), Delete
- [ ] **HPAs**: Edit (YAML), Delete
- [ ] **PVCs**: Edit (YAML), Delete
- [ ] **PVs**: Edit (YAML), Delete
- [ ] **Storage Classes**: Edit (YAML), Delete
- [ ] **Resource Quotas**: Edit (YAML), Delete
- [ ] **Limit Ranges**: Edit (YAML), Delete
- [ ] **Nodes**: Cordon, Uncordon, Drain, Shell (exec), Describe
- [ ] **Service Accounts / Roles / ClusterRoles / Bindings**: Edit (YAML), Delete
- [ ] **Namespaces**: Create, Delete (with confirmation)
- [ ] **Network Policies**: Edit (YAML), Delete

### New Resource Types (backend + list view + nav)

- [ ] **Replication Controllers** (`kubectl get replicationcontrollers`)
- [ ] **Pod Disruption Budgets** (`kubectl get poddisruptionbudgets`)
- [ ] **Priority Classes** (`kubectl get priorityclasses`)
- [ ] **Runtime Classes** (`kubectl get runtimeclasses`)
- [ ] **Leases** (`kubectl get leases`)
- [ ] **Mutating Webhook Configurations** (`kubectl get mutatingwebhookconfigurations`)
- [ ] **Validating Webhook Configurations** (`kubectl get validatingwebhookconfigurations`)
- [ ] **Endpoints** (`kubectl get endpoints`)
- [ ] **Endpoint Slices** (`kubectl get endpointslices`)
- [ ] **Ingress Classes** (`kubectl get ingressclasses`)
- [ ] **Namespaces** (as a browsable list, not just a filter)
- [ ] **Helm Charts** (`helm search repo` / `helm repo` management)
- [ ] **Helm Releases** (`helm list` across namespaces, upgrade, rollback, uninstall)
- [ ] **CRD Definitions** (`kubectl get crds`)

### Functional Improvements

- [ ] Log streaming: follow mode, timestamps toggle, search/filter, download
- [ ] All destructive actions require a confirmation dialog showing resource name
- [ ] Force delete is only offered for pods in Running/Pending phase (state-aware context menu)
- [ ] Resource detail drawer: structured metadata, conditions, events, containers, YAML tab
- [ ] Edit Resource modal uses YAML editor with syntax highlighting and validation
- [ ] Shell/exec: auto-detects available shell (bash → ash → sh), container selector for multi-container pods
- [ ] Port Forwarding moved to Network section, "Open in Browser" button for HTTP ports

### CI / Workflow

- [ ] `cargo fmt` CI check passes
- [ ] PR review uses `qwen3-coder-next` model

---

## Work Implemented

### Phase 0 — Already done on this branch

| Item | Status |
|------|--------|
| `cargo fmt` failure on `kube.rs` | ✅ Fixed |
| PR review model → `qwen3-coder-next` | ✅ Updated |

### Phase 1 — Navigation Restructure

**Files**: `src/pages/Kubernetes/KubernetesPage.tsx`

- Reorder `NAV_SECTIONS` to match the requested layout exactly
- Add top-level sections: Namespaces, Events, Helm, Custom Resources
- Move Port Forwarding from Cluster → Network
- Move Overview from Cluster → Workloads
- Add missing `ActiveSection` union values
- Add routing for all new sections

### Phase 2 — Missing Resource Backends (Rust)

**File**: `src-tauri/src/commands/kube.rs`
**New Tauri commands** (all follow existing `list_*` pattern with `--output json`):

| Command | Resource |
|---------|----------|
| `list_replicationcontrollers` | Replication Controllers |
| `list_poddisruptionbudgets` | Pod Disruption Budgets |
| `list_priorityclasses` | Priority Classes |
| `list_runtimeclasses` | Runtime Classes |
| `list_leases` | Leases |
| `list_mutatingwebhookconfigurations` | Mutating Webhooks |
| `list_validatingwebhookconfigurations` | Validating Webhooks |
| `list_endpoints` | Endpoints |
| `list_endpointslices` | Endpoint Slices |
| `list_ingressclasses` | Ingress Classes |
| `attach_pod` | Pod attach (`kubectl attach -it`) |
| `force_delete_resource` | Force delete (`--grace-period=0 --force`) |
| `helm_list_repos` | Helm repo list |
| `helm_search_repo` | Helm chart search |
| `helm_list_releases` | Helm release list |
| `helm_upgrade` | Helm upgrade/install |
| `helm_rollback` | Helm rollback |
| `helm_uninstall` | Helm release delete |
| `list_crds` | CRD definitions |
| `list_custom_resources` | CRD instances by group/version/resource |
| `list_namespaces_resource` | Namespaces as a resource list (with status/age) |
| `create_namespace` | Create namespace |
| `delete_namespace` | Delete namespace |
| `get_resource_yaml` | Fetch any resource as YAML for editor |
| `describe_resource` | `kubectl describe` output |
| `stream_pod_logs` | Streaming logs (SSE or Tauri event channel) |
| `restart_statefulset` | `kubectl rollout restart sts/` |
| `restart_daemonset` | `kubectl rollout restart ds/` |
| `scale_statefulset` | `kubectl scale sts/` |
| `scale_replicaset` | `kubectl scale rs/` |
| `suspend_cronjob` | Patch CronJob spec.suspend=true |
| `resume_cronjob` | Patch CronJob spec.suspend=false |
| `trigger_cronjob` | `kubectl create job --from=cronjob/` |

### Phase 3 — Missing Resource List Components (React)

**Directory**: `src/components/Kubernetes/`
New components needed:

| Component | Notes |
|-----------|-------|
| `ReplicationControllerList.tsx` | |
| `PodDisruptionBudgetList.tsx` | |
| `PriorityClassList.tsx` | |
| `RuntimeClassList.tsx` | |
| `LeaseList.tsx` | |
| `MutatingWebhookList.tsx` | |
| `ValidatingWebhookList.tsx` | |
| `EndpointList.tsx` | |
| `EndpointSliceList.tsx` | |
| `IngressClassList.tsx` | |
| `NamespaceList.tsx` | With Create/Delete actions |
| `HelmChartList.tsx` | Charts browser |
| `HelmReleaseList.tsx` | Releases with Upgrade/Rollback/Uninstall |
| `CrdList.tsx` | CRD definitions |
| `WorkloadOverview.tsx` | Summary dashboard for Workloads section |

### Phase 4 — Resource Action Context Menus

**Pattern**: Each list component gets a `ResourceActionMenu` dropdown with state-aware items.

Common shared component: `ResourceActionMenu.tsx` accepting:
```ts
interface ResourceAction {
  label: string;
  icon: React.ElementType;
  onClick: () => void;
  variant?: "default" | "destructive";
  disabled?: boolean;
  hidden?: boolean;
}
```

Pod-specific: shell (with container selector), attach, logs, edit, delete, force delete (only shown
when pod.status ∈ {Running, Pending}).

All destructive actions (delete, force delete, drain, uninstall) open a `ConfirmDeleteDialog.tsx`
displaying the resource name before proceeding.

### Phase 5 — Log Streaming

Replace static `getPodLogsCmd` with streaming using Tauri event channel:
- Backend: `stream_pod_logs` spawns `kubectl logs --follow` and emits Tauri events per line
- Frontend: `LogStreamPanel.tsx` — virtual-scrolled, follow toggle, timestamps toggle, search, download

### Phase 6 — YAML Editor Integration

`EditResourceModal.tsx` exists. Wire it to all resource types via `get_resource_yaml` + `edit_resource`.
Add read-only YAML tab to all detail views.

---

## Testing Needed

- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` — all existing tests pass after new commands added
- [ ] Each new `list_*` Rust command has a unit test with mock JSON fixture
- [ ] `attach_pod` and `force_delete_resource` have unit tests validating command construction
- [ ] `npx tsc --noEmit` — zero TypeScript errors
- [ ] `npx eslint . --max-warnings 0` — zero lint warnings
- [ ] `cargo fmt --check` — clean
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] Manual: all 14+ new nav items render without errors against a live cluster
- [ ] Manual: Pod action menu shows all 6 actions; Force Delete hidden for Succeeded/Failed pods
- [ ] Manual: Delete confirmation dialog shows resource name and requires confirmation
- [ ] Manual: Log streaming follows new output in real time, search highlights matches
- [ ] Manual: YAML editor loads existing resource YAML and successfully applies edits
- [ ] Manual: Helm Charts list shows available charts; Releases list shows installed releases
- [ ] Manual: CRD list shows definitions; clicking a CRD shows its instances
- [ ] CI: `cargo fmt --check` passes (was failing before this branch)
- [ ] CI: PR review workflow uses `qwen3-coder-next` model
