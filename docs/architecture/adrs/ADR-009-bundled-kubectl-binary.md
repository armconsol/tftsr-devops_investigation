# ADR-009: Bundle kubectl Binary for Cross-Platform Consistency

**Date**: 2026-06-02  
**Status**: Accepted  
**Deciders**: Shaun Arman, RJ Cooper  
**Context**: Hackathon v1.0.0 — Shell Execution System

---

## Context

TRCAA v1.0.0 introduced `execute_shell_command` tool for AI agents, with kubectl as a primary use case (diagnosing Kubernetes pod failures, checking deployments, viewing logs). kubectl is a critical tool for IT troubleshooting but has several challenges:

**Problems with system kubectl**:
- Version skew: User's kubectl may be v1.25 while cluster is v1.30 (API changes)
- Not installed: Many Windows/macOS users don't have kubectl
- PATH issues: kubectl in non-standard location (WSL, Homebrew, Chocolatey)
- Permission issues: System kubectl may require admin rights on Windows
- Configuration drift: `~/.kube/config` may be misconfigured or missing

**Requirements**:
- AI agents need reliable kubectl execution across all platforms
- Users should not need to install kubectl separately
- kubectl version should be consistent (no version skew errors)
- Work with multiple kubeconfig files (dev, staging, prod clusters)

**Alternatives Considered**:

1. **Use system kubectl (require manual install)**
   - ✅ No binary bundling needed
   - ❌ Poor UX — user must install kubectl separately
   - ❌ Version skew issues
   - ❌ PATH configuration required
   - ❌ Windows complexity (WSL vs native)

2. **Download kubectl at runtime (first use)**
   - ✅ No bloat in installer
   - ✅ Always latest version
   - ❌ Requires internet on first run
   - ❌ Download failure = broken feature
   - ❌ Security risk (MITM, checksum verification)

3. **Bundle kubectl as resource file**
   - ✅ Works offline
   - ✅ Consistent version
   - ✅ No user setup required
   - ❌ Increases installer size (~50MB per platform)
   - ❌ Need to update kubectl periodically

4. **Kubernetes client library (k8s-openapi crate)**
   - ✅ No binary needed
   - ✅ Native Rust implementation
   - ❌ Complex API (YAML → Rust types)
   - ❌ Doesn't support `kubectl apply -f` directly
   - ❌ No support for kubectl plugins
   - ❌ AI agents know kubectl CLI, not k8s-openapi API

---

## Decision

**Bundle kubectl v1.30.0 binary for all platforms (Linux amd64/arm64, macOS arm64/Intel, Windows amd64) as a Tauri resource.**

### Implementation

**Build-time binary download**: `scripts/download-kubectl.sh`

```bash
#!/bin/bash
VERSION="1.30.0"
OS=$1  # linux, darwin, windows
ARCH=$2  # amd64, arm64

curl -LO "https://dl.k8s.io/release/v${VERSION}/bin/${OS}/${ARCH}/kubectl"
chmod +x kubectl
mv kubectl "binaries/kubectl-${OS}-${ARCH}"
```

**CI/CD Integration**: `.github/workflows/release.yml`

```yaml
- name: Download kubectl binaries
  run: |
    ./scripts/download-kubectl.sh linux amd64
    ./scripts/download-kubectl.sh linux arm64
    ./scripts/download-kubectl.sh darwin arm64
    ./scripts/download-kubectl.sh darwin amd64
    ./scripts/download-kubectl.sh windows amd64
```

**Tauri Resource Bundling**: `src-tauri/tauri.conf.json`

```json
{
  "tauri": {
    "bundle": {
      "resources": [
        "binaries/kubectl-*"
      ]
    }
  }
}
```

**Runtime Binary Extraction**: `src-tauri/src/shell/kubectl.rs`

```rust
pub fn get_kubectl_path() -> Result<PathBuf, String> {
    let resource_dir = tauri::api::path::resource_dir(...)
        .ok_or("Failed to get resource directory")?;
    
    #[cfg(target_os = "linux")]
    let binary_name = if cfg!(target_arch = "aarch64") {
        "kubectl-linux-arm64"
    } else {
        "kubectl-linux-amd64"
    };
    
    #[cfg(target_os = "macos")]
    let binary_name = if cfg!(target_arch = "aarch64") {
        "kubectl-darwin-arm64"
    } else {
        "kubectl-darwin-amd64"
    };
    
    #[cfg(target_os = "windows")]
    let binary_name = "kubectl-windows-amd64.exe";
    
    let kubectl_path = resource_dir.join(binary_name);
    
    // Ensure executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&kubectl_path)
            .map_err(|e| format!("kubectl binary not found: {e}"))?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&kubectl_path, perms)?;
    }
    
    Ok(kubectl_path)
}
```

**Execution with Custom Kubeconfig**: `src-tauri/src/shell/executor.rs`

```rust
pub async fn execute_kubectl(command: &str, kubeconfig_id: Option<String>) -> Result<Output> {
    let kubectl_path = kubectl::get_kubectl_path()?;
    
    let mut cmd = Command::new(kubectl_path);
    
    // Inject kubeconfig if provided
    if let Some(id) = kubeconfig_id {
        let kubeconfig = kubeconfig::get_and_decrypt(id)?;
        let temp_path = write_temp_kubeconfig(kubeconfig)?;
        cmd.env("KUBECONFIG", temp_path);
    }
    
    cmd.args(command.split_whitespace());
    cmd.output().await
}
```

### Version Selection Rationale

**kubectl v1.30.0** (released April 2024):
- **Compatibility**: Supports Kubernetes v1.29, v1.30, v1.31 (n±1 version skew)
- **Stability**: 1.30 is a stable release (not beta)
- **Feature coverage**: Includes all common troubleshooting commands
- **Size**: ~50MB per platform (acceptable for installer)

---

## Consequences

### Positive

- **Zero-configuration**: kubectl works immediately after install
- **Consistent behavior**: Same kubectl version on all platforms
- **Offline capable**: No internet required for kubectl execution
- **Kubeconfig flexibility**: Users can upload multiple kubeconfig files
- **Security**: Binary checksum verified during CI build
- **Reliability**: No version skew errors with Kubernetes 1.29-1.31 clusters

### Negative

- **Installer size**: Increases by ~50MB per platform (150MB total for all platforms)
- **Update lag**: kubectl version frozen until TRCAA release
- **Disk usage**: Each install includes kubectl binary (no sharing across users)
- **Maintenance**: Need to periodically update kubectl version

### Trade-offs

We chose **reliability and UX over installer size**. The 50MB increase is acceptable for a desktop application targeting IT engineers who likely have kubectl needs.

---

## Mitigation Strategies

**Installer size**: 
- Compress binaries in bundle (reduces to ~15MB per platform)
- Document minimum disk space requirement in README

**kubectl version updates**:
- Add `scripts/update-kubectl.sh` to automate version bumps
- Schedule quarterly kubectl version reviews
- Document current version in CLAUDE.md and wiki

**Platform-specific issues**:
- Windows: Sign kubectl binary to avoid SmartScreen warnings
- macOS: Sign and notarize to pass Gatekeeper
- Linux: Verify `chmod +x` works across all distros

---

## Future Enhancements

1. **Optional system kubectl**: Add "Use system kubectl" toggle in Settings (falls back to bundled if not found)
2. **Version display**: Show kubectl version in Settings UI
3. **Auto-update**: Download newer kubectl if available (requires secure checksum verification)
4. **Plugin support**: Bundle common kubectl plugins (kubectx, kubens, stern)

---

## Related Decisions

- **ADR-007**: Three-Tier Shell Safety (kubectl commands classified as Tier 1/Tier 2)
- **ADR-008**: MCP Protocol Integration (alternative to bundling binaries — use MCP kubectl server)

---

## References

- **kubectl Releases**: https://kubernetes.io/releases/
- **Download Script**: `scripts/download-kubectl.sh`
- **Binary Management**: `src-tauri/src/shell/kubectl.rs`
- **Implementation PR**: #30 (Hackathon v1.0.0)
- **CI/CD**: `.github/workflows/release.yml` (kubectl download step)
- **Wiki**: `docs/wiki/Shell-Execution.md` (kubectl section)
