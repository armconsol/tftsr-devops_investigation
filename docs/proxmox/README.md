# Proxmox Integration Documentation

This directory contains documentation for the Proxmox integration into TRCAA.

## Documentation Files

### Overview

- **`IMPLEMENTATION_SUMMARY.md`** - High-level summary of the implementation plan
- **`QUICK_REFERENCE.md`** - Quick reference card for developers
- **`TICKET-proxmox-integration.md`** - Complete implementation plan with technical details

### Implementation Phases

- **Phase 1** - Foundation (Week 1)
- **Phase 2** - Proxmox VE Management (Week 2)
- **Phase 3** - Proxmox Backup Server (Week 3)
- **Phase 4** - Multi-Cluster & Cross-Datacenter (Week 4)
- **Phase 5** - Triage Integration (Week 5)
- **Phase 6** - Testing & Documentation (Week 6)

## Quick Start

### For Developers

1. Review `QUICK_REFERENCE.md` for API endpoints and IPC commands
2. Read `TICKET-proxmox-integration.md` for complete technical details
3. Follow implementation phases in order
4. Write tests first (TDD approach)
5. Run `cargo test` and `npm run test` after each phase

### For Users

See the user-facing documentation in `docs/wiki/Proxmox-Integration.md` (to be created during Phase 6).

## Implementation Checklist

- [ ] Phase 1: Foundation
  - [ ] Create `src-tauri/src/proxmox/` module
  - [ ] Implement authentication flow
  - [ ] Create Proxmox API client
  - [ ] Database migrations
  - [ ] Basic IPC commands
  - [ ] Frontend: Cluster management UI

- [ ] Phase 2: Proxmox VE Management
  - [ ] VM management commands
  - [ ] Node status and metrics
  - [ ] Storage management
  - [ ] VM lifecycle operations
  - [ ] Frontend: VM manager interface

- [ ] Phase 3: Proxmox Backup Server
  - [ ] Backup job management
  - [ ] Datastore management
  - [ ] Backup listing and restoration
  - [ ] Frontend: Backup manager interface

- [ ] Phase 4: Multi-Cluster & Cross-Datacenter
  - [ ] Cluster registry
  - [ ] Cross-cluster metrics aggregation
  - [ ] Live migration between clusters
  - [ ] Dashboard with multi-cluster view

- [ ] Phase 5: Triage Integration
  - [ ] Link Proxmox resources to issues
  - [ ] Log collection from Proxmox
  - [ ] PII detection in Proxmox logs
  - [ ] Integration with existing triage workflow

- [ ] Phase 6: Testing & Documentation
  - [ ] End-to-end testing
  - [ ] Performance optimization
  - [ ] User documentation
  - [ ] Developer documentation
  - [ ] Release preparation

## Testing

### Rust Tests

```bash
# Run all Proxmox tests
cargo test --manifest-path src-tauri/Cargo.toml --lib proxmox

# Test coverage
cargo test --manifest-path src-tauri/Cargo.toml --lib proxmox -- --test-threads=1
```

### Frontend Tests

```bash
# Unit tests
npm run test -- proxmox

# Coverage
npm run test:coverage -- proxmox
```

## References

- **Proxmox API Docs:** https://pve.proxmox.com/pve-docs/api-viewer/
- **Proxmox Datacenter Manager:** https://github.com/proxmox/proxmox-datacenter-manager
- **TRCAA Integrations Pattern:** `docs/wiki/Integrations.md`

## Questions?

See `TICKET-proxmox-integration.md` for detailed technical information or contact the development team.
