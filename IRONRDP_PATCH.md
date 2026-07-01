# IronRDP Patch for RDP6 Bitmap Stride Fix

## Issue

When connecting to servers that send RDP6 compressed bitmaps (like NVIDIA DGX), the rendered display showed horizontal striping and blur artifacts.

## Root Cause

RDP6 bitmap updates can have different dimensions than their placement rectangles:
- `update.width` and `update.height` specify the **decoded bitmap size**
- `update.rectangle` specifies **where to place** the bitmap on screen

IronRDP's `apply_rgb24()` was chunking the decoded RGB24 data using `rectangle.width()` instead of the actual `update.width`, causing stride mismatches when these values differed.

Example from DGX server logs:
```
32 bpp compressed RDP6_BITMAP_STREAM: source=348x11, rectangle=345x11
```

The decoder produced 348 pixels per row, but `apply_rgb24()` was chunking at 345 pixels per row, causing pixel data misalignment → horizontal stripes.

## Fix

**Location:** `/tmp/ironrdp-patch/`

**Modified Files:**
1. `crates/ironrdp-session/src/image.rs`
   - Added `apply_rgb24_with_dimensions()` that accepts explicit `source_width` parameter
   - Modified `apply_rgb24()` to delegate to the new function for backward compatibility

2. `crates/ironrdp-session/src/fast_path.rs`
   - Updated RDP6 bitmap handler to use `apply_rgb24_with_dimensions()`
   - Pass `update.width` (not `rectangle.width()`) for chunking

**Commits in patch repo:**
```bash
cd /tmp/ironrdp-patch
git log --oneline
```

## Upstream Status

**Pull Request:** https://github.com/Devolutions/IronRDP/pull/1398

Submitted to IronRDP upstream on 2026-07-01 by @armconsol.

Includes:
- Complete fix with detailed explanation
- Real traffic examples from DGX server showing dimension mismatch
- Debug logging to expose the issue
- Backward-compatible API changes

## Building with Patch

The patch is applied via path dependencies in `src-tauri/Cargo.toml`:

```toml
ironrdp = { path = "/tmp/ironrdp-patch/crates/ironrdp", features = [...] }
ironrdp-graphics = { path = "/tmp/ironrdp-patch/crates/ironrdp-graphics" }
# ... (all ironrdp crates)
```

**For production builds**, either:
1. Wait for upstream fix and use released version
2. Use git dependency with our patched fork
3. Vendor the patched source into this repo

## Testing

Verified against NVIDIA DGX server:
- Before: Horizontal striping and blur at 1920x1080
- After: Clean rendering with no artifacts

Debug output confirms dimension mismatches are now handled correctly:
```
DEBUG ironrdp_session::fast_path: source=348x11, rectangle=345x11
```

## Patch Maintenance

If updating IronRDP version:
1. Clone new version to `/tmp/ironrdp-patch`
2. Re-apply the fix to `image.rs` and `fast_path.rs`
3. Test against DGX server
4. Update `Cargo.toml` dependencies
