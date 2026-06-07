# Proxmox Integration - Implementation Summary

## Overview

This document summarizes the implementation plan for adding Proxmox integration to the TRCAA application (v1.2.0).

## What Was Planned

### Core Features

1. **Multi-Cluster Management** - Support for multiple Proxmox clusters (both VE and PBS)
2. **Cross-Datacenter Metrics** - Unified dashboard across all clusters
3. **Full VM Management** - Start/stop/reboot/migrate operations
4. **Backup Management** - PBS job and backup management
5. **Live Migration** - VM migration between clusters
6. **Triage Integration** - Link Proxmox resources to issues and collect logs

## Critical Corrections (Based on User Feedback)

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

### Cluster Selection UI

**Requirement:** Dropdown with three selection modes:

| Mode | Description | Use Case |
|------|-------------|----------|
| **Single Cluster** | Select one specific cluster | Targeted operations on one cluster |
| **Multiple Clusters** | Select 2+ specific clusters | Cross-cluster operations |
| **ALL Clusters** | All configured clusters | Global operations, dashboard |

### Authentication

- Root username/password authentication to Proxmox nodes (port 8006)
- Automatic API token generation and management
- Encrypted credential storage using AES-256-GCM
- SSL fingerprint verification (configurable)
- Support for self-signed certificates

### Technical Approach

**Backend:**
- New module: `src-tauri/src/proxmox/`
- API client with proper authentication flow
- Cluster registry for multi-cluster support
- Metrics aggregation across clusters
- Database migrations for new schema

**Frontend:**
- New sidebar item: "Proxmox"
- Cluster selector and management UI
- VM manager interface
- Backup manager interface
- Cross-cluster dashboard
- State management with Zustand

## Files Created

### Documentation

1. **`docs/TICKET-proxmox-integration.md`** (27 KB)
   - Complete implementation plan
   - Architecture details
   - Implementation phases (6 weeks)
   - Testing strategy
   - Security considerations
   - Risk assessment

2. **`docs/PROXMOX-QUICK-REFERENCE.md`** (8 KB)
   - Quick reference card
   - API endpoints
   - IPC commands
   - Common tasks
   - Troubleshooting guide

## Key Decisions

### 1. Authentication Method

**Decision:** Use root credentials + port 8006 (VE) / 8007 (PBS)

**Rationale:**
- Simpler than Proxmox Datacenter Manager setup
- No additional network configuration required
- Works in all environments
- Aligns with user's feedback
- Default ports set by cluster type, user can override

### 2. Credential Storage

**Decision:** Store root credentials encrypted, generate API tokens

**Rationale:**
- Consistent with existing integration patterns
- Uses `encrypt_token()` from `src-tauri/src/integrations/auth.rs`
- API tokens provide better security than storing passwords
- Token auto-refresh before expiry

### 3. Multi-Cluster Support

**Decision:** Full multi-cluster support (primary feature)

**Rationale:**
- Key selling point of Proxmox Datacenter Manager
- Enables cross-datacenter management
- Supports active/standby architectures
- Allows unified monitoring

### 4. UI Location

**Decision:** New sidebar item (not settings tab)

**Rationale:**
- Proxmox is a core feature, not just configuration
- Similar to Kubernetes integration
- Easy access for daily operations
- Dashboard potential

## Implementation Phases

| Phase | Duration | Focus | Deliverables |
|-------|----------|-------|--------------|
| 1 | Week 1 | Foundation | Auth flow, API client, DB schema |
| 2 | Week 2 | VE Management | VM operations, node status, **Ceph management** |
| 3 | Week 3 | PBS + Advanced | Backup jobs, **SDN, Firewall, HA groups** |
| 4 | Week 4 | Cross-Datacenter | Cluster registry, metrics, **cluster selector UI** |
| 5 | Week 5 | Triage Integration | Resource linking, log collection |
| 6 | Week 6 | Testing & Docs | Tests, documentation, release |

## TDD Compliance

### Rust Tests

- **Target Coverage:** 80%+
- **Test Files:**
  - `src-tauri/src/proxmox/tests/auth_tests.rs`
  - `src-tauri/src/proxmox/tests/client_tests.rs`
  - `src-tauri/src/proxmox/tests/cluster_tests.rs`
  - `src-tauri/src/proxmox/tests/metrics_tests.rs`
- **Approach:** TDD with mockito for HTTP mocking

### Frontend Tests

- **Unit Tests:** Vitest, 80%+ coverage
- **Component Tests:** React Testing Library
- **E2E Tests:** WebdriverIO for critical paths

## Security Considerations

### Encryption

- **Passwords:** AES-256-GCM encrypted
- **API Tokens:** AES-256-GCM encrypted
- **Key Source:** `TRCAA_ENCRYPTION_KEY` env var or auto-generated `.enckey`

### Audit Logging

- Cluster add/remove
- Authentication events
- VM lifecycle operations
- Migration operations
- Backup operations

### SSL/TLS

- Fingerprint verification (configurable)
- Support for self-signed certificates
- Certificate pinning option

## Database Changes

### New Tables

1. **proxmox_clusters** - Store cluster configuration
2. **proxmox_resources** - Cache resource status
3. **proxmox_credentials** - Store API tokens

### Migration

- File: `src-tauri/src/db/migrations.rs`
- Number: 012_proxmox_clusters
- Type: Additive (no breaking changes)

## Integration Points

### Existing Patterns

- **Authentication:** Use `src-tauri/src/integrations/auth.rs`
- **Encryption:** Use `encrypt_token()` / `decrypt_token()`
- **Audit:** Use `src-tauri/src/audit/log.rs`
- **IPC:** Follow `src-tauri/src/commands/integrations.rs` pattern

### New Patterns

- **Cluster Registry:** Manage multiple client connections
- **Metrics Aggregation:** Cross-cluster data collection
- **Live Migration:** Multi-cluster coordination

## Success Criteria

### Functional

**Cluster Management:**
- [ ] Add/remove multiple clusters (VE and PBS)
- [ ] Default ports configured correctly (8006 for VE, 8007 for PBS)
- [ ] User can override port per cluster
- [ ] Cluster selection dropdown (single/multi/all) works

**Authentication:**
- [ ] Authentication with root credentials
- [ ] API token generation and storage
- [ ] SSL fingerprint verification configurable

**Proxmox VE:**
- [ ] VM management operations
- [ ] Ceph management (pools, OSDs, MDS, RBD, health)
- [ ] SDN management (zones, DHCP, firewall)
- [ ] Firewall management (rules, enable/disable)
- [ ] HA group management

**Proxmox Backup Server:**
- [ ] PBS backup operations
- [ ] Backup scheduling (create/edit/delete jobs)
- [ ] Manual backup trigger
- [ ] Backup restoration
- [ ] Backup replication between clusters

**Cross-Datacenter:**
- [ ] Cross-cluster metrics
- [ ] Live migration between clusters
- [ ] Global dashboard

**Triage Integration:**
- [ ] Triage integration (link resources, collect logs)

### Non-Functional

- [ ] ≥80% code coverage
- [ ] <2s cluster status refresh
- [ ] <5s VM list (100 VMs)
- [ ] All credentials encrypted
- [ ] Documentation complete

## Next Steps

1. **Review Plan** - User reviews documentation
2. **Clarify Requirements** - Address any questions
3. **Begin Implementation** - Phase 1 (Week 1)
4. **TDD Approach** - Write tests first, then implementation
5. **Iterate** - Phases 2-6
6. **Release** - v1.2.0

## Questions for User

Before implementation begins, please confirm:

1. **Authentication Flow** - Root credentials → API token ✓ (Confirmed)
2. **Cluster Support** - Both VE and PBS ✓ (Confirmed)
3. **Multi-Cluster** - Full support with cross-datacenter ✓ (Confirmed)
4. **UI Location** - Sidebar item ✓ (Confirmed)
5. **Credential Storage** - Encrypted in database ✓ (Confirmed)
6. **Version** - v1.2.0 ✓ (Confirmed)

## References

- **Proxmox API:** https://pve.proxmox.com/pve-docs/api-viewer/
- **Proxmox Datacenter Manager:** https://github.com/proxmox/proxmox-datacenter-manager
- **TRCAA Integrations:** `docs/wiki/Integrations.md`
- **Architecture Docs:** `docs/architecture/`

---

**Document Version:** 1.0  
**Date:** 2026-06-06  
**Status:** Planning Complete - Ready for Implementation  
**Next Action:** User approval to begin Phase 1
