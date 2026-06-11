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

