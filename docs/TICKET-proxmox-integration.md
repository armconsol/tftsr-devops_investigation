# Proxmox Integration Implementation Plan

**Version:** v1.2.0  
**Date:** 2026-06-06  
**Status:** Planning Phase

---

## Executive Summary

Implement a full-featured Proxmox integration into TRCAA that supports both Proxmox VE (Virtual Environment) and Proxmox Backup Server (PBS) with multi-cluster management, cross-datacenter metrics, live migrations, and full administrative functions. Authentication uses root credentials via the default Proxmox API ports (8006 for VE, 8007 for PBS), with encrypted credential storage and API token management.

---

## Important Corrections & Clarifications

### Port Configuration

**Correction:** Proxmox VE and PBS use **different default ports**:

| Service | Default Port | API Endpoint |
|---------|--------------|--------------|
| Proxmox VE | **8006** | `https://hostname:8006/api2/json` |
| Proxmox Backup Server | **8007** | `https://hostname:8007/api2/json` |

**Implementation:**
- Default port set by cluster type (8006 for VE, 8007 for PBS)
- User can override port if needed
- Port displayed in cluster configuration UI

### Ceph Storage Management

**Addition:** Full Ceph cluster management required:

| Component | Management Operations |
|-----------|----------------------|
| **Ceph Pools** | Create, delete, list, quota management |
| **Ceph OSDs** | List, status, weight management, out/in |
| **Ceph MDS** | List, status, failover management |
| **Ceph RBD** | Create, delete, clone, snap, resize |
| **Ceph Monitors** | List, status, quorum health |
| **Ceph Health** | Overall cluster health monitoring |

**API Endpoints:**
```
GET  /api2/json/nodes/{node}/ceph/pools
POST /api2/json/nodes/{node}/ceph/pool
GET  /api2/json/nodes/{node}/ceph/osd
POST /api2/json/nodes/{node}/ceph/osd/{id}/set
GET  /api2/json/nodes/{node}/ceph/mds
GET  /api2/json/nodes/{node}/ceph/mon
GET  /api2/json/cluster/ceph/status
```

### Proxmox Datacenter Manager Features (v1.2.0)

**Addition:** Include these PDM features in v1.2.0:

1. **SDN (Software-Defined Networking)**
   - List virtual networks
   - View network status
   - Bridge configuration

2. **Firewall Management**
   - List firewall rules
   - Enable/disable firewall
   - Rule management (add, delete, update)

3. **HA (High Availability) Groups**
   - List HA groups
   - Manage HA resources
   - Failover configuration

4. **Update Management**
   - Check for package updates
   - List available updates
   - Update status across clusters

5. **User Management Integration**
   - LDAP integration status
   - AD integration status
   - OpenID Connect status

### Backup Management Scope

**Clarification:** Full backup job management including:

| Feature | Description |
|---------|-------------|
| **Backup Scheduling** | Cron-style scheduling for backup jobs |
| **Trigger Backups** | Manual backup job execution |
| **Backup Restoration** | Restore backups to target cluster |
| **Backup Replication** | Cross-cluster backup replication |
| **Deduplication** | Monitor deduplication status |
| **Backup Jobs** | Create, delete, list, edit backup jobs |

**API Endpoints:**
```
GET  /api2/json/nodes/{node}/backup
POST /api2/json/nodes/{node}/backup/{jobid}/run
GET  /api2/json/nodes/{node}/backup/status
POST /api2/json/nodes/{node}/backup/restore
```

### Cluster Selection UI

**Requirement:** Dropdown with three selection modes:

| Mode | Description | Use Case |
|------|-------------|----------|
| **Single Cluster** | Select one specific cluster | Targeted operations on one cluster |
| **Multiple Clusters** | Select 2+ specific clusters | Cross-cluster operations |
| **ALL Clusters** | All configured clusters | Global operations, dashboard |

**Implementation:**
- Cluster selector dropdown in sidebar
- "Select Mode" toggle (single/multi/all)
- Multi-select checkbox interface for "Multiple" mode
- "Select All" checkbox for "ALL" mode
- Visual indication of selected clusters

### Cross-Datacenter Features

**Clarification:** "Datacenter" means multiple Proxmox clusters managed together:

| Feature | Description |
|---------|-------------|
| **Global Dashboard** | Aggregated metrics across all clusters |
| **Cross-Cluster Search** | Search VMs/backups across all clusters |
| **Live Migration** | Migrate VMs between clusters |
| **Backup Replication** | Replicate backups between datacenters |
| **Unified Alerts** | Single view of all cluster health |

---

## Requirements

### Must-Have Features

1. **Authentication & Security**
   - Root username/password authentication to Proxmox nodes
   - API token generation and storage (encrypted)
   - Fingerprint verification for SSL/TLS connections
   - Support for self-signed certificates (common in Proxmox deployments)
   - All credentials encrypted at rest using AES-256-GCM

2. **Multi-Cluster Management**
   - Add/remove Proxmox clusters (VE and/or PBS)
   - List all configured clusters
   - Active/standby cluster support
   - Cross-cluster resource visibility
   - Cluster health monitoring

3. **Proxmox VE Functions**
   - View cluster status and resource utilization (CPU, RAM, storage)
   - VM lifecycle management (start, stop, reboot, shutdown, suspend)
   - VM configuration viewing and editing
   - Storage and disk management
   - Network configuration
   - HA (High Availability) group management
   - Clone, migrate, template management

4. **Proxmox Backup Server Functions**
   - View backup status and health
   - Backup job management
   - Datastore management
   - Backup restoration capabilities
   - Deduplication status

5. **Cross-Datacenter Features**
   - Dashboard aggregating all clusters
   - Resource utilization across clusters
   - Live migration between clusters
   - Backup replication monitoring
   - Global search across clusters

6. **Triage Integration**
   - Link Proxmox resources to issues
   - VM/host logs collection
   - Integration with existing triage workflow
   - PII detection in Proxmox logs

---

## Technical Architecture

### Backend (`src-tauri/`)

#### 1. New Module: `src-tauri/src/proxmox/`

```
src-tauri/src/proxmox/
├── mod.rs                 # Module exports
├── client.rs              # Proxmox API client (VE + PBS)
├── cluster.rs             # Cluster management logic
├── auth.rs                # Authentication (root creds → API token)
├── models.rs              # Rust models for Proxmox API
├── metrics.rs             # Cross-cluster metrics aggregation
├── migration.rs           # Live migration logic
└── backup.rs              # PBS backup management
```

#### 2. Database Schema Updates

**Migration 012: Proxmox Clusters**

```sql
CREATE TABLE IF NOT EXISTS proxmox_clusters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    node_address TEXT NOT NULL,      -- hostname:port (e.g., pve1.example.com:8006)
    node_fingerprint TEXT,           -- SSL fingerprint for verification
    username TEXT NOT NULL,          -- root or other user
    encrypted_password TEXT NOT NULL, -- AES-256-GCM encrypted
    cluster_type TEXT NOT NULL CHECK(cluster_type IN ('ve', 'pbs')),
    status TEXT NOT NULL DEFAULT 'unknown', -- 'connected', 'disconnected', 'error'
    last_connected_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(name, node_address)
);

CREATE TABLE IF NOT EXISTS proxmox_resources (
    id TEXT PRIMARY KEY,
    cluster_id TEXT NOT NULL REFERENCES proxmox_clusters(id) ON DELETE CASCADE,
    resource_type TEXT NOT NULL,     -- 'node', 'vm', 'ct', 'storage', 'backup', 'ceph_pool', 'ceph_osd', 'ceph_mds', 'ceph_rbd', 'sdn_zone', 'sdn_dhcp', 'firewall'
    resource_id TEXT NOT NULL,       -- VM ID, storage ID, pool name, OSD ID, etc.
    name TEXT,
    status TEXT,
    cpu_usage REAL,
    memory_usage REAL,
    storage_usage REAL,
    details TEXT,                    -- JSON blob for resource-specific data
    last_updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(cluster_id, resource_type, resource_id)
);

CREATE TABLE IF NOT EXISTS proxmox_credentials (
    id TEXT PRIMARY KEY,
    cluster_id TEXT NOT NULL REFERENCES proxmox_clusters(id) ON DELETE CASCADE,
    api_token TEXT NOT NULL,         -- Encrypted API token
    token_hash TEXT NOT NULL,        -- SHA-256 for audit
    expires_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(cluster_id)
);
```

**Update existing tables:**

```sql
-- Extend credentials CHECK constraint
-- (requires ALTER TABLE + data migration in SQLite)

-- Add proxmox to integration_config
ALTER TABLE integration_config 
ADD COLUMN proxmox_config TEXT;
```

#### 3. API Client Implementation

**Authentication Flow:**

```rust
// 1. User provides: hostname, root username, root password, SSL fingerprint
// 2. Validate SSL fingerprint (optional, for security)
// 3. POST /api2/json/access/ticket with root credentials
// 4. Receive: { ticket: "PVE@pam!root!<ticket>", CSRFPreventionToken: "<token>" }
// 5. Extract PVE ticket and convert to API token
// 6. Store encrypted API token in proxmox_credentials table
// 7. Cache token in memory with expiry

// API token format for Proxmox:
// <username>!<timestamp>!<hash>!<CSRFToken>
// Example: root!1686000000!abc123...!def456...
```

**Client Structure:**

```rust
pub struct ProxmoxClient {
    base_url: String,              // https://hostname:PORT (8006 for VE, 8007 for PBS)
    username: String,              // root or user
    api_token: String,             // Encrypted token
    csrf_token: String,            // For write operations
    verify_fingerprint: bool,      // Validate SSL cert
    cluster_type: ClusterType,     // VE or PBS
    client: reqwest::Client,
}

// Port configuration by cluster type
pub fn get_default_port(cluster_type: &ClusterType) -> u16 {
    match cluster_type {
        ClusterType::VE => 8006,
        ClusterType::PBS => 8007,
    }
}

// Cluster configuration with default port
pub struct ProxmoxClusterConfig {
    pub name: String,
    pub hostname: String,
    pub port: Option<u16>,         // None = use default based on cluster_type
    pub username: String,
    pub encrypted_password: String,
    pub verify_fingerprint: bool,
    pub cluster_type: ClusterType,
}
```

**Key Methods:**

```rust
impl ProxmoxClient {
    // Authentication
    pub async fn authenticate(username: &str, password: &str) -> Result<AuthResponse, String>
    
    // VE methods
    pub async fn list_vms(&self) -> Result<Vec<VirtualMachine>, String>
    pub async fn get_vm_status(&self, vm_id: u32) -> Result<VmStatus, String>
    pub async fn start_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn stop_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn reboot_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn shutdown_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn suspend_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn clone_vm(&self, vm_id: u32, new_id: u32, name: &str) -> Result<TaskResult, String>
    pub async fn migrate_vm(&self, vm_id: u32, target_node: &str, online: bool) -> Result<TaskResult, String>
    pub async fn list_nodes(&self) -> Result<Vec<NodeInfo>, String>
    pub async fn get_node_status(&self, node: &str) -> Result<NodeStatus, String>
    pub async fn list_storage(&self, node: &str) -> Result<Vec<StorageInfo>, String>
    pub async fn list_ha_groups(&self) -> Result<Vec<HAGroup>, String>
    
    // Ceph Management (Proxmox VE)
    pub async fn list_ceph_pools(&self) -> Result<Vec<CephPool>, String>
    pub async fn create_ceph_pool(&self, pool_name: &str, size: u32) -> Result<TaskResult, String>
    pub async fn delete_ceph_pool(&self, pool_name: &str) -> Result<TaskResult, String>
    pub async fn list_ceph_osds(&self) -> Result<Vec<CephOSD>, String>
    pub async fn set_ceph_osd_weight(&self, osd_id: u32, weight: f64) -> Result<TaskResult, String>
    pub async fn ceph_osd_out(&self, osd_id: u32) -> Result<TaskResult, String>
    pub async fn ceph_osd_in(&self, osd_id: u32) -> Result<TaskResult, String>
    pub async fn list_ceph_mds(&self) -> Result<Vec<CephMDS>, String>
    pub async fn ceph_mds_failover(&self, mds_name: &str) -> Result<TaskResult, String>
    pub async fn list_ceph_rbd(&self, pool_name: &str) -> Result<Vec<CephRBD>, String>
    pub async fn create_ceph_rbd(&self, pool_name: &str, rbd_name: &str, size: u64) -> Result<TaskResult, String>
    pub async fn delete_ceph_rbd(&self, pool_name: &str, rbd_name: &str) -> Result<TaskResult, String>
    pub async fn resize_ceph_rbd(&self, pool_name: &str, rbd_name: &str, new_size: u64) -> Result<TaskResult, String>
    pub async fn get_ceph_status(&self) -> Result<CephStatus, String>
    pub async fn get_ceph_health(&self) -> Result<CephHealth, String>
    
    // SDN Management (Proxmox VE)
    pub async fn list_sdn_zones(&self) -> Result<Vec<SDNZone>, String>
    pub async fn list_sdn_dhcp(&self) -> Result<Vec<SDNDHCP>, String>
    pub async fn list_sdn_firewall(&self) -> Result<Vec<SDNFirewall>, String>
    
    // Firewall Management (Proxmox VE)
    pub async fn list_firewall_rules(&self, node: &str) -> Result<Vec<FirewallRule>, String>
    pub async fn add_firewall_rule(&self, node: &str, rule: FirewallRule) -> Result<TaskResult, String>
    pub async fn delete_firewall_rule(&self, node: &str, rule_id: u32) -> Result<TaskResult, String>
    pub async fn enable_firewall(&self, node: &str) -> Result<TaskResult, String>
    pub async fn disable_firewall(&self, node: &str) -> Result<TaskResult, String>
    
    // PBS methods
    pub async fn list_backup_jobs(&self) -> Result<Vec<BackupJob>, String>
    pub async fn run_backup_job(&self, job_id: &str) -> Result<TaskResult, String>
    pub async fn list_datastores(&self) -> Result<Vec<Datastore>, String>
    pub async fn list_backups(&self, datastore: &str) -> Result<Vec<Backup>, String>
    pub async fn restore_backup(&self, backup_id: &str, datastore: &str) -> Result<TaskResult, String>
    pub async fn create_backup_job(&self, job: BackupJobConfig) -> Result<TaskResult, String>
    pub async fn delete_backup_job(&self, job_id: &str) -> Result<TaskResult, String>
    
    // Cross-cluster
    pub async fn get_cluster_metrics(&self) -> Result<ClusterMetrics, String>
}
```

**API Endpoint Mapping:**

| Operation | Endpoint |
|-----------|----------|
| **VE Authentication** | `POST /api2/json/access/ticket` |
| **PBS Authentication** | `POST /api2/json/access/ticket` |
| **List VMs** | `GET /api2/json/nodes/{node}/qemu` |
| **List Containers** | `GET /api2/json/nodes/{node}/lxc` |
| **List Nodes** | `GET /api2/json/nodes` |
| **List Storage** | `GET /api2/json/nodes/{node}/storage` |
| **List Ceph Pools** | `GET /api2/json/nodes/{node}/ceph/pool` |
| **List Ceph OSDs** | `GET /api2/json/nodes/{node}/ceph/osd` |
| **List Ceph MDS** | `GET /api2/json/nodes/{node}/ceph/mds` |
| **List Ceph RBD** | `GET /api2/json/nodes/{node}/ceph/rbd` |
| **List Ceph Status** | `GET /api2/json/cluster/ceph/status` |
| **List SDN Zones** | `GET /api2/json/nodes/{node}/sdn/zones` |
| **List Firewall Rules** | `GET /api2/json/nodes/{node}/firewall/rules` |
| **List Backup Jobs** | `GET /api2/json/nodes/{node}/backup` |
| **List Datastores** | `GET /api2/json/nodes/{node}/storage` |

```rust
impl ProxmoxClient {
    // Authentication
    pub async fn authenticate(username: &str, password: &str) -> Result<AuthResponse, String>
    
    // VE methods
    pub async fn list_vms(&self) -> Result<Vec<VirtualMachine>, String>
    pub async fn get_vm_status(&self, vm_id: u32) -> Result<VmStatus, String>
    pub async fn start_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn stop_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn reboot_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn shutdown_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn suspend_vm(&self, vm_id: u32) -> Result<TaskResult, String>
    pub async fn clone_vm(&self, vm_id: u32, new_id: u32, name: &str) -> Result<TaskResult, String>
    pub async fn migrate_vm(&self, vm_id: u32, target_node: &str, online: bool) -> Result<TaskResult, String>
    pub async fn list_nodes(&self) -> Result<Vec<NodeInfo>, String>
    pub async fn get_node_status(&self, node: &str) -> Result<NodeStatus, String>
    pub async fn list_storage(&self, node: &str) -> Result<Vec<StorageInfo>, String>
    pub async fn list_ha_groups(&self) -> Result<Vec<HAGroup>, String>
    
    // PBS methods
    pub async fn list_backup_jobs(&self) -> Result<Vec<BackupJob>, String>
    pub async fn run_backup_job(&self, job_id: &str) -> Result<TaskResult, String>
    pub async fn list_datastores(&self) -> Result<Vec<Datastore>, String>
    pub async fn list_backups(&self, datastore: &str) -> Result<Vec<Backup>, String>
    pub async fn restore_backup(&self, backup_id: &str, datastore: &str) -> Result<TaskResult, String>
    
    // Cross-cluster
    pub async fn get_cluster_metrics(&self) -> Result<ClusterMetrics, String>
}
```

#### 4. Cluster Management

**Cluster Registry:**

```rust
// src-tauri/src/proxmox/cluster.rs
pub struct ClusterRegistry {
    clusters: Mutex<HashMap<String, ProxmoxClient>>,
    config: Arc<Mutex<ClusterConfig>>,
}

impl ClusterRegistry {
    pub async fn add_cluster(&self, config: ProxmoxClusterConfig) -> Result<(), String>
    pub async fn remove_cluster(&self, cluster_id: &str) -> Result<(), String>
    pub async fn get_cluster(&self, cluster_id: &str) -> Option<&ProxmoxClient>
    pub async fn list_clusters(&self) -> Vec<ProxmoxClusterInfo>
    pub async fn get_all_metrics(&self) -> Result<Vec<ClusterMetrics>, String>
    pub async fn live_migration(&self, vm_id: u32, source_cluster: &str, target_cluster: &str) -> Result<TaskResult, String>
}
```

#### 5. Metrics Aggregation

**Cross-Cluster Dashboard Data:**

```rust
#[derive(Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub cluster_id: String,
    pub cluster_name: String,
    pub timestamp: String,
    pub nodes: Vec<NodeMetric>,
    pub vms: Vec<VmMetric>,
    pub storage: Vec<StorageMetric>,
    pub summary: ClusterSummary,
}

#[derive(Serialize, Deserialize)]
pub struct ClusterSummary {
    pub total_nodes: u32,
    pub total_vms: u32,
    pub running_vms: u32,
    pub stopped_vms: u32,
    pub total_cpu_cores: u32,
    pub used_cpu_cores: u32,
    pub total_ram_gb: f64,
    pub used_ram_gb: f64,
    pub total_storage_gb: f64,
    pub used_storage_gb: f64,
    pub health_status: HealthStatus, // 'healthy', 'warning', 'critical'
}
```

#### 6. Triage Integration

**Proxmox Resource Linking:**

```rust
// Link VM/host to issue
pub async fn link_proxmox_resource(
    issue_id: &str,
    cluster_id: &str,
    resource_type: &str,
    resource_id: &str,
) -> Result<LinkResult, String>

// Collect Proxmox logs for issue
pub async fn collect_proxmox_logs(
    issue_id: &str,
    cluster_id: &str,
    resource_type: &str,
    resource_id: &str,
    time_range: &str,  // e.g., "1h", "24h", "7d"
) -> Result<LogFile, String>
```

### Frontend (`src/`)

#### 1. Sidebar Update (`src/App.tsx`)

```typescript
import ProxmoxPage from "@/pages/Proxmox";

// Add to navItems
const navItems = [
  { to: "/", icon: Home, label: "Dashboard" },
  { to: "/new-issue", icon: Plus, label: "New Issue" },
  { to: "/history", icon: Clock, label: "History" },
  { to: "/proxmox", icon: Server, label: "Proxmox" }, // NEW
];

// Add route
<Route path="/proxmox" element={<ProxmoxPage />} />
```

#### 2. Proxmox Page (`src/pages/Proxmox/`)

```
src/pages/Proxmox/
├── index.tsx              # Main page with cluster selector
├── ClusterList.tsx        # Cluster management panel
├── ClusterDashboard.tsx   # Cluster metrics dashboard
├── VMManager.tsx          # VM management panel
├── BackupManager.tsx      # PBS backup management
├── AddClusterModal.tsx    # Add new cluster modal
├── ResourceViewer.tsx     # Resource details viewer
└── MigrationWizard.tsx    # Live migration wizard
```

**Main Page Structure:**

```tsx
<ProxmoxPage>
  {/* Cluster Selector */}
  <ClusterSelector />
  
  {/* Dashboard Tabs */}
  <Tabs>
    <Tab label="Overview">
      <ClusterDashboard />
    </Tab>
    <Tab label="VMs">
      <VMManager />
    </Tab>
    <Tab label="Backups">
      <BackupManager />
    </Tab>
    <Tab label="Clusters">
      <ClusterList />
    </Tab>
  </Tabs>
</ProxmoxPage>
```

#### 3. IPC Commands (`src/lib/tauriCommands.ts`)

```typescript
// Proxmox Cluster Management
export const addProxmoxClusterCmd = (config: ProxmoxClusterConfig) =>
  invoke<void>("add_proxmox_cluster", { config });

export const removeProxmoxClusterCmd = (clusterId: string) =>
  invoke<void>("remove_proxmox_cluster", { clusterId });

export const listProxmoxClustersCmd = () =>
  invoke<ProxmoxClusterInfo[]>("list_proxmox_clusters");

export const getProxmoxClusterCmd = (clusterId: string) =>
  invoke<ProxmoxClusterInfo>("get_proxmox_cluster", { clusterId });

// Authentication
export const testProxmoxConnectionCmd = (config: ProxmoxClusterConfig) =>
  invoke<ConnectionResult>("test_proxmox_connection", { config });

// VE Operations
export const listProxmoxVMsCmd = (clusterId: string) =>
  invoke<VirtualMachine[]>("list_proxmox_vms", { clusterId });

export const startProxmoxVMCmd = (clusterId: string, vmId: number) =>
  invoke<TaskResult>("start_proxmox_vm", { clusterId, vmId });

export const stopProxmoxVMCmd = (clusterId: string, vmId: number) =>
  invoke<TaskResult>("stop_proxmox_vm", { clusterId, vmId });

export const rebootProxmoxVMCmd = (clusterId: string, vmId: number) =>
  invoke<TaskResult>("reboot_proxmox_vm", { clusterId, vmId });

export const migrateProxmoxVMCmd = (clusterId: string, vmId: number, targetClusterId: string, online: boolean) =>
  invoke<TaskResult>("migrate_proxmox_vm", { clusterId, vmId, targetClusterId, online });

// PBS Operations
export const listProxmoxBackupsCmd = (clusterId: string) =>
  invoke<Backup[]>("list_proxmox_backups", { clusterId });

export const runProxmoxBackupJobCmd = (clusterId: string, jobId: string) =>
  invoke<TaskResult>("run_proxmox_backup_job", { clusterId, jobId });

// Metrics
export const getProxmoxMetricsCmd = (clusterId: string) =>
  invoke<ClusterMetrics>("get_proxmox_metrics", { clusterId });

export const getCrossClusterMetricsCmd = () =>
  invoke<ClusterMetrics[]>("get_cross_cluster_metrics");

// Triage Integration
export const linkProxmoxResourceCmd = (issueId: string, clusterId: string, resourceType: string, resourceId: string) =>
  invoke<LinkResult>("link_proxmox_resource", { issueId, clusterId, resourceType, resourceId });

export const collectProxmoxLogsCmd = (issueId: string, clusterId: string, resourceType: string, resourceId: string, timeRange: string) =>
  invoke<LogFile>("collect_proxmox_logs", { issueId, clusterId, resourceType, resourceId, timeRange });
```

#### 4. State Management

**Zustand Store (`src/stores/proxmoxStore.ts`):**

```typescript
import { create } from 'zustand';

interface ProxmoxState {
  clusters: ProxmoxClusterInfo[];
  activeClusterId: string | null;
  vms: Record<string, VirtualMachine[]>;
  metrics: Record<string, ClusterMetrics>;
  loading: boolean;
  error: string | null;
  
  // Actions
  addCluster: (cluster: ProxmoxClusterConfig) => Promise<void>;
  removeCluster: (clusterId: string) => Promise<void>;
  setActiveCluster: (clusterId: string | null) => void;
  refreshVms: (clusterId: string) => Promise<void>;
  refreshMetrics: (clusterId: string) => Promise<void>;
  clearError: () => void;
}

export const useProxmoxStore = create<ProxmoxState>((set, get) => ({
  clusters: [],
  activeClusterId: null,
  vms: {},
  metrics: {},
  loading: false,
  error: null,
  
  addCluster: async (cluster) => {
    // Implementation
  },
  
  // ... other actions
}));
```

---

## Implementation Phases

### Phase 1: Foundation (Week 1)

**Tasks:**
1. Create `src-tauri/src/proxmox/` module structure
2. Implement authentication flow (`proxmox/auth.rs`)
3. Create Proxmox API client (`proxmox/client.rs`)
4. Database migrations (012_proxmox_clusters)
5. Basic IPC commands (add/remove/list clusters)
6. Frontend: Cluster management UI

**TDD Tests:**
- Authentication flow
- API client request/response handling
- Credential encryption/decryption
- Cluster CRUD operations

### Phase 2: Proxmox VE Management (Week 2)

**Tasks:**
1. Implement VM management commands
2. Node status and metrics
3. Storage management (local, ZFS, Ceph)
4. **Ceph Management:**
   - Pool management (list, create, delete, quota)
   - OSD management (list, weight, out/in)
   - MDS management (list, failover)
   - RBD management (list, create, delete, resize, clone)
   - Ceph health monitoring
5. VM lifecycle operations (start/stop/reboot)
6. Frontend: VM manager interface

**TDD Tests:**
- VM listing and status
- VM lifecycle operations
- Node metrics collection
- Storage inventory
- Ceph pool/OSD/MDS/RBD operations
- Ceph health monitoring

### Phase 3: Proxmox Backup Server & Advanced Features (Week 3)

**Tasks:**
1. Implement PBS backup job management
2. Datastore management
3. Backup listing and restoration
4. **Backup Scheduling:**
   - Create/edit/delete backup jobs
   - Cron-style scheduling
   - Manual backup trigger
   - Backup replication between clusters
5. **SDN Management (Proxmox VE):**
   - List SDN zones, DHCP, firewall
6. **Firewall Management (Proxmox VE):**
   - List/add/delete firewall rules
   - Enable/disable firewall
7. **HA Group Management (Proxmox VE):**
   - List HA groups
   - Manage HA resources
   - Failover configuration
8. Frontend: Backup manager interface

**TDD Tests:**
- Backup job operations
- Datastore management
- Backup listing and filtering
- Restore operations
- Backup scheduling
- SDN zone management
- Firewall rule management
- HA group management

### Phase 4: Multi-Cluster & Cross-Datacenter (Week 4)

**Tasks:**
1. Implement cluster registry
2. Cross-cluster metrics aggregation
3. Live migration between clusters
4. **Cluster Selection UI:**
   - Dropdown with three modes:
     - Single cluster
     - Multiple clusters (multi-select)
     - ALL clusters
   - Visual indication of selected clusters
   - "Select All" checkbox
5. Dashboard with multi-cluster view
6. Frontend: Cluster selector and dashboard

**TDD Tests:**
- Cluster registry operations
- Cross-cluster metrics
- Live migration workflow
- Dashboard data aggregation
- Cluster selection (single/multi/all)

### Phase 5: Triage Integration (Week 5)

**Tasks:**
1. Link Proxmox resources to issues
2. Log collection from Proxmox
3. PII detection in Proxmox logs
4. Integration with existing triage workflow
5. Frontend: Resource linking UI

**TDD Tests:**
- Resource linking
- Log collection
- PII detection
- Issue-integration workflow

### Phase 6: Testing & Documentation (Week 6)

**Tasks:**
1. End-to-end testing
2. Performance optimization
3. Documentation
4. Release preparation

---

## Security Considerations

### 1. Credential Storage

**Current Practice (from Integrations):**
- Use `encrypt_token()` / `decrypt_token()` from `src-tauri/src/integrations/auth.rs`
- AES-256-GCM encryption with nonce
- Key derived from `TRCAA_ENCRYPTION_KEY` env var or auto-generated `.enckey` file
- SHA-256 hash for audit trail

**Proxmox Implementation:**
```rust
// Store root password (encrypted)
let encrypted_password = encrypt_token(&password)?;
db.execute(
    "INSERT INTO proxmox_clusters (..., encrypted_password, ...) VALUES (...)",
    rusqlite::params![..., encrypted_password, ...]
)?;

// Store API token (encrypted)
let encrypted_token = encrypt_token(&api_token)?;
db.execute(
    "INSERT INTO proxmox_credentials (..., api_token, token_hash, ...) VALUES (...)",
    rusqlite::params![..., encrypted_token, token_hash, ...]
)?;
```

### 2. SSL/TLS Verification

**Options:**
- **Strict (default):** Verify SSL fingerprint against configured value
- **Permissive:** Accept any certificate (for self-signed, common in Proxmox)
- **User Choice:** Configuration option per cluster

**Implementation:**
```rust
pub struct ProxmoxClusterConfig {
    pub name: String,
    pub node_address: String,
    pub username: String,
    pub encrypted_password: String,
    pub verify_fingerprint: bool,  // New field
    pub cluster_type: ClusterType,
}

// In client
if config.verify_fingerprint {
    let cert = get_certificate(&node_address)?;
    if cert.fingerprint() != config.node_fingerprint {
        return Err("SSL fingerprint mismatch".to_string());
    }
}
```

### 3. API Token Management

**Token Lifecycle:**
- Generated from root credentials
- Stored encrypted in database
- Cached in memory with expiry
- Auto-refresh before expiry

**Token Format:**
```
root!1686000000!abc123def456...!csrf789xyz
```

**Expiry Handling:**
```rust
// Check token expiry before API calls
if token.expires_at < chrono::Utc::now() {
    // Auto-refresh using stored credentials
    let new_token = client.refresh_token().await?;
    // Update database
}
```

### 4. Audit Logging

**Events to Log:**
- Cluster added/removed
- Authentication success/failure
- VM lifecycle operations
- Migration operations
- Backup operations

**Example:**
```rust
audit::log::write_audit_event(
    &db,
    "proxmox_vm_started",
    "proxmox_resource",
    &format!("{}:vm-{}", cluster_id, vm_id),
    &serde_json::json!({
        "cluster_id": cluster_id,
        "vm_id": vm_id,
        "username": username
    }).to_string(),
)
.map_err(|e| format!("Failed to log audit event: {e}"))?;
```

---

## Testing Strategy

### Unit Tests (Rust)

**Target Coverage:** 80%+

**Test Files:**
- `src-tauri/src/proxmox/tests/auth_tests.rs`
- `src-tauri/src/proxmox/tests/client_tests.rs`
- `src-tauri/src/proxmox/tests/cluster_tests.rs`
- `src-tauri/src/proxmox/tests/metrics_tests.rs`

**Test Approach:**
- HTTP mocking with `mockito`
- In-memory SQLite for database tests
- Property-based testing with `proptest`

### Integration Tests (Rust)

**Scenarios:**
1. Add cluster with valid credentials
2. Add cluster with invalid credentials
3. List VMs across multiple clusters
4. Start/stop VM
5. Live migration between clusters
6. Backup job execution

### Frontend Tests (TypeScript)

**Test Files:**
- `tests/unit/proxmox/cluster.test.ts`
- `tests/unit/proxmox/metrics.test.ts`
- `tests/unit/proxmox/vm-manager.test.ts`

**Test Approach:**
- Vitest for unit tests
- React Testing Library for component tests
- Mock Tauri IPC calls

### E2E Tests

**Scenarios:**
1. Full cluster setup workflow
2. VM management workflow
3. Cross-cluster migration
4. Backup and restore workflow

---

## Migration Strategy

### Database Migration (012_proxmox_clusters)

```rust
// src-tauri/src/db/migrations.rs

(
    "012_proxmox_clusters",
    r#"
    CREATE TABLE IF NOT EXISTS proxmox_clusters (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        node_address TEXT NOT NULL,
        node_fingerprint TEXT,
        username TEXT NOT NULL,
        encrypted_password TEXT NOT NULL,
        cluster_type TEXT NOT NULL CHECK(cluster_type IN ('ve', 'pbs')),
        status TEXT NOT NULL DEFAULT 'unknown',
        last_connected_at TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(name, node_address)
    );
    
    CREATE TABLE IF NOT EXISTS proxmox_resources (
        id TEXT PRIMARY KEY,
        cluster_id TEXT NOT NULL REFERENCES proxmox_clusters(id) ON DELETE CASCADE,
        resource_type TEXT NOT NULL,
        resource_id TEXT NOT NULL,
        name TEXT,
        status TEXT,
        cpu_usage REAL,
        memory_usage REAL,
        storage_usage REAL,
        details TEXT,
        last_updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(cluster_id, resource_type, resource_id)
    );
    
    CREATE TABLE IF NOT EXISTS proxmox_credentials (
        id TEXT PRIMARY KEY,
        cluster_id TEXT NOT NULL REFERENCES proxmox_clusters(id) ON DELETE CASCADE,
        api_token TEXT NOT NULL,
        token_hash TEXT NOT NULL,
        expires_at TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(cluster_id)
    );
    
    -- Add proxmox to integration_config
    ALTER TABLE integration_config ADD COLUMN proxmox_config TEXT;
    "#,
),
```

### Backward Compatibility

- Existing integrations (Confluence, ServiceNow, Azure DevOps) remain unchanged
- New tables are additive only
- No breaking changes to existing APIs

---

## Performance Considerations

### Caching Strategy

**In-Memory Caches:**
- Cluster clients (ProxmoxClient instances)
- VM status (5-second TTL)
- Node metrics (10-second TTL)
- Storage inventory (1-minute TTL)

**Database Caching:**
- Use SQLite's built-in caching
- Index on `cluster_id` for fast lookups
- Consider WAL mode for concurrent access

### API Rate Limiting

**Proxmox API Limits:**
- Default: 100 requests/minute per user
- Implement exponential backoff on rate limit errors

**Implementation:**
```rust
struct RateLimiter {
    requests: Mutex<Vec<Timestamp>>,
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    async fn acquire(&self) {
        loop {
            let mut requests = self.requests.lock().unwrap();
            let now = chrono::Utc::now();
            let window_start = now - self.window;
            
            // Remove old requests
            requests.retain(|&t| t > window_start);
            
            if requests.len() < self.limit as usize {
                requests.push(now);
                break;
            }
            
            drop(requests);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
```

---

## Error Handling

### Common Errors

**Authentication Errors:**
- Invalid credentials
- SSL fingerprint mismatch
- Certificate verification failed

**API Errors:**
- Network timeout
- Rate limit exceeded
- Resource not found
- Permission denied

**Cluster Errors:**
- Cluster unreachable
- Authentication expired
- API version mismatch

### User-Facing Messages

```typescript
// Good
"Failed to connect to Proxmox cluster 'pve-cluster': Invalid credentials"

// Better
"Authentication failed for cluster 'pve-cluster'. Please check your username and password."

// Best
"Unable to authenticate to 'pve-cluster'. Verify your root credentials are correct and the cluster is accessible at port 8006."
```

---

## Documentation Requirements

### User Documentation

**New Wiki Page:** `docs/wiki/Proxmox-Integration.md`

**Sections:**
1. Overview
2. Getting Started
3. Adding a Proxmox Cluster
4. Managing Virtual Machines
5. Managing Backups (PBS)
6. Cross-Datacenter Management
7. Live Migration
8. Troubleshooting
9. API Reference

### Developer Documentation

**Code Comments:**
- All public functions must have doc comments
- Complex logic must have inline comments

**Architecture Docs:**
- Update `docs/architecture/` with Proxmox integration
- Database schema documentation
- API client design

---

## Rollout Plan

### Pre-Release (Week 6)

**Checklist:**
- [ ] All tests passing (unit, integration, E2E)
- [ ] Code coverage >= 80%
- [ ] Documentation complete
- [ ] Changelog updated
- [ ] Version bumped to v1.2.0

### Release

**Steps:**
1. Create release branch `release/v1.2.0`
2. Update version in `Cargo.toml` and package.json
3. Run full test suite
4. Create GitHub release
5. Update documentation
6. Announce release

### Post-Release

**Monitoring:**
- Error tracking (if implemented)
- User feedback collection
- Performance monitoring

**Future Enhancements:**
- Email notifications for cluster issues
- Webhook integration for alerts
- Advanced HA management
- Custom dashboard widgets

---

## Success Criteria

### Functional Requirements

**Cluster Management:**
- [ ] Can add/remove multiple Proxmox clusters (VE and PBS)
- [ ] Default ports configured correctly (8006 for VE, 8007 for PBS)
- [ ] User can override port per cluster
- [ ] Cluster list shows all configured clusters
- [ ] Cluster selection dropdown (single/multi/all) works

**Authentication:**
- [ ] Authentication with root credentials works
- [ ] API token generation and storage works
- [ ] SSL fingerprint verification configurable
- [ ] Support for self-signed certificates

**Proxmox VE:**
- [ ] VM management operations work (start/stop/reboot/shutdown/suspend)
- [ ] Ceph management works:
  - [ ] Pool management (list, create, delete, quota)
  - [ ] OSD management (list, weight, out/in)
  - [ ] MDS management (list, failover)
  - [ ] RBD management (list, create, delete, resize, clone)
  - [ ] Ceph health monitoring
- [ ] SDN management works (zones, DHCP, firewall)
- [ ] Firewall management works (rules, enable/disable)
- [ ] HA group management works
- [ ] Storage management (local, ZFS, Ceph)

**Proxmox Backup Server:**
- [ ] PBS backup operations work
- [ ] Backup scheduling works (create/edit/delete jobs)
- [ ] Manual backup trigger works
- [ ] Backup restoration works
- [ ] Backup replication between clusters works
- [ ] Deduplication status monitoring works

**Cross-Datacenter:**
- [ ] Cross-cluster metrics display correctly
- [ ] Live migration between clusters works
- [ ] Global dashboard shows all clusters
- [ ] Cross-cluster search works

**Triage Integration:**
- [ ] Triage integration works (link resources, collect logs)
- [ ] PII detection in Proxmox logs

### Non-Functional Requirements

- [ ] All credentials encrypted at rest
- [ ] SSL/TLS verification configurable
- [ ] Performance: < 2s for cluster status refresh
- [ ] Performance: < 5s for VM list (100 VMs)
- [ ] Tests: >= 80% code coverage
- [ ] Tests: All critical paths covered
- [ ] Documentation: User and developer docs complete

---

## Risk Assessment

### Technical Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Proxmox API changes | High | Low | Abstract API layer, version checking |
| SSL/TLS complexity | Medium | Medium | Provide clear config options |
| Performance at scale | Medium | Low | Caching, rate limiting, pagination |
| Multi-cluster complexity | High | Medium | Modular design, clear separation |

### Schedule Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| API discovery delays | Medium | Medium | Start with documentation research |
| Testing complexity | Medium | Medium | TDD approach, mock server |
| Integration issues | Low | Low | Incremental implementation |

---

## Conclusion

This plan provides a comprehensive roadmap for implementing Proxmox integration into TRCAA v1.2.0. The approach emphasizes:

1. **Security:** Encrypted credentials, SSL verification, audit logging
2. **Flexibility:** Support for both VE and PBS, multi-cluster management
3. **User Experience:** Intuitive UI, cross-datacenter visibility
4. **Maintainability:** Clean architecture, comprehensive tests, documentation

The phased approach allows for incremental delivery and validation at each stage, reducing risk and enabling early feedback.
