# Pull Request Summary

## PR #100: Fix Proxmox Remote Add Error

**URL**: https://gogs.tftsr.com/sarman/tftsr-devops_investigation/pulls/100

**Branch**: `fix/proxmox-remote-add-error` → `beta`

**Version**: `1.2.3` → `1.2.4`

---

## Problem

Users could not add Proxmox remotes when providing URLs with port numbers (e.g., `https://172.0.0.18:8006`). The error displayed was: **"Failed to add remote"**

### Root Cause
The `RemotesPage.tsx` component incorrectly parsed URLs containing ports:
1. User enters: `https://172.0.0.18:8006`
2. Code strips protocol → `172.0.0.18:8006`
3. Code uses this **with port still attached** as hostname
4. Code **also** sends separate port parameter: `8006`
5. Backend receives malformed: `url: "172.0.0.18:8006"` + `port: 8006`
6. Connection fails

---

## Solution

Added URL parsing logic to properly handle ports in both add and edit operations:

```typescript
// Parse URL to extract hostname and port
let hostname = config.url.replace(/^https?:\/\//, '');
let port = config.type === 'pve' ? 8006 : 8007;

// If URL contains port, extract it
const portMatch = hostname.match(/:(\d+)$/);
if (portMatch) {
  port = parseInt(portMatch[1], 10);
  hostname = hostname.replace(/:\d+$/, '');
}
```

Now correctly handles:
- ✅ Full URLs with ports: `https://172.0.0.18:8006` → hostname: `172.0.0.18`, port: `8006`
- ✅ Hostnames only: `172.0.0.18` → hostname: `172.0.0.18`, port: `8006` (default)
- ✅ Custom ports: `https://192.168.1.100:8443` → hostname: `192.168.1.100`, port: `8443`

---

## Changes

### Modified Files
- **`src/pages/Proxmox/RemotesPage.tsx`**
  - Fixed `handleAddRemote()` function
  - Fixed `handleEditRemote()` function
  - Added port extraction logic
  - Properly separates hostname from port

### Version Bump
- `package.json`: `1.2.3` → `1.2.4`
- `src-tauri/Cargo.toml`: `1.2.3` → `1.2.4`
- `src-tauri/tauri.conf.json`: `1.2.3` → `1.2.4`
- `src-tauri/Cargo.lock`: Updated
- `src-tauri/gen/schemas/macOS-schema.json`: Regenerated

---

## Commits

1. **`666de6dd`** - `fix(proxmox): parse port from URL when adding remote`
2. **`58cbe525`** - `chore: bump version to 1.2.4`
3. **`0b409c32`** - `chore: update Cargo.lock and schema for v1.2.4`

---

## Testing

### Completed
- [x] ESLint checks passed
- [x] Rust compilation successful
- [x] Database corruption fixed (removed 0-byte DB)

### Required Before Merge
- [ ] Manual test: Add remote with `https://172.0.0.18:8006`
- [ ] Manual test: Add remote with `172.0.0.18` (should use port 8006)
- [ ] Manual test: Add PBS remote with custom port
- [ ] Manual test: Edit existing remote and verify port changes
- [ ] Verify remote connection succeeds
- [ ] Verify VMs/containers load after adding remote
- [ ] Test with self-signed certificates
- [ ] Test with API token authentication

---

## Stats

- **Files changed**: 6
- **Additions**: +263 lines
- **Deletions**: -10 lines
- **State**: Open, mergeable
- **CI Status**: Pending

---

## Next Steps

1. ✅ Branch pushed to origin
2. ✅ PR created (#100)
3. ⏳ Awaiting review
4. ⏳ Manual testing
5. ⏳ Merge to beta
6. ⏳ Test on beta branch
7. ⏳ Merge to master (if applicable)
