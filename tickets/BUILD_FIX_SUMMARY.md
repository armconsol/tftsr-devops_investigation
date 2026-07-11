# Windows Build Fix Summary

## Issue
Windows build was failing with linker error:
```
undefined reference to `memset_explicit'
```

This was caused by `libsodium-sys-stable` (used by `tauri-plugin-stronghold`) requiring `memset_explicit`, which is not available in older MinGW toolchains.

## Root Cause
- `tauri-plugin-stronghold` → `stronghold_engine` → `libsodium-sys-stable v1.24.0`
- libsodium uses `memset_explicit` for secure memory clearing
- MinGW doesn't provide `memset_explicit` in its standard library
- The function is only available in Windows 8+ SDK with specific headers

## Solution
Created a C shim (`memset_s_shim.c`) that provides `memset_explicit` implementation:
- Uses volatile pointers to prevent compiler optimization of memory clearing
- Falls back to `memset_s` if Windows 8+ headers are available
- Compiled only for Windows GNU targets via `build.rs`

## Changes Made

### Files Added
- **`src-tauri/memset_s_shim.c`** - C implementation of memset_explicit fallback

### Files Modified
- **`src-tauri/build.rs`**
  - Added conditional compilation of shim for Windows GNU targets
  - Uses `cc` crate to compile C code

- **`src-tauri/Cargo.toml`**
  - Added `cc = "1.0"` to `[build-dependencies]`

- **`.gitea/workflows/release-beta.yml`**
  - Set `CFLAGS_x86_64_pc_windows_gnu: "-D_WIN32_WINNT=0x0602"` (Windows 8)
  - Set `SODIUM_STATIC: "yes"` to force static linking
  - Set `SODIUM_LIB_DIR: ""` to use vendored build

## Technical Details

### The C Shim
```c
void *memset_explicit(void *s, int c, size_t n) {
    volatile unsigned char *p = (volatile unsigned char *)s;
    while (n--) {
        *p++ = (unsigned char)c;
    }
    return s;
}
```

The `volatile` keyword prevents the compiler from optimizing away the memory write operations, which is crucial for security-sensitive memory clearing (like clearing crypto keys).

### Build Process
1. `build.rs` detects Windows GNU target
2. Compiles `memset_s_shim.c` using `cc::Build`
3. Links the shim object into the final binary
4. libsodium finds the symbol at link time

## Commit
**`9e3e3766`** - `fix(build): resolve Windows MinGW memset_explicit linking error`

## Testing
- ✅ macOS build: Compiles successfully (shim not compiled)
- ⏳ Windows build: Will be tested in CI
- ⏳ Linux builds: Should not be affected (shim not compiled)

## References
- Issue: Windows cross-compilation failing with `memset_explicit` undefined
- libsodium uses `memset_explicit` for secure memory operations
- MinGW compatibility issue with Windows 8+ APIs
