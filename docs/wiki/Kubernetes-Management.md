# Kubernetes Management

This document describes the Kubernetes Management UI — a Lens Desktop v5-equivalent Kubernetes management experience built into the Troubleshooting and RCA Assistant.

---

## Overview

The Kubernetes Management UI provides full feature parity with Lens Desktop v5.x (the last open-source release), delivering a complete cluster management IDE directly inside the application. The implementation is MIT-licensed and uses the bundled `kubectl` binary for all cluster operations.

**Current version: v1.1.0**

---

## Page Layout

The Kubernetes page uses a Lens-style shell layout:

```
┌──────────────────────────────────────────────────────────────┐
│  Hotbar: Cluster selector | Namespace selector | Refresh | + │
├──────────────┬───────────────────────────────────────────────┤
│   SIDEBAR    │              MAIN CONTENT                     │
│              │                                               │
│ ▶ WORKLOADS  │  ClusterOverview (default)                    │
│   Pods       │  — or —                                       │
│   Deployments│  Selected resource list                       │
│   DaemonSets │  — or —                                       │
│   StatefulSets│  Detail panel                                │
│   ReplicaSets │                                              │
│   Jobs       │                                               │
│   CronJobs   │                                               │
│              │                                               │
│ ▶ NETWORKING │                                               │
│   Services   │                                               │
│   Ingresses  │                                               │
│   NetworkPols│                                               │
│              │                                               │
│ ▶ CONFIG     │                                               │
│   ConfigMaps │                                               │
│   Secrets    │                                               │
│   HPAs       │                                               │
│   PVCs       │                                               │
│   PVs        │                                               │
│   StorageClass│                                              │
│   ResourceQ  │                                               │
│   LimitRanges│                                               │
│              │                                               │
│ ▶ ACCESS CTL │                                               │
│   ServiceAccts│                                              │
│   Roles      │                                               │
│   ClusterRoles│                                              │
│   RoleBindings│                                              │
│   CRBindings │                                               │
│              │                                               │
│ ▶ CLUSTER    │                                               │
│   Overview   │                                               │
│   Nodes      │                                               │
│   Events     │                                               │
│   Port Fwd   │                                               │
└──────────────┴───────────────────────────────────────────────┘
```

**Keyboard shortcut**: `Ctrl+K` opens the Command Palette for quick navigation.

---

## Resource Types (26 total)

### Workloads (7)
| Resource | Component | Actions |
|----------|-----------|---------|
| Pods | `PodList` + `PodDetail` | Logs, exec, scale, delete |
| Deployments | `DeploymentList` + `DeploymentDetail` | Scale, restart, rollback, delete |
| Daemon Sets | `DaemonSetList` | Delete |
| Stateful Sets | `StatefulSetList` | Delete |
| Replica Sets | `ReplicaSetList` | Delete |
| Jobs | `JobList` | Delete |
| Cron Jobs | `CronJobList` | Delete |

### Services & Networking (3)
| Resource | Component | Actions |
|----------|-----------|---------|
| Services | `ServiceList` + `ServiceDetail` | Port forward, delete |
| Ingresses | `IngressList` | Delete |
| Network Policies | `NetworkPolicyList` | Delete |

### Config & Storage (8)
| Resource | Component | Actions |
|----------|-----------|---------|
| Config Maps | `ConfigMapList` + `ConfigMapDetail` | Edit, delete |
| Secrets | `SecretList` + `SecretDetail` | View masked, delete |
| Horizontal Pod Autoscalers | `HPAList` | Delete |
| Persistent Volume Claims | `PVCList` | Delete |
| Persistent Volumes | `PVList` | Delete |
| Storage Classes | `StorageClassList` | Delete |
| Resource Quotas | `ResourceQuotaList` | Delete |
| Limit Ranges | `LimitRangeList` | Delete |

### Access Control (5)
| Resource | Component | Actions |
|----------|-----------|---------|
| Service Accounts | `ServiceAccountList` | Delete |
| Roles | `RoleList` + `RbacViewer`/`RbacEditor` | Create, delete |
| Cluster Roles | `ClusterRoleList` + `RbacViewer`/`RbacEditor` | Create, delete |
| Role Bindings | `RoleBindingList` | Delete |
| Cluster Role Bindings | `ClusterRoleBindingList` | Delete |

### Cluster (4)
| Resource | Component | Notes |
|----------|-----------|-------|
| Overview | `ClusterOverview` | Live node/pod/deployment counts |
| Nodes | `NodeList` | Cordon, uncordon, drain |
| Events | `EventList` | Filterable by namespace |
| Port Forwarding | `PortForwardList` + `PortForwardForm` | Start/stop/delete tunnels |

---

## Advanced Features

### Terminal (`Terminal.tsx`)
- Full xterm.js implementation with multi-tab session management
- Shell selection: `sh`, `bash`, `zsh`
- Connects to pods via `exec_pod` IPC command
- `xterm-addon-fit` for automatic resize
- `xterm-addon-web-links` for clickable URLs in output
- Sessions identified by `pod/container/namespace`

### YAML Editor (`YamlEditor.tsx`)
- Monaco editor (`@monaco-editor/react`) with YAML syntax highlighting
- Language: `yaml`, Theme: `vs-dark`
- Controlled value with Apply/Cancel buttons
- Used in: `CreateResourceModal`, `EditResourceModal`, detail panels, `RbacEditor`

### Metrics Charts (`MetricsChart.tsx`)
- recharts `LineChart` and `BarChart` with `ResponsiveContainer`
- Time range selector: 5m, 15m, 1h, 6h, 1d
- Used in: `ApplicationView`, `ClusterOverview`

### Command Palette (`CommandPalette.tsx`)
- Triggered with `Ctrl+K` from anywhere in the Kubernetes page
- 12 navigation commands covering all major resource types
- Keyboard navigation: ↑/↓ arrows, Enter to execute, Escape to close
- Filter commands by typing

### RBAC Management (`RbacViewer.tsx` / `RbacEditor.tsx`)
- Viewer: live data from `listRolesCmd`, `listClusterrolesCmd`, `listRolebindingsCmd`, `listClusterrolebindingsCmd`
- Editor: YAML editor with template generation for Roles, ClusterRoles, RoleBindings, ClusterRoleBindings
- Create via `createResourceCmd`, delete via `deleteResourceCmd`

### Cluster Overview (`ClusterOverview.tsx`)
- Real-time counts: nodes (ready/total), pods (running/total), deployments, namespaces
- Node table with status, roles, version, age
- All data loaded from `listNodesCmd`, `listPodsCmd`, `listDeploymentsCmd`, `listNamespacesCmd`

---

## Backend Architecture

All Kubernetes operations use the bundled `kubectl` binary (v1.30.0) via `tokio::process::Command`. No direct Kubernetes API client library is used — this approach avoids TLS certificate management complexity and works with any cluster configuration.

### State

```rust
pub struct AppState {
    pub clusters: Arc<TokioMutex<HashMap<String, ClusterClient>>>,
    pub port_forwards: Arc<TokioMutex<HashMap<String, PortForwardSession>>>,
    pub watchers: Arc<Mutex<HashMap<String, WatcherHandle>>>,
    // ...
}
```

Clusters are stored in-memory only (not persisted). Kubeconfigs are stored encrypted in the database and written to temporary files at command execution time.

### Security

- **Input validation**: `validate_resource_name()` enforces Kubernetes DNS subdomain rules and prevents command injection
- **Temp file cleanup**: `TempFileCleanup` guard auto-deletes kubeconfig temp files on scope exit
- **No credential logging**: kubeconfig content never appears in audit logs
- **Three-tier command safety**: shell commands additionally classified by `classifier.rs` (Tier 1 auto, Tier 2 approval, Tier 3 deny)

### Commands (48 total)

#### Cluster Management (5)
- `add_cluster`, `remove_cluster`, `list_clusters`, `test_cluster_connection`, `discover_pods`

#### Port Forwarding (5)
- `start_port_forward`, `stop_port_forward`, `list_port_forwards`, `delete_port_forward`, `shutdown_port_forwards`

#### Resource Discovery (26)
- `list_namespaces`, `list_pods`, `list_services`, `list_deployments`, `list_statefulsets`, `list_daemonsets`
- `list_replicasets`, `list_jobs`, `list_cronjobs`
- `list_configmaps`, `list_secrets`, `list_nodes`, `list_events`
- `list_ingresses`, `list_persistentvolumeclaims`, `list_persistentvolumes`
- `list_serviceaccounts`, `list_roles`, `list_clusterroles`, `list_rolebindings`, `list_clusterrolebindings`
- `list_horizontalpodautoscalers`
- `list_storageclasses`, `list_networkpolicies`, `list_resourcequotas`, `list_limitranges` *(v1.1.0)*

#### Resource Management (8)
- `get_pod_logs`, `scale_deployment`, `restart_deployment`, `delete_resource`, `exec_pod`
- `cordon_node`, `uncordon_node`, `drain_node`

#### YAML Operations (2)
- `create_resource`, `edit_resource`

#### Rollback (1)
- `rollback_deployment`

#### Event Subscription (3)
- `subscribe_to_k8s_events`, `subscribe_to_all_k8s_events`, `unsubscribe_from_k8s_events`

---

## Frontend State Management

Store: `src/stores/kubernetesStore.ts` (Zustand, not persisted)

| State | Purpose |
|-------|---------|
| `selectedClusterId` | Active cluster (drives namespace/resource loading) |
| `selectedNamespace` | Active namespace filter |
| `clusters`, `contexts` | Cluster metadata |
| `namespaces` | Cached namespace list per cluster |
| `loadedResources` | Set of resource types currently loaded |
| `terminalSessions` | Active xterm.js terminal sessions |
| `globalSearchQuery` | Cross-resource search state |
| `bulkSelection` | Multi-resource selection per type |

---

## Key Files

| Path | Purpose |
|------|---------|
| `src/pages/Kubernetes/KubernetesPage.tsx` | Lens-like page shell (sidebar + hotbar + content) |
| `src/components/Kubernetes/ResourceBrowser.tsx` | Legacy resource browser (5 types) |
| `src/components/Kubernetes/ClusterOverview.tsx` | Live cluster summary |
| `src/components/Kubernetes/Terminal.tsx` | xterm.js pod exec terminal |
| `src/components/Kubernetes/YamlEditor.tsx` | Monaco YAML editor |
| `src/components/Kubernetes/MetricsChart.tsx` | recharts metrics visualization |
| `src/components/Kubernetes/RbacViewer.tsx` | Live RBAC resource viewer |
| `src/components/Kubernetes/RbacEditor.tsx` | RBAC create/edit via YAML |
| `src/components/Kubernetes/CommandPalette.tsx` | Ctrl+K command palette |
| `src/lib/eventBus.ts` | Frontend event bus for K8s watchers |
| `src-tauri/src/commands/kube.rs` | All 48 Kubernetes Tauri commands |
| `src-tauri/src/kube/` | Client, port forward, watcher, refresh modules |

---

## Dependencies

### Frontend (npm)
| Package | Version | Purpose |
|---------|---------|---------|
| `xterm` | 5.x | Terminal emulator |
| `xterm-addon-fit` | 0.8.x | Auto-resize |
| `xterm-addon-web-links` | 0.9.x | Clickable URLs |
| `@monaco-editor/react` | 4.x | YAML editor |
| `recharts` | 2.x | Metrics charts |

### Backend (Cargo)
No external Kubernetes client libraries. Uses `tokio::process::Command` + bundled kubectl binary.

---

## Known Limitations

1. **Metrics**: CPU/memory charts show placeholder data — requires metrics-server integration (future work)
2. **Real-time updates**: Watcher backend exists but frontend integration is polling-based; true watch streams pending
3. **Helm**: Not yet integrated (planned for v1.2.0)
4. **StorageClasses**: Cluster-scoped, no namespace filter
5. **Node metrics**: Cordon/drain requires cluster admin privileges
