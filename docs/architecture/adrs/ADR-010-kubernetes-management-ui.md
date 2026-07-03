# ADR-010: Kubernetes Management UI

## Status

Accepted

## Context

The application needed a complete Kubernetes Management UI with feature parity to Lens Desktop v5.x. This required implementing:

1. **Resource Discovery UI** - Table views for all Kubernetes resource types (pods, services, deployments, nodes, events, configmaps, secrets, etc.)
2. **Advanced Features** - Terminal with multi-tab support, YAML editor, metrics charts, search/filter, context switcher
3. **Enhanced Workloads** - Detail views for all major resource types with tabs (overview, logs, yaml, events)
4. **Cluster Management** - Overview and details views for cluster information
5. **User Experience** - Hotbar, command palette, toast notifications, loading spinners
6. **Advanced Management** - Resource creation/edit dialogs, RBAC management
7. **Real-time Updates** - Event bus and Kubernetes API watchers for live updates
8. **RBAC Management** - Viewer and editor for roles, clusterroles, bindings

## Decision

We implemented a complete Kubernetes Management UI following the existing architecture:

- **Frontend**: React + TypeScript + Zustand (state management)
- **Backend**: Tauri 2 + Rust with existing kube commands
- **UI Components**: Custom shadcn-style components with Tailwind CSS
- **State Management**: Zustand `kubernetesStore` for clusters, namespaces, resources, terminals, search, bulk selection

### Key Design Decisions

1. **Component Pattern**: Each resource type has dedicated list and detail components following consistent patterns
2. **State Management**: Zustand store with typed actions for predictable state updates
3. **Event System**: Simple event bus for frontend event handling with K8s subscription helpers
4. **Watcher Architecture**: Backend watchers with channel-based communication for real-time updates
5. **Security**: PII detection before external sends, hash-chained audit logging

### Implementation Details

- **26 Resource Components**: PodList, ServiceList, DeploymentList, StatefulSetList, DaemonSetList, NodeList, EventList, ConfigMapList, SecretList, ReplicaSetList, JobList, CronJobList, IngressList, PVCList, PVList, ServiceAccountList, RoleList, ClusterRoleList, RoleBindingList, ClusterRoleBindingList, HPAList, plus detail views
- **Advanced Components**: Terminal, YamlEditor, MetricsChart, SearchBar, ContextSwitcher, ApplicationView
- **UX Components**: Hotbar, CommandPalette, Toast, LoadingSpinner
- **Management Components**: CreateResourceModal, EditResourceModal, RbacViewer, RbacEditor
- **Backend**: Event bus, watcher module with subscribe/unsubscribe commands

### Dependencies Added

- **Frontend**: xterm, xterm-addon-fit, xterm-addon-web-links (terminal), @monaco-editor/react (YAML editor), react-chartjs-2, chart.js (metrics)
- **Backend**: k8s-openapi with watch feature for live watcher streams

## Consequences

### Positive

- Complete Lens-like Kubernetes Management UI
- Real-time updates via event bus and watchers
- RBAC management with viewer and editor
- Extensible architecture for future features
- Consistent UI patterns across all resource types

### Negative

- Large dependency footprint (xterm, monaco-editor, chart.js)
- Build size increased (~584 KB JS bundle)

### Operational Notes

- Metrics charts are backed by live cluster data
- Terminal and YAML editor rely on their bundled frontend libraries
- Watchers deliver real-time updates through the backend event bus

## References

- [Kubernetes Management Implementation Plan](../KUBERNETES-MANAGEMENT-IMPLEMENTATION-PLAN.md)
- [Lens Desktop v5.x Features](../lens-desktop-v5x-features.md)
- [Tauri Documentation](https://tauri.app)
- [React Documentation](https://react.dev)
- [Zustand Documentation](https://zustand-demo.pmnd.rs)
