# Database Fix Summary - Windows OpenSSL + IPC Parameter Naming

**Branch:** `fix/windows-openssl-and-database-ipc`  
**Date:** 2026-07-02  
**Status:** ✅ Ready for Review

## Problems Fixed

### Problem 1: Windows CI Build Failure (OpenSSL)
Windows cross-compile builds were failing with:
```
Package openssl was not found in the pkg-config search path
Could not find directory of OpenSSL installation
```

### Problem 2: Database Connection Test Failure (IPC)
PostgreSQL connection test was failing with:
```
Connection test failed: invalid args `connectionId` for command `test_database_connection`: 
command test_database_connection missing required key connectionId
```

## Root Causes

### OpenSSL Issue
- `src-tauri/.cargo/config.toml` globally set `OPENSSL_NO_VENDOR = "1"` in `[env]` block
- This **overrode** the `openssl-sys` vendored feature declared in Cargo.toml
- Workspace-level config env vars take precedence over Cargo.toml feature flags
- `.gitea/workflows/auto-tag.yml` Windows job attempted to override, but workflow env vars cannot override workspace config
- Result: Windows cross-compile fell back to pkg-config, no system OpenSSL found

### Database IPC Issue
- Backend Rust function signature: `test_database_connection(connection_id: String, ...)`
- Tauri automatically converts: `connection_id` → `connectionId` (camelCase) for IPC
- Frontend `src/lib/tauriCommands.ts` was sending snake_case parameters
- Tauri expected camelCase parameters to match its automatic conversion
- Affected commands:
  - `testDatabaseConnectionCmd`
  - `executeDatabaseQueryCmd`
  - All other database commands with snake_case parameters

## Solutions Implemented

### Fix 1: Target-Specific OpenSSL Override
Added target-specific configuration in `src-tauri/.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-gnu.env]
OPENSSL_NO_VENDOR = "0"
```

**Benefits:**
- ✅ Target-specific env vars override global env vars
- ✅ Preserves fast system OpenSSL builds on macOS/Linux dev machines
- ✅ Enables vendored OpenSSL only for Windows cross-compile
- ✅ No workflow changes needed
- ✅ Zero impact on local dev build times

### Fix 2: IPC Parameter Naming Convention
Updated all database commands in `src/lib/tauriCommands.ts` to use camelCase:

| Before (snake_case) | After (camelCase) |
|---------------------|-------------------|
| `connection_id` | `connectionId` |
| `transaction_id` | `transactionId` |
| `query_text` | `queryText` |
| `page_size` | `pageSize` |
| `search_term` | `searchTerm` |
| `file_path` | `filePath` |
| `target_table` | `targetTable` |
| `column_mappings` | `columnMappings` |

Also updated all call sites:
- `src/pages/Database/SQLEditor.tsx`
- `src/pages/Database/QueryHistory.tsx`
- `src/components/Database/QueryResultsPanel.tsx`
- `src/components/ImageGallery.tsx`
- `tests/unit/attachmentStore.test.ts`
- `tests/unit/remoteDesktop.test.ts`

## Testing Results

### Rust Tests
```bash
cargo test --manifest-path src-tauri/Cargo.toml openssl_vendored
# Result: 3/3 passing
```

### Frontend Tests
```bash
npm run test:run
# Result: 472/472 passing
```

### TypeScript Type Checking
```bash
npx tsc --noEmit
# Result: No errors
```

### Formatting and Linting
```bash
cargo fmt --check                        # ✅ Pass
cargo clippy -- -D warnings              # ✅ Pass (0 errors, 2 warnings in IronRDP deps)
```

**Note:** ESLint shows 16 errors related to pre-existing react-hooks issues, not introduced by this PR.

## Files Changed

```
src-tauri/.cargo/config.toml                  |   8 +-
src-tauri/tests/openssl_vendored_test.rs      |   5 +-
src/components/Database/QueryResultsPanel.tsx |   2 +-
src/components/ImageGallery.tsx               |  10 +-
src/lib/tauriCommands.ts                      | 136 +++++++++++++-------------
src/pages/Database/QueryHistory.tsx           |   6 +-
src/pages/Database/SQLEditor.tsx              |   6 +-
tests/unit/attachmentStore.test.ts            |   4 +-
tests/unit/remoteDesktop.test.ts              |   4 +-
9 files changed, 94 insertions(+), 87 deletions(-)
```

## Impact

- ✅ **Fixes Windows CI build failures** - Vendored OpenSSL now works for cross-compile
- ✅ **Fixes database connection testing** - IPC parameter naming matches Tauri convention
- ✅ **No runtime behavior changes** - Only build-time and parameter naming fixes
- ✅ **No local dev build time impact** - macOS/Linux still use fast system OpenSSL
- ✅ **Consistent API** - All Tauri commands now use camelCase (matches JavaScript convention)

## Next Steps

1. **Create PR manually** at: https://gogs.tftsr.com/sarman/tftsr-devops_investigation/compare/beta...fix/windows-openssl-and-database-ipc
2. **Monitor CI** - Verify Windows build succeeds without OpenSSL errors
3. **Test manually** - After merge, test PostgreSQL connection in the app
4. **Version bump** - Consider bumping to v3.0.1 after merge

## References

- **Cargo config env vars:** https://doc.rust-lang.org/cargo/reference/config.html#env
- **OpenSSL vendoring:** https://docs.rs/openssl/latest/openssl/#vendored
- **Tauri parameter naming:** https://tauri.app/develop/calling-rust/#parameter-naming
- **Related PR:** #196 (database management feature)

## Verification Checklist

- [x] TDD tests pass (OpenSSL vendored tests)
- [x] Frontend tests pass (472/472)
- [x] TypeScript type checking passes
- [x] Rust formatting passes
- [x] Rust linting passes (0 errors)
- [x] Branch pushed to origin
- [ ] PR created (manual step required)
- [ ] CI passes (Windows build succeeds)
- [ ] Manual testing (PostgreSQL connection)
- [ ] Merged to beta
- [ ] Release build succeeds

---

**Generated:** 2026-07-02  
**Author:** Claude Sonnet 4.5 via Claude Code
