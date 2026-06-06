# Proxmox Integration - Quick Reference

**Version:** v1.2.0  
**Status:** Planning ✓ | Implementation: Pending

---

## Core Concepts

### Port Configuration

| Service | Default Port | API Endpoint |
|---------|--------------|--------------|
| Proxmox VE | **8006** | `https://hostname:8006/api2/json` |
| Proxmox Backup Server | **8007** | `https://hostname:8007/api2/json` |

**Implementation:**
- Default port set by cluster type (8006 for VE, 8007 for PBS)
- User can override port if needed
- Port displayed in cluster configuration UI

### Authentication Flow

```
User Input → Root Credentials → Proxmox API → API Token → Encrypted Storage
     ↓
SSL Fingerprint Verification (Optional)
```

### Data Flow

```
Proxmox Cluster (port 8006 for VE, 8007 for PBS)
    ↓ HTTPS API
ProxmoxClient (cached in memory)
    ↓ Encrypted Token
Database (SQLite + AES-256-GCM)
```

---

## Key Files

### Backend

| File | Purpose |
|------|---------|
| `src-tauri/src/proxmox/mod.rs` | Module exports |
| `src-tauri/src/proxmox/client.rs` | Proxmox API client |
| `src-tauri/src/proxmox/auth.rs` | Authentication logic |
| `src-tauri/src/proxmox/cluster.rs` | Cluster registry |
| `src-tauri/src/proxmox/models.rs` | Data models |
| `src-tauri/src/commands/proxmox.rs` | IPC commands |
| `src-tauri/src/db/migrations.rs` | DB schema (migration 012) |

### Frontend

| File | Purpose |
|------|---------|
| `src/pages/Proxmox/index.tsx` | Main page |
| `src/pages/Proxmox/ClusterList.tsx` | Cluster management |
| `src/pages/Proxmox/ClusterDashboard.tsx` | Metrics dashboard |
| `src/pages/Proxmox/VMManager.tsx` | VM operations |
| `src/pages/Proxmox/AddClusterModal.tsx` | Add cluster UI |
| `src/lib/tauriCommands.ts` | IPC wrappers |
| `src/stores/proxmoxStore.ts` | State management |

---

## Database Schema

### New Tables

**proxmox_clusters**
```sql
id TEXT PRIMARY KEY
name TEXT NOT NULL
node_address TEXT NOT NULL  -- hostname:8006
node_fingerprint TEXT       -- SSL cert hash
username TEXT NOT NULL      -- root
encrypted_password TEXT NOT NULL
cluster_type TEXT CHECK('ve' OR 'pbs')
status TEXT DEFAULT 'unknown'
last_connected_at TEXT
created_at TEXT
updated_at TEXT
```

**proxmox_resources**
```sql
id TEXT PRIMARY KEY
cluster_id TEXT NOT NULL
resource_type TEXT          -- 'node', 'vm', 'ct', 'storage', 'backup'
resource_id TEXT            -- VM ID, storage ID
name TEXT
status TEXT
cpu_usage REAL
memory_usage REAL
storage_usage REAL
details TEXT                -- JSON blob
last_updated_at TEXT
```

**proxmox_credentials**
```sql
id TEXT PRIMARY KEY
cluster_id TEXT NOT NULL
api_token TEXT NOT NULL     -- Encrypted API token
token_hash TEXT NOT NULL    -- SHA-256 for audit
expires_at TEXT
created_at TEXT
```

---

## API Endpoints

### Authentication

```
POST /api2/json/access/ticket
Request: { username: "root", password: "..." }
Response: { ticket: "PVE@pam!root!...", CSRFPreventionToken: "..." }
```

### Proxmox VE

```
GET  /api2/json/nodes              - List nodes
GET  /api2/json/nodes/{node}/qemu - List VMs
GET  /api2/json/nodes/{node}/qemu/{vmid}/status/current - Get VM status
POST /api2/json/nodes/{node}/qemu/{vmid}/status/start   - Start VM
POST /api2/json/nodes/{node}/qemu/{vmid}/status/stop    - Stop VM
POST /api2/json/nodes/{node}/qemu/{vmid}/status/reboot  - Reboot VM
POST /api2/json/nodes/{node}/qemu/{vmid}/migrate        - Migrate VM
GET  /api2/json/nodes/{node}/storage  - List storage
GET  /api2/json/cluster/resources     - Cluster resources

### Ceph Management

```
GET  /api2/json/nodes/{node}/ceph/pool   - List pools
POST /api2/json/nodes/{node}/ceph/pool   - Create pool
DELETE /api2/json/nodes/{node}/ceph/pool/{pool} - Delete pool
GET  /api2/json/nodes/{node}/ceph/osd    - List OSDs
POST /api2/json/nodes/{node}/ceph/osd/{id}/set - Set OSD weight
POST /api2/json/nodes/{node}/ceph/osd/{id}/out - Set OSD out
POST /api2/json/nodes/{node}/ceph/osd/{id}/in - Set OSD in
GET  /api2/json/nodes/{node}/ceph/mds    - List MDS
POST /api2/json/nodes/{node}/ceph/mds/{id}/failover - MDS failover
GET  /api2/json/nodes/{node}/ceph/rbd    - List RBDs
POST /api2/json/nodes/{node}/ceph/rbd   - Create RBD
DELETE /api2/json/nodes/{node}/ceph/rbd/{pool}/{name} - Delete RBD
PUT  /api2/json/nodes/{node}/ceph/rbd/{pool}/{name} - Resize RBD
GET  /api2/json/cluster/ceph/status      - Ceph status
GET  /api2/json/cluster/ceph/health      - Ceph health
```

### SDN Management

```
GET  /api2/json/nodes/{node}/sdn/zones   - List SDN zones
GET  /api2/json/nodes/{node}/sdn/dhcp    - List SDN DHCP
GET  /api2/json/nodes/{node}/sdn/firewall - List SDN firewall
```

### Firewall Management

```
GET  /api2/json/nodes/{node}/firewall/rules  - List firewall rules
POST /api2/json/nodes/{node}/firewall/rules  - Add firewall rule
DELETE /api2/json/nodes/{node}/firewall/rules/{ruleid} - Delete firewall rule
POST /api2/json/nodes/{node}/firewall/status - Enable firewall
DELETE /api2/json/nodes/{node}/firewall/status - Disable firewall
```

### HA Group Management

```
GET  /api2/json/cluster/ha/resources     - List HA resources
GET  /api2/json/cluster/ha/groups        - List HA groups
POST /api2/json/cluster/ha/groups        - Create HA group
DELETE /api2/json/cluster/ha/groups/{group} - Delete HA group
POST /api2/json/cluster/ha/resources/{rid} - Manage HA resource
```

### Proxmox Backup Server

```
GET  /api2/json/nodes/{node}/backup - List backups
POST /api2/json/nodes/{node}/backup/{jobid}/run - Run backup job
GET  /api2/json/nodes/{node}/storage - List datastores
GET  /api2/json/nodes/{node}/backup/status - Backup status

### Backup Scheduling & Replication

```
POST /api2/json/nodes/{node}/backup/{jobid} - Create/edit backup job
DELETE /api2/json/nodes/{node}/backup/{jobid} - Delete backup job
POST /api2/json/nodes/{node}/backup/restore - Restore backup
GET  /api2/json/nodes/{node}/backup/replication - List replication status
POST /api2/json/nodes/{node}/backup/replication - Trigger replication
```

---

## IPC Commands

### Cluster Management

```typescript
addProxmoxClusterCmd(config)
removeProxmoxClusterCmd(clusterId)
listProxmoxClustersCmd()
getProxmoxClusterCmd(clusterId)
testProxmoxConnectionCmd(config)
```

### VM Operations

```typescript
listProxmoxVMsCmd(clusterId)
startProxmoxVMCmd(clusterId, vmId)
stopProxmoxVMCmd(clusterId, vmId)
rebootProxmoxVMCmd(clusterId, vmId)
shutdownProxmoxVMCmd(clusterId, vmId)
suspendProxmoxVMCmd(clusterId, vmId)
cloneProxmoxVMCmd(clusterId, vmId, newId, name)
migrateProxmoxVMCmd(clusterId, vmId, targetClusterId, online)
```

### PBS Operations

```typescript
listProxmoxBackupsCmd(clusterId)
runProxmoxBackupJobCmd(clusterId, jobId)
listProxmoxDatastoresCmd(clusterId)
restoreProxmoxBackupCmd(clusterId, backupId, datastore)
```

### Metrics

```typescript
getProxmoxMetricsCmd(clusterId)
getCrossClusterMetricsCmd()
```

### Triage Integration

```typescript
linkProxmoxResourceCmd(issueId, clusterId, resourceType, resourceId)
collectProxmoxLogsCmd(issueId, clusterId, resourceType, resourceId, timeRange)
```

---

## Configuration

### Environment Variables

```bash
# Encryption key (auto-generated if not set)
TRCAA_ENCRYPTION_KEY=<32-byte-hex-key>

# Optional: Proxmox-specific config
PROXMOX_DEFAULT_PORT=8006
PROXMOX_DEFAULT_TIMEOUT=30
PROXMOX_ENABLE_SSL_VERIFY=true
```

### Cluster Configuration (JSON)

```json
{
  "name": "pve-cluster-1",
  "node_address": "pve1.example.com:8006",
  "node_fingerprint": "SHA256:ABC123...",
  "username": "root",
  "encrypted_password": "base64(gcm-encrypted-password)",
  "cluster_type": "ve"
}
```

---

## Security Checklist

- [ ] All passwords encrypted with AES-256-GCM
- [ ] API tokens stored encrypted
- [ ] SSL fingerprint verification configurable
- [ ] Audit logging for all operations
- [ ] No credentials in logs
- [ ] CSRF tokens handled properly
- [ ] Rate limiting implemented
- [ ] Error messages don't leak sensitive info

---

## Testing Strategy

### Rust Tests

```bash
# Run all Proxmox tests
cargo test --manifest-path src-tauri/Cargo.toml --lib proxmox

# Run specific test module
cargo test --manifest-path src-tauri/Cargo.toml -- lib proxmox::client

# Test coverage
cargo test --manifest-path src-tauri/Cargo.toml --lib proxmox -- --test-threads=1 --nocapture
```

### Frontend Tests

```bash
# Unit tests
npm run test -- proxmox

# Coverage
npm run test:coverage -- proxmox
```

### E2E Tests

```bash
# Full integration
npm run test:e2e
```

---

## Common Tasks

### Add New Cluster

1. Call `addProxmoxClusterCmd(config)`
2. Backend validates credentials
3. Generates API token
4. Stores encrypted credentials
5. Returns success/error

### List VMs

1. Call `listProxmoxVMsCmd(clusterId)`
2. Client authenticates (if needed)
3. Calls Proxmox API
4. Returns VM list

### Start VM

1. Call `startProxmoxVMCmd(clusterId, vmId)`
2. Client validates authentication
3. Calls Proxmox API
4. Returns task status

### Live Migration

1. Call `migrateProxmoxVMCmd(sourceClusterId, vmId, targetClusterId, online)`
2. Validates both clusters
3. Creates migration task
4. Returns task ID for polling

---

## Troubleshooting

### Common Issues

**"SSL fingerprint mismatch"**
- Verify cluster SSL certificate
- Disable fingerprint verification for self-signed certs

**"Authentication failed"**
- Verify root credentials
- Check Proxmox API is accessible on port 8006
- Ensure user has proper permissions

**"Rate limit exceeded"**
- Implement exponential backoff
- Reduce request frequency
- Use caching

**"Cluster unreachable"**
- Verify network connectivity
- Check firewall rules
- Ensure Proxmox service is running

---

## Performance Targets

| Operation | Target Latency | Max Data |
|-----------|---------------|----------|
| Cluster list | < 1s | 50 clusters |
| VM list | < 2s | 100 VMs |
| VM status | < 500ms | N/A |
| Metrics refresh | < 5s | 10 nodes |
| Migration | < 10s | N/A |

---

## Next Steps

1. ✅ **Planning complete** - This document
2. ⏳ **Phase 1** - Foundation (Week 1)
3. ⏳ **Phase 2** - VE Management (Week 2)
4. ⏳ **Phase 3** - PBS Support (Week 3)
5. ⏳ **Phase 4** - Cross-Datacenter (Week 4)
6. ⏳ **Phase 5** - Triage Integration (Week 5)
7. ⏳ **Phase 6** - Testing & Docs (Week 6)

---

## Resources

- **Proxmox API Docs:** https://pve.proxmox.com/pve-docs/api-viewer/
- **Proxmox Datacenter Manager:** https://github.com/proxmox/proxmox-datacenter-manager
- **TRCAA Architecture:** `docs/architecture/`
- **Integration Patterns:** `docs/wiki/Integrations.md`

---

**Document Version:** 1.0  
**Last Updated:** 2026-06-06  
**Author:** AI Assistant  
**Review Status:** Pending
