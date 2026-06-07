# Kubernetes Management

This document describes the Kubernetes Management UI implementation in Troubleshooting and RCA Assistant.

## Overview

The application includes a complete Kubernetes Management UI with feature parity to Lens Desktop v5.x, implemented in two phases:

- **Phase 1 (v1.0.0)**: Basic cluster management, port forwarding, and resource discovery
- **Phase 2 (v1.1.0)**: Advanced features, enhanced workloads, and real-time updates

## Features

### Phase 1: Basic Management

- **Cluster Management**: Add, remove, list clusters with kubeconfig support
- **Port Forwarding**: Start, stop, list, and delete port forwards
- **Resource Discovery**: View pods, services, deployments, statefulsets, daemonsets, namespaces
- **Resource Management**: Scale, restart, delete, exec into resources
- **Context Switching**: Switch between clusters and namespaces

### Phase 2: Advanced Features

- **26 Resource Types**: All major Kubernetes resource types with table views
- **Detail Views**: Tabs for overview, logs, yaml, events for each resource
- **Terminal**: Multi-tab terminal with session management
- **YAML Editor**: Create and edit resources with YAML
- **Metrics Charts**: CPU, memory, and network usage visualization
- **Search & Filter**: Search by name, labels, annotations
- **Context Switcher**: Quick cluster and context switching
- **RBAC Management**: Viewer and editor for roles, clusterroles, bindings
- **Real-time Updates**: Event bus and Kubernetes API watchers

## Architecture

### Frontend

- **State Management**: Zustand `kubernetesStore` for clusters, namespaces, resources, terminals, search, bulk selection
- **Components**: 26 resource list components, 8 detail views, 8 advanced components, 6 UX components
- **Event System**: Simple event bus for frontend event handling

### Backend

- **Commands**: 43 kube-related commands in `src-tauri/src/commands/kube.rs`
- **Client**: Kubernetes client with kubeconfig support
- **Port Forwarding**: Complete port forward runtime with kubeconfig injection
- **Watchers**: Resource watchers with channel-based communication (placeholder implementation)

## Resource Types

### Workloads (11)
- Pod
- Deployment
- Service
- StatefulSet
- DaemonSet
- ReplicaSet
- Job
- CronJob
- Ingress
- HPA

### Infrastructure (5)
- Node
- Namespace
- PVC
- PV
- ServiceAccount

### Configuration (2)
- ConfigMap
- Secret

### RBAC (4)
- Role
- ClusterRole
- RoleBinding
- ClusterRoleBinding

### Events (1)
- Event

## API Commands

### Cluster Management
- `list_clusters()` - List all clusters
- `add_cluster()` - Add cluster with kubeconfig
- `remove_cluster()` - Remove cluster
- `set_active_cluster()` - Set active cluster

### Port Forwarding
- `list_port_forwards()` - List active port forwards
- `start_port_forward()` - Start port forward
- `stop_port_forward()` - Stop port forward
- `delete_port_forward()` - Delete port forward
- `shutdown_port_forwards()` - Shutdown all port forwards

### Resource Discovery
- `list_pods()` - List pods
- `list_services()` - List services
- `list_deployments()` - List deployments
- `list_statefulsets()` - List statefulsets
- `list_daemonsets()` - List daemonsets
- `list_namespaces()` - List namespaces
- `list_nodes()` - List nodes
- `list_events()` - List events
- `list_configmaps()` - List configmaps
- `list_secrets()` - List secrets
- `list_replicasets()` - List replicasets
- `list_jobs()` - List jobs
- `list_cronjobs()` - List cronjobs
- `list_ingresses()` - List ingresses
- `list_pvcs()` - List PVCs
- `list_pvs()` - List PVs
- `list_serviceaccounts()` - List service accounts
- `list_roles()` - List roles
- `list_clusterroles()` - List cluster roles
- `list_rolebindings()` - List role bindings
- `list_clusterrolebindings()` - List cluster role bindings
- `list_hpas()` - List HPAs

### Resource Management
- `get_pod_detail()` - Get pod details
- `get_deployment_detail()` - Get deployment details
- `get_service_detail()` - Get service details
- `get_configmap_detail()` - Get configmap details
- `get_secret_detail()` - Get secret details
- `get_node_detail()` - Get node details
- `get_namespace_detail()` - Get namespace details
- `get_pvc_detail()` - Get PVC details
- `get_pv_detail()` - Get PV details
- `get_serviceaccount_detail()` - Get service account details
- `get_role_detail()` - Get role details
- `get_clusterrole_detail()` - Get cluster role details
- `get_rolebinding_detail()` - Get role binding details
- `get_clusterrolebinding_detail()` - Get cluster role binding details
- `get_hpa_detail()` - Get HPA details
- `get_event_detail()` - Get event details
- `get_replicaset_detail()` - Get replica set details
- `get_job_detail()` - Get job details
- `get_cronjob_detail()` - Get cronjob details
- `get_ingress_detail()` - Get ingress details
- `scale_deployment()` - Scale deployment
- `restart_deployment()` - Restart deployment
- `delete_resource()` - Delete resource
- `exec_into_pod()` - Execute command in pod
- `get_pod_logs()` - Get pod logs
- `get_resource_yaml()` - Get resource YAML

### Advanced
- `subscribe_to_k8s_events()` - Subscribe to K8s events
- `subscribe_to_all_k8s_events()` - Subscribe to all K8s events
- `unsubscribe_from_k8s_events()` - Unsubscribe from events

## State Management

### Kubernetes Store (`src/stores/kubernetesStore.ts`)

```typescript
interface KubernetesState {
  clusters: Cluster[];
  activeClusterId: string | null;
  namespaces: Namespace[];
  activeNamespace: string | null;
  resources: Record<string, Resource[]>;
  resourceLoading: Record<string, boolean>;
  terminals: TerminalSession[];
  searchQuery: string;
  searchResults: Resource[];
  bulkSelection: Set<string>;
}
```

## Event System

### Event Bus (`src/lib/eventBus.ts`)

```typescript
// Subscribe to events
const unsubscribe = eventBus.on('k8s:resource:updated', (data) => {
  console.log('Resource updated:', data);
});

// Unsubscribe
unsubscribe();

// Emit events
eventBus.emit('k8s:resource:updated', {
  clusterId: 'cluster-1',
  namespace: 'default',
  resourceType: 'pod',
  resource: podData
});
```

## Future Enhancements

- **Helm Support**: Chart management and release tracking
- **Extension System**: Plugin architecture for custom features
- **Advanced Metrics**: Custom metrics and dashboards
- **Bulk Actions**: Batch operations on resources
- **Resource Creation**: Form-based resource creation
- **Health Monitoring**: Cluster and resource health status

## Dependencies

### Frontend
- `xterm` - Terminal rendering
- `xterm-addon-fit` - Terminal resizing
- `xterm-addon-web-links` - Web link detection
- `@monaco-editor/react` - YAML editor
- `react-chartjs-2` - Metrics charts
- `chart.js` - Chart rendering

### Backend
- `k8s-openapi` with `watch` feature - Kubernetes API watchers
- `tokio-stream` - Async streams for watchers

## Testing

### Frontend Tests
- 114 tests passing
- Unit tests for stores, components, and utilities

### Backend Tests
- 331 tests passing
- Tests for kube commands, port forwarding, and resource management

## Documentation

- [Kubernetes Management Implementation Plan](../KUBERNETES-MANAGEMENT-IMPLEMENTATION-PLAN.md)
- [Lens Desktop v5.x Features](../lens-desktop-v5x-features.md)
- [Architecture Documentation](../architecture/README.md)
- [ADR-010: Kubernetes Management UI](../architecture/adrs/ADR-010-kubernetes-management-ui.md)
