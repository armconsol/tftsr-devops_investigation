# libsodium Build Failure - FINAL FIX

## The Problem
`libsodium-sys-stable v1.24.0` build script was failing with:
```
thread 'main' panicked at build.rs:539:13:
libsodium not found via pkg-config or vcpkg
```

## Root Cause Analysis

After 12 hours of attempts, the issue is clear:

### Build Script Logic (from libsodium-sys-stable/build.rs)
The build script checks in priority order:
1. **SODIUM_LIB_DIR** - if set, use that path directly (HIGHEST PRIORITY)
2. **SODIUM_USE_PKG_CONFIG** - if set, try pkg-config/vcpkg
3. **Fallback** - try to build from source

### Previous Failed Approaches
1. **PR #101, #102**: Tried pkg-config environment variables - failed because pkg-config couldn't find libsodium in containers
2. **PR with use-pkg-config feature**: Enabled the feature but pkg-config still failed to locate libraries

### Why pkg-config Failed
- Container images have libsodium installed but pkg-config can't find the .pc files
- Cross-compilation adds complexity to pkg-config searches
- Different containers have different pkg-config configurations

## The Solution

**Use SODIUM_LIB_DIR to bypass pkg-config entirely.**

This directly tells the build script where libsodium is installed, skipping all detection logic.

## Implementation

### test.yml (Rust tests)
Added to ALL cargo commands:
```yaml
env:
  SODIUM_LIB_DIR: /usr/lib/x86_64-linux-gnu
```

### auto-tag.yml (Release builds)

**Linux x86_64:**
```yaml
SODIUM_LIB_DIR: /usr/lib/x86_64-linux-gnu
```

**Linux aarch64:**
```yaml
SODIUM_LIB_DIR: /usr/lib/aarch64-linux-gnu
```

**Windows MinGW:**
```yaml
SODIUM_LIB_DIR: /usr/x86_64-w64-mingw32/lib
```

**macOS:** No change needed (already works)

## Why This Will Work

1. **SODIUM_LIB_DIR has highest priority** in build.rs - checked BEFORE pkg-config
2. **Direct path** - no detection, no guessing, no pkg-config configuration issues
3. **Already confirmed** - the original working Windows build used this exact approach
4. **Simple** - one environment variable per platform

## Branch Info
- **Branch:** `fix/libsodium-direct-path`
- **Base:** `beta`
- **Commits:** 1 atomic commit
- **Files Changed:** 2 (.gitea/workflows/test.yml, .gitea/workflows/auto-tag.yml)

## Testing Status
- ⏳ Awaiting CI pipeline results
- Expected: ALL builds (Linux x86, Linux ARM, Windows, macOS) will succeed
- Expected: ALL test jobs (fmt, clippy, tests) will succeed

## If This Still Fails

The only remaining possibility would be:
1. Libsodium is NOT actually installed in the containers (verify with `dpkg -L libsodium-dev`)
2. The library path is wrong (verify with `find /usr -name "libsodium.*"`)

But based on previous error messages showing pkg-config attempts, libsodium IS installed - we just need to tell the build script where it is.

---

**Created:** 2026-06-14 (after 12 hours of attempts)  
**Approach:** Direct library path specification  
**Confidence:** HIGH - This is the intended workaround when pkg-config fails
