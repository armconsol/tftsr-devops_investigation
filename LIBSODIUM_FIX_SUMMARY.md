# libsodium Build Failures - Root Cause Analysis & Fix

## Issue Summary

All three CI build platforms (linux-amd64, windows-amd64, linux-arm64) were failing with libsodium detection errors in `libsodium-sys-stable v1.24.0`.

### Error Details

**linux-amd64 & linux-arm64:**
```
libsodium not found via pkg-config or vcpkg
```

**windows-amd64:**
```
SODIUM_LIB_DIR is incompatible with SODIUM_USE_PKG_CONFIG. 
Set the only one env variable
```

## Root Cause

The `libsodium-sys-stable` crate (dependency chain: `tauri-plugin-stronghold` → `stronghold_engine` → `libsodium-sys-stable`) has strict requirements for environment variable configuration:

1. **Linux builds** require `SODIUM_USE_PKG_CONFIG=1` to use pkg-config detection
2. **Windows builds** require either:
   - `SODIUM_LIB_DIR` pointing to the pre-built library directory, OR
   - `SODIUM_USE_PKG_CONFIG` for pkg-config detection
   - **BUT NOT BOTH** (mutually exclusive)
3. **Cross-compilation** requires proper PKG_CONFIG_PATH setup to find architecture-specific .pc files

### Original Configuration Issues

**release-beta.yml (beta branch releases):**
- **linux-amd64**: Missing `SODIUM_USE_PKG_CONFIG=1`
- **windows-amd64**: Set `SODIUM_LIB_DIR: ""` (empty string) which conflicts with implicit pkg-config attempt
- **linux-arm64**: Missing `SODIUM_USE_PKG_CONFIG=1`, incomplete PKG_CONFIG_PATH

**auto-tag.yml (master branch releases):**
- **linux-amd64**: ✅ Already had `SODIUM_USE_PKG_CONFIG=1`
- **windows-amd64**: ✅ Already had correct configuration
- **linux-arm64**: Had `SODIUM_USE_PKG_CONFIG=1` but incomplete PKG_CONFIG_PATH

## Solution

### Changes to `.gitea/workflows/release-beta.yml`

#### 1. Linux amd64 Build
```yaml
env:
  APPIMAGE_EXTRACT_AND_RUN: "1"
  SODIUM_USE_PKG_CONFIG: "1"  # Added
```

**Why:** Forces libsodium-sys to use pkg-config, which finds `libsodium-dev` package installed in the Docker image.

#### 2. Windows amd64 Build
```yaml
env:
  CC_x86_64_pc_windows_gnu: x86_64-w64-mingw32-gcc
  CXX_x86_64_pc_windows_gnu: x86_64-w64-mingw32-g++
  AR_x86_64_pc_windows_gnu: x86_64-w64-mingw32-ar
  CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER: x86_64-w64-mingw32-gcc
  OPENSSL_NO_VENDOR: "0"
  OPENSSL_STATIC: "1"
  SODIUM_LIB_DIR: /usr/x86_64-w64-mingw32/lib  # Changed from ""
  SODIUM_STATIC: "1"                           # Changed from "yes"
  SODIUM_USE_PKG_CONFIG: "no"                  # Added (explicit disable)
```

**Why:** 
- Points `SODIUM_LIB_DIR` to the actual pre-built libsodium location (installed by Dockerfile.windows-cross)
- Explicitly disables pkg-config to prevent conflict
- Standardizes `SODIUM_STATIC` to "1" (matches auto-tag.yml)

#### 3. Linux arm64 Build
```yaml
env:
  CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc
  CXX_aarch64_unknown_linux_gnu: aarch64-linux-gnu-g++
  AR_aarch64_unknown_linux_gnu: aarch64-linux-gnu-ar
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: aarch64-linux-gnu-gcc
  PKG_CONFIG_SYSROOT_DIR: /usr/aarch64-linux-gnu
  PKG_CONFIG_PATH: /usr/lib/aarch64-linux-gnu/pkgconfig:/usr/aarch64-linux-gnu/lib/pkgconfig  # Extended
  PKG_CONFIG_ALLOW_CROSS: "1"
  SODIUM_USE_PKG_CONFIG: "1"  # Added
  OPENSSL_NO_VENDOR: "0"
  OPENSSL_STATIC: "1"
  APPIMAGE_EXTRACT_AND_RUN: "1"
```

**Why:**
- Added `SODIUM_USE_PKG_CONFIG=1` to force pkg-config detection
- Extended PKG_CONFIG_PATH to include `/usr/aarch64-linux-gnu/lib/pkgconfig` where arm64 libsodium.pc is located

### Changes to `.gitea/workflows/auto-tag.yml`

#### Linux arm64 Build Only
```yaml
PKG_CONFIG_PATH: /usr/lib/aarch64-linux-gnu/pkgconfig:/usr/aarch64-linux-gnu/lib/pkgconfig
```

**Why:** Same PKG_CONFIG_PATH extension as release-beta.yml for consistency.

## Technical Details

### Docker Image libsodium Installation

**Dockerfile.linux-amd64:**
```dockerfile
RUN apt-get install -y -qq --no-install-recommends \
    libsodium-dev \
    ...
```
Installs to: `/usr/lib/x86_64-linux-gnu/` with pkgconfig in `/usr/lib/x86_64-linux-gnu/pkgconfig/`

**Dockerfile.linux-arm64:**
```dockerfile
RUN apt-get install -y -qq --no-install-recommends \
    libsodium-dev:arm64 \
    ...
```
Installs to: `/usr/aarch64-linux-gnu/lib/` with pkgconfig in `/usr/aarch64-linux-gnu/lib/pkgconfig/`

**Dockerfile.windows-cross:**
```dockerfile
RUN set -eu \
    && SODIUM_VER="1.0.20" \
    && curl -fsSL "https://download.libsodium.org/libsodium/releases/libsodium-${SODIUM_VER}.tar.gz" \
       | tar -xz -C /tmp \
    && cd "/tmp/libsodium-${SODIUM_VER}" \
    && ./configure \
         --host=x86_64-w64-mingw32 \
         --prefix=/usr/x86_64-w64-mingw32 \
         --disable-shared \
         --enable-static \
    && make -j"$(nproc)" \
    && make install \
    && rm -rf "/tmp/libsodium-${SODIUM_VER}"
```
Installs to: `/usr/x86_64-w64-mingw32/lib/libsodium.a`

### libsodium-sys-stable Build Logic

From the error messages, the crate's build.rs checks in this order:
1. If `SODIUM_LIB_DIR` is set AND `SODIUM_USE_PKG_CONFIG` is set → **ERROR** (mutually exclusive)
2. If `SODIUM_LIB_DIR` is set → use direct library path
3. If `SODIUM_USE_PKG_CONFIG` is set → use pkg-config
4. Try pkg-config automatically
5. Try vcpkg
6. If all fail → panic with "libsodium not found via pkg-config or vcpkg"

## Testing Strategy

### Pre-merge Testing
1. ✅ Local syntax validation (yaml parsing)
2. ✅ Git diff review
3. ⏳ Push to beta branch and monitor CI runs

### Post-merge Validation
1. Verify all four platform builds succeed in release-beta.yml workflow
2. Check artifact uploads complete successfully
3. Download and smoke-test each platform binary

## Files Modified

- `.gitea/workflows/release-beta.yml` - 3 build job environment sections
- `.gitea/workflows/auto-tag.yml` - 1 build job environment section (linux-arm64)

## Related History

- PR #101: Initial Windows memset_explicit fix (addressed different issue)
- PR #102: This fix (libsodium detection across all platforms)

## Success Criteria

All platform builds in release-beta.yml workflow must:
- ✅ Complete `cargo build` without libsodium errors
- ✅ Generate platform-specific bundles (.deb, .rpm, .exe, .msi, .dmg)
- ✅ Successfully upload artifacts to Gitea releases
- ✅ Exit with code 0

## References

- libsodium-sys-stable crate: https://crates.io/crates/libsodium-sys-stable
- libsodium source: https://download.libsodium.org/libsodium/releases/
- pkg-config documentation: https://www.freedesktop.org/wiki/Software/pkg-config/
