# Release v1.2.0

**Release Date**: 2026-06-11  
**Commit**: 446ebf95  
**Status**: Production-ready with Proxmox Datacenter Manager feature parity

## Overview

v1.2.0 introduces 100% Proxmox Datacenter Manager (PDM) feature parity, enabling full cluster management for Proxmox VE and Backup Server directly within the application. This release also includes critical bug fixes and navigation improvements.

## Changes since v1.1.0

### Proxmox Datacenter Manager Feature Parity

**New Features**:
- 100% Proxmox Datacenter Manager (PDM) feature parity implemented
- Multi-cluster management (Proxmox VE and Backup Server)
- VM lifecycle management (start/stop/reboot/shutdown/migrate)
- Ceph cluster management (pools, OSDs, MDS, RBD, health)
- SDN management (EVPN zones, virtual networks)
- Firewall management (rules, zones, enable/disable)
- HA groups management (groups, resources, failover)
- Update management (check, list, install updates)
- User management (LDAP, Active Directory, OpenID Connect)
- ACME/Let's Encrypt certificate management
- Remote shell access (PTY-based terminals)
- Dashboard with 13 widget types
- Live migration between clusters

**Proxmox Cluster Management**:
- Add, edit, and remove Proxmox clusters via UI
- Persistent cluster storage with SQLCipher AES-256 encryption
- Connection caching for improved performance
- SSL certificate verification options
- Connection timeout and retry configuration

**Navigation Improvements**:
- Proxmox submenu with 12 management pages
- Settings page with update channel selection (stable/pre-release)
- Auto-update check and download configuration

**Technical Implementation**:
- 22 Rust backend modules in `src-tauri/src/proxmox/`
- 33 React components in `src/components/Proxmox/`
- 14 Proxmox management pages in `src/pages/Proxmox/`
- Database persistence with SQLCipher AES-256 encryption
- 406 Rust unit tests + 386 frontend tests

### Bug Fixes

- Fixed cluster save functionality (mock data → IPC calls)
- Added Proxmox settings section to Settings navigation
- Implemented Proxmox submenu navigation with expandable section
- Fixed Proxmox cluster connection caching issues

### Documentation Updates

- Updated all Proxmox documentation for v1.2.0
- Added Proxmox feature parity completion summary
- Updated CHANGELOG.md for v1.2.0 release

## Changes since v1.1.0

See v1.1.0 release notes for v1.1.0 → v1.1.0 changes.

---

# Release v1.1.0

**Release Date**: 2026-06-06  
**Commit**: 21758cfd  
**Status**: Production-ready with Kubernetes Management UI

## Overview

v1.1.0 introduces the Kubernetes Management UI with FreeLens parity, enabling full cluster management directly within the application. This release also includes critical bug fixes and documentation updates for the v1.0.0 Shell Execution feature.

## Changes since v1.0.1

### Kubernetes Management UI (FreeLens Parity)

**New Features**:
- PTY-based interactive terminals with real-time shell access
- Cluster metrics dashboard (nodes, pods, resources)
- Port forwarding with local binding and URL generation
- Inline YAML editor with syntax highlighting
- Multi-cluster kubeconfig management
- Real-time log streaming with filter support
- Resource visualization (CPU, memory, replica counts)

**Technical Implementation**:
- WebSocket-based terminal connections (pty, stdout, stderr, resize)
- Metrics collection via kubectl API (nodes, pods, namespaces)
- Port forwarding via `kubectl port-forward` with auto-allocated ports
- YAML validation and linting before apply/delete operations
- AES-256-GCM encrypted kubeconfig storage per cluster

### Bug Fixes

- Fixed kubeconfig context switching in multi-cluster environments
- Corrected domain prompt count from 17 to 15 in documentation
- Fixed CI/CD references from GitHub to Gitea Actions
- Updated CHANGELOG.md for v1.1.0 release

### Documentation Updates

- Updated all CI/CD references from `.github/workflows/` to `.gitea/workflows/`
- Updated release notes and wiki to reflect v1.1.0 features
- Removed completed features from Future Enhancements sections

## Changes since v1.0.0

See v1.0.1 release notes for v1.0.0 → v1.0.1 changes.

---

# Release v1.0.1

This release ensures the domain prompt fix is cleanly packaged.

## Changes since v1.0.0
- Domain prompts now instruct AI to use execute_shell_command tool
- UI contrast improvements for kubeconfig file upload
- ARM64 Linux build fix

