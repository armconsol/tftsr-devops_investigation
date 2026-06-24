# FreeLens Feature Inventory — Complete Analysis

**Project**: FreeLens (https://github.com/freelensapp/freelens)
**License**: MIT License (Copyright 2024-2026 Freelens Authors; Copyright 2022 OpenLens Authors)
**Description**: Free and open-source Kubernetes IDE, community fork of Open Lens v5
**Analysis Date**: 2026-06-08
**Repository Commit**: main branch (latest)

---

## Executive Summary

FreeLens is a production-ready, feature-complete Kubernetes desktop IDE built on Electron with a comprehensive resource management interface. The application provides extensive coverage of Kubernetes API resources with dedicated UI components, context menus, and detail views for nearly all standard Kubernetes objects.

**Key Findings**:
- **13 main navigation categories** with 60+ resource types
- **Comprehensive pod management**: shell/exec, logs, attach, edit, delete, force delete, force finalize
- **Full workload lifecycle**: scale, restart, edit, delete for Deployments, StatefulSets, DaemonSets
- **Helm chart integration**: install, upgrade, rollback, delete
- **Port forwarding UI**: start/stop/edit/open in browser
- **Terminal integration**: built-in terminal with kubectl and node shell access
- **Resource metrics**: CPU/memory usage visualization (when metrics-server available)
- **YAML editing**: Monaco editor with syntax highlighting
- **RBAC management**: full support for roles, bindings, service accounts
- **Extension ecosystem**: plugin architecture for custom functionality

---

## Left Navigation Structure (Complete)

### 1. Favorites
- User-bookmarked resources for quick access

### 2. Cluster Overview
- Cluster-wide dashboard with health metrics

### 3. Nodes
- Node list and details
- **Context Menu Actions**:
  - Shell (node shell access via SSH or similar)
  - Cordon/Uncordon
  - Drain (with confirmation)
  - Edit
  - Delete

### 4. Workloads
Parent category containing:

#### 4.1 Overview
- Aggregated workload dashboard

#### 4.2 Pods
- Pod list with status, IP, node, age
- **Context Menu Actions**:
  - Shell (per-container with auto-detection: bash/ash/sh, PowerShell for Windows nodes)
  - Logs (per-container, including init and ephemeral containers)
  - Attach (kubectl attach -it)
  - Edit (YAML editor)
  - Delete (graceful)
  - Force Delete (skip grace period, only for Running/Pending phases)
  - Force Finalize (remove finalizers when stuck)

#### 4.3 Deployments
- **Context Menu Actions**:
  - Scale (replica count dialog)
  - Restart (rolling restart)
  - Edit
  - Delete

#### 4.4 StatefulSets
- **Context Menu Actions**:
  - Restart
  - Edit
  - Delete

#### 4.5 DaemonSets
- **Context Menu Actions**:
  - Restart
  - Edit
  - Delete

#### 4.6 Jobs
- **Context Menu Actions**:
  - Edit
  - Delete

#### 4.7 CronJobs
- **Context Menu Actions**:
  - Edit
  - Delete

#### 4.8 ReplicaSets
- List view (typically managed by Deployments)

#### 4.9 ReplicationControllers
- Legacy replication support

### 5. Config
Parent category containing:

#### 5.1 ConfigMaps
- **Context Menu Actions**:
  - Edit
  - Delete

#### 5.2 Secrets
- **Context Menu Actions**:
  - Edit (with data obfuscation)
  - Delete

#### 5.3 Horizontal Pod Autoscalers (HPA)
- HPA configuration and status

#### 5.4 Vertical Pod Autoscalers (VPA)
- VPA recommendations and settings

#### 5.5 Resource Quotas
- Namespace quota limits

#### 5.6 Limit Ranges
- Default resource limits

#### 5.7 Priority Classes
- Pod scheduling priority definitions

#### 5.8 Runtime Classes
- Container runtime selection

#### 5.9 Pod Disruption Budgets
- PDB configuration

#### 5.10 Leases
- Coordination.k8s.io lease objects

#### 5.11 Mutating Webhook Configurations
- Admission webhook config

#### 5.12 Validating Webhook Configurations
- Validation webhook config

### 6. Network
Parent category containing:

#### 6.1 Services
- Service list and endpoints
- **Context Menu Actions**:
  - Edit
  - Delete

#### 6.2 Ingresses
- Ingress rules and backends

#### 6.3 Ingress Classes
- IngressClass definitions

#### 6.4 Network Policies
- Network segmentation rules

#### 6.5 Endpoints
- Service endpoint slices

#### 6.6 Endpoint Slices
- EndpointSlice objects

#### 6.7 Port Forwards
- Active port-forward management
- **Context Menu Actions**:
  - Open (in browser, for HTTP/HTTPS)
  - Edit (change local/remote port, protocol)
  - Start
  - Stop
  - Delete

### 7. Storage
Parent category containing:

#### 7.1 Persistent Volumes
- Cluster-wide PV list

#### 7.2 Persistent Volume Claims
- PVC list with binding status

#### 7.3 Storage Classes
- Dynamic provisioning configuration

### 8. Namespaces
- Namespace list and quota overview
- Namespace filtering (global namespace selector in UI)

### 9. Events
- Cluster events stream with filtering

### 10. Helm
Parent category containing:

#### 10.1 Charts
- Helm chart repository browser
- Search across configured repositories
- **Chart Actions**:
  - Install (opens install dialog with values editor)

#### 10.2 Releases
- Deployed Helm releases
- **Context Menu Actions**:
  - Upgrade (opens upgrade dialog)
  - Rollback (to previous revision)
  - Delete

### 11. User Management (RBAC)
Parent category containing:

#### 11.1 Service Accounts
- **Context Menu Actions**:
  - Edit
  - Delete

#### 11.2 Roles
- Namespace-scoped RBAC roles

#### 11.3 Role Bindings
- Role-to-subject mappings

#### 11.4 Cluster Roles
- Cluster-wide RBAC roles

#### 11.5 Cluster Role Bindings
- ClusterRole-to-subject mappings

### 12. Custom Resources
- **Custom Resource Definitions (CRDs)**
- **Custom Resources** (instances of CRDs)
- Dynamic UI generation for any CRD installed in cluster

### 13. Pod Security Policies (PSP)
- Legacy PSP support (deprecated in K8s 1.25+)

---

## Detail Views

All resources support a **detail drawer** (right-side panel) showing:

### Pod Detail View
- **Status** (Running, Pending, Failed, etc.)
- **Node** (clickable link to node)
- **Host IPs** (multi-IP support)
- **Pod IPs** (IPv4/IPv6)
- **Service Account** (clickable link)
- **Priority Class** (clickable link)
- **QoS Class** (BestEffort, Burstable, Guaranteed)
- **Runtime Class** (clickable link)
- **Termination Grace Period**
- **Node Selector** (labels)
- **Tolerations** (with key/value/effect)
- **Affinity/Anti-Affinity** (node and pod affinity rules)
- **Resource Requests** (CPU, memory, ephemeral-storage)
- **Resource Limits** (CPU, memory)
- **Secrets** (mounted secrets with clickable links)
- **Conditions** (PodScheduled, Initialized, ContainersReady, Ready)
- **Init Containers** (with status, restart count, state)
- **Containers** (with status, restart count, image, ports, env vars, volume mounts, liveness/readiness probes)
- **Ephemeral Containers** (debug containers)
- **Volumes** (ConfigMaps, Secrets, PVCs, EmptyDir, HostPath, etc.)

### Other Resource Detail Views
- **Deployment**: replicas, strategy, conditions, selector, pod template
- **Service**: type, cluster IP, external IP, ports, selector, endpoints
- **ConfigMap**: data key-value pairs
- **Secret**: data keys (values obfuscated)
- **Node**: conditions, addresses, capacity, allocatable, taints, images
- **PVC**: access modes, storage class, volume name, capacity
- **Ingress**: rules, TLS, backends

All detail views include:
- **Metadata** section (name, namespace, labels, annotations, creation time, resource version, UID)
- **YAML view** (Monaco editor with syntax highlighting)
- **Events** related to the resource

---

## Dock Panel (Bottom Panel)

The dock is a tabbed bottom panel supporting multiple simultaneous views:

### Terminal
- **Node Shell**: SSH or similar access to cluster nodes
- **Pod Shell**: `kubectl exec -it` with container selection
- **Pod Attach**: `kubectl attach -it` for attaching to running container
- **Custom Commands**: run arbitrary kubectl commands
- **Multi-tab support**: multiple shells in separate tabs
- **Shell Detection**: auto-selects bash/ash/sh on Linux, PowerShell on Windows nodes

### Logs
- **Pod Logs**: per-container log streaming
- **Container Selection**: dropdown for multi-container pods (including init and ephemeral)
- **Follow Mode**: tail -f equivalent
- **Timestamps**: toggle timestamp display
- **Previous Logs**: view logs from crashed/restarted containers
- **Search/Filter**: text search within logs
- **Download**: save logs to file
- **Wrap Lines**: toggle line wrapping

### Edit Resource
- **YAML Editor**: Monaco-based syntax highlighting
- **Apply Changes**: update resource via kubectl apply
- **Validation**: client-side YAML validation
- **Diff View**: show changes before applying

### Create Resource
- **YAML Editor**: create new resources from scratch
- **Templates**: common resource templates
- **Multi-resource**: create multiple resources from YAML with `---` separator

### Install Chart
- **Chart Selection**: from Helm repository browser
- **Values Editor**: YAML editor for values.yaml
- **Release Name**: custom release name
- **Namespace Selection**: target namespace
- **Preview**: dry-run before install

### Upgrade Chart
- **Current Values**: shows existing values
- **New Version Selection**: dropdown of available chart versions
- **Values Diff**: highlight changes from current release
- **Revision History**: list previous revisions

---

## Special Features

### Metrics & Resource Usage
- **Pod Metrics**: CPU and memory usage graphs (requires metrics-server)
- **Node Metrics**: cluster-wide resource utilization
- **Container Metrics**: per-container CPU/memory in detail view
- **Historical Charts**: time-series graphs for resource usage

### Namespace Filtering
- **Global Namespace Selector**: filters all views to selected namespace(s)
- **Multi-namespace Selection**: view resources across multiple namespaces
- **All Namespaces**: cluster-wide view

### Search & Filtering
- **Global Search**: search across all resource types
- **Per-View Search**: resource-specific search with multiple field filtering
- **Label Filtering**: filter by labels and annotations

### Context Menu Behavior
- **Toolbar Mode**: icons with tooltips in detail view header
- **Table Row Menu**: three-dot menu in list views
- **Right-click Context Menu**: anywhere on resource row

### Delete Modes (Intelligent)

FreeLens implements **intelligent delete mode selection** based on resource state:

#### Pod Deletion
- **Delete** (graceful): default for all phases
- **Force Delete** (grace period = 0): only shown for Running/Pending pods with `terminationGracePeriodSeconds > 0`
- **Force Finalize** (remove finalizers): shown when pod has `deletionTimestamp` AND finalizers

Logic prevents showing "Force Delete" for terminal phases (Succeeded, Failed, Unknown) where it would have no effect.

#### Generic Resource Deletion
- **Delete**: default
- **Force Finalize**: only when resource has `deletionTimestamp` AND finalizers

### Confirmation Dialogs
All destructive actions (delete, drain, restart) require user confirmation with resource name displayed.

---

## Kubernetes API Coverage

FreeLens supports **all standard Kubernetes API groups**:

### Core (v1)
- Pods, Services, Endpoints, ConfigMaps, Secrets, Namespaces, Nodes, PersistentVolumes, PersistentVolumeClaims, ServiceAccounts, Events, ResourceQuotas, LimitRanges

### Apps (apps/v1)
- Deployments, StatefulSets, DaemonSets, ReplicaSets, ReplicationControllers

### Batch (batch/v1, batch/v1beta1)
- Jobs, CronJobs

### Networking (networking.k8s.io/v1)
- Ingresses, IngressClasses, NetworkPolicies

### Storage (storage.k8s.io/v1)
- StorageClasses, VolumeAttachments

### RBAC (rbac.authorization.k8s.io/v1)
- Roles, RoleBindings, ClusterRoles, ClusterRoleBindings

### Autoscaling (autoscaling/v1, autoscaling/v2)
- HorizontalPodAutoscalers, VerticalPodAutoscalers

### Policy (policy/v1, policy/v1beta1)
- PodDisruptionBudgets, PodSecurityPolicies

### Admission (admissionregistration.k8s.io/v1)
- MutatingWebhookConfigurations, ValidatingWebhookConfigurations

### Scheduling (scheduling.k8s.io/v1)
- PriorityClasses

### Node (node.k8s.io/v1)
- RuntimeClasses

### Coordination (coordination.k8s.io/v1)
- Leases

### Discovery (discovery.k8s.io/v1)
- EndpointSlices

### Custom Resources
- Full CRD support with dynamic UI generation

### Helm
- Charts, Releases (via Helm API, not native K8s)

---

## Extension System

FreeLens supports extensions via a plugin API:
- **Custom Pages**: add new sidebar items and routes
- **Custom Menus**: inject menu items into resource context menus
- **Custom Resource Views**: override or enhance detail views
- **Protocol Handlers**: register custom URL schemes
- **Preferences**: add extension settings to preferences UI

Extensions are TypeScript/JavaScript modules loaded at runtime.

---

## Comparison to TFTSR Requirements

Based on the TFTSR project's needs for Kubernetes cluster management, FreeLens provides:

### Strengths
✅ **Complete resource coverage**: All K8s API objects supported
✅ **Shell execution**: Built-in terminal with pod exec and node shell
✅ **Log streaming**: Real-time log viewing with container selection
✅ **YAML editing**: Monaco editor with validation
✅ **Port forwarding**: Full UI for managing forwards
✅ **Helm integration**: Chart install, upgrade, rollback
✅ **RBAC management**: Full RBAC resource support
✅ **Extension API**: Customizable via plugins
✅ **Multi-cluster**: Supports multiple kubeconfig contexts
✅ **Metrics**: Resource usage visualization (when metrics-server available)
✅ **Open source**: MIT licensed, can be forked/customized

### Potential Gaps for TFTSR
⚠️ **No AI integration**: FreeLens is a pure Kubernetes IDE, no AI/ML features
⚠️ **No RCA/triage features**: No incident management or root cause analysis
⚠️ **No PII detection**: Standard K8s IDE, no data privacy features
⚠️ **No audit logging**: No built-in audit trail (relies on K8s audit logs)
⚠️ **Electron-based**: Desktop app, not web-based (may not fit deployment model)
⚠️ **No integrations**: No Confluence, ServiceNow, ADO connectors

### Feature Parity Opportunities
If building TFTSR's K8s management UI, FreeLens demonstrates best practices for:
- **Resource action menus**: Comprehensive context menus with confirmation flows
- **Detail views**: Structured drawer layout with expandable sections
- **Intelligent delete modes**: State-aware action availability
- **Terminal integration**: Seamless kubectl exec and attach
- **Log viewer**: Feature-rich log streaming with filters
- **Port forward UI**: Start/stop/edit/open workflow
- **Helm UI**: Chart browser, install wizard, upgrade/rollback flows

---

## Technical Architecture Insights

### Codebase Organization
- **Dependency Injection**: Uses `@ogre-tools/injectable` for all services
- **State Management**: MobX for reactive stores
- **Component Pattern**: React with TypeScript, HOCs for injection
- **Menu System**: Dynamic menu generation based on resource type and state
- **API Layer**: Abstractions for `kubectl`, Helm API, metrics-server
- **Store Pattern**: Separate stores for each resource type with watch API integration

### Key Design Patterns
1. **KubeObjectMenu**: Generic menu component that dynamically injects resource-specific menu items
2. **Sidebar Items**: Injectable pattern for navigation tree construction
3. **Detail Views**: Drawer-based detail panels with tabbed sections
4. **Dock System**: Multi-tab bottom panel for logs, terminal, editors
5. **State-aware Actions**: Action availability based on resource phase, deletion timestamp, finalizers

### Menu Item Registration
Each resource type registers menu items via injectables:
- `pod-shell-menu.tsx`: Shell action for pods
- `pod-logs-menu.tsx`: Logs action for pods
- `deployment-menu.tsx`: Scale and Restart for deployments
- `node-menu.tsx`: Cordon, Uncordon, Drain for nodes

This modular approach allows easy extension without modifying core menu code.

---

## Recommendations for TFTSR

### 1. Feature Parity Checklist
If implementing K8s management in TFTSR, prioritize:
- [ ] Pod shell exec (with container selection)
- [ ] Log streaming (with follow/timestamps/search)
- [ ] YAML editor (with validation)
- [ ] Delete modes (graceful, force, finalize based on state)
- [ ] Port forwarding UI
- [ ] Helm chart management
- [ ] Resource detail views (structured drawer layout)
- [ ] Namespace filtering
- [ ] Metrics/resource usage (if metrics-server available)

### 2. Integration Points
TFTSR could integrate K8s management with:
- **AI Analysis**: Use pod logs, events, describe output as context for AI triage
- **RCA Workflow**: Link K8s resources to incident timeline
- **Audit Trail**: Log all kubectl commands executed via UI
- **PII Detection**: Scan logs and ConfigMaps before AI processing

### 3. Web vs Desktop
FreeLens is Electron-based. For TFTSR (likely Tauri web UI):
- **Pros**: Can reuse architecture patterns, menu system, detail view layouts
- **Cons**: Cannot directly fork FreeLens (Electron vs Tauri)
- **Approach**: Study FreeLens UI/UX patterns, implement in React + Tauri with Rust backend

### 4. Licensing
MIT license allows:
- ✅ Studying code for design patterns
- ✅ Borrowing UI/UX concepts
- ✅ Forking and modifying (with attribution)
- ❌ Cannot claim FreeLens authors' copyright as your own

---

## Sources

1. FreeLens GitHub Repository. "freelensapp/freelens." GitHub, 2026-06-08. https://github.com/freelensapp/freelens
2. FreeLens. "LICENSE." MIT License, 2024-2026. https://github.com/freelensapp/freelens/blob/main/LICENSE
3. FreeLens. "KubeObjectMenu Component." TypeScript source, main branch. `/packages/core/src/renderer/components/kube-object-menu/kube-object-menu.tsx`
4. FreeLens. "Pod Menu Actions." TypeScript source, main branch. `/packages/core/src/renderer/components/node-pod-menu/`
5. FreeLens. "Sidebar Navigation." TypeScript source, main branch. `/packages/core/src/common/sidebar-menu-items-starting-order.ts`
6. FreeLens. "Deployment, StatefulSet, DaemonSet Menus." TypeScript source, main branch. `/packages/core/src/renderer/components/workloads-*/`
7. FreeLens. "Helm Release Menu." TypeScript source, main branch. `/packages/core/src/renderer/components/helm-releases/release-menu.tsx`
8. FreeLens. "Port Forward Menu." TypeScript source, main branch. `/packages/core/src/renderer/components/network-port-forwards/port-forward-menu.tsx`

---

**Analysis completed by**: Claude Code (Technical Researcher)
**Format**: Markdown ticket for project documentation
