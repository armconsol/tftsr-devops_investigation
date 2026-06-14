# libsodium pkg-config Detection Fix

> **Note:** This document describes **only the changes in this PR (PR #102)**. For the complete fix history including PR #101 (Docker packages, smoke test), see `LIBSODIUM_BUILD_HISTORY.md`.

## Description

This PR fixes libsodium build failures that persisted after adding `libsodium-dev` packages to Docker images (PR #101). The issue was that `libsodium-sys-stable`'s build script wasn't being explicitly told **how** to find libsodium.

**Remaining build failures after PR #101:**

1. **Linux amd64/arm64**: `libsodium not found via pkg-config or vcpkg` (despite `libsodium-dev` + `pkg-config` being installed)
2. **Windows cross-build**: `SODIUM_LIB_DIR is incompatible with SODIUM_USE_PKG_CONFIG` (conflicting detection methods)

## Root Cause

The `libsodium-sys-stable` crate's `build.rs` checks environment variables in this precedence:

1. If `SODIUM_LIB_DIR` is set → use explicit path (incompatible with `SODIUM_USE_PKG_CONFIG` mode)
2. If `SODIUM_USE_PKG_CONFIG` ≠ `"no"` (string equality) → try pkg-config detection
3. Fall back to vcpkg or fail with error

**Note on string values:** The build script performs string comparison, so `"no"` disables pkg-config while any other value (including `"1"`, `"yes"`, or empty) enables it. YAML quotes preserve these as strings.

**What went wrong:**

- **Linux**: Had the packages installed but wasn't explicitly told to use pkg-config → fell through to vcpkg → failed
- **Windows**: Set `SODIUM_LIB_DIR` (from previous PR) but also had pkg-config available → conflicting modes → build script error

## Changes in This PR

### `.gitea/workflows/auto-tag.yml`

#### Linux amd64 build (line ~347)
```yaml
env:
  SODIUM_USE_PKG_CONFIG: "1"  # NEW: Force pkg-config detection
```

**Why:** Ensures `libsodium-sys-stable` uses the installed `libsodium-dev` package via pkg-config.

#### Linux arm64 build (line ~633)
```yaml
env:
  SODIUM_USE_PKG_CONFIG: "1"  # NEW: Force pkg-config for cross-compile
```

**Why:** Same as amd64 - force pkg-config to find the arm64 libsodium package.

#### Windows cross-compile build (line ~448)
```yaml
env:
  SODIUM_LIB_DIR: /usr/x86_64-w64-mingw32/lib      # Already present from PR #101
  SODIUM_STATIC: "1"                               # Already present from PR #101
  SODIUM_USE_PKG_CONFIG: "no"                      # NEW: Disable pkg-config
```

**Why:** Prevents conflict between explicit path mode (`SODIUM_LIB_DIR`) and pkg-config detection. Windows uses pre-built libsodium from Dockerfile, not system packages.

### Documentation

**Files changed in this PR:**
- `LIBSODIUM_BUILD_FIX.md` (this file) - Documents env var strategy for pkg-config detection
- `LIBSODIUM_PKG_CONFIG_FIX.md` - Alternative/detailed version of this doc
- `LIBSODIUM_BUILD_HISTORY.md` - Complete fix history across PR #101 and PR #102

Explains:
- Platform-specific environment variable strategy
- Build script precedence order
- Rationale for each approach

## Strategy Summary

| Platform | Method | Env Vars | Reason |
|----------|--------|----------|--------|
| Linux amd64 | pkg-config | `SODIUM_USE_PKG_CONFIG=1` | Has `libsodium-dev` + `pkg-config` installed |
| Linux arm64 | pkg-config | `SODIUM_USE_PKG_CONFIG=1` | Has `libsodium-dev:arm64` + `pkg-config` |
| Windows | explicit path | `SODIUM_LIB_DIR=...` + `SODIUM_USE_PKG_CONFIG=no` | Pre-built lib in known location, disable pkg-config |

## Testing

This PR only modifies CI workflow environment variables. Testing occurs via CI pipeline:

- [ ] Linux amd64 build succeeds with pkg-config detection
- [ ] Linux arm64 build succeeds with cross-compile pkg-config
- [ ] Windows build succeeds with explicit lib path (no pkg-config conflict)
- [ ] All platforms produce valid `.deb`, `.rpm`, `.exe`, `.msi` artifacts

## Acceptance Criteria (This PR Only)

- [x] Added `SODIUM_USE_PKG_CONFIG` env vars to all three CI build targets
- [x] Documentation accurately reflects only changes in this PR
- [ ] Linux amd64 CI build succeeds
- [ ] Linux arm64 CI build succeeds  
- [ ] Windows CI build succeeds
- [ ] All platforms produce valid artifacts

## Relationship to PR #101

**PR #101** (already merged to beta):
- Added `libsodium-dev` to Linux Docker images (`.docker/Dockerfile.*`)
- Added `SODIUM_LIB_DIR` + `SODIUM_STATIC` to Windows workflow
- Added smoke test in `src-tauri/src/state.rs`

**This PR (PR #102)**:
- Adds `SODIUM_USE_PKG_CONFIG` env vars to tell build script **how** to find libsodium
- Fixes detection failures that persisted after package installation
- **No Dockerfile changes** (those were in PR #101)
- **No test changes** (those were in PR #101)

Both PRs together form the complete fix. See `LIBSODIUM_BUILD_HISTORY.md` for the full story.
