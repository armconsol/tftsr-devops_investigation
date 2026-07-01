# RDP6 Bitmap Stride Mismatch Causes Horizontal Striping Artifacts

## Description

When connecting to servers that send RDP6 compressed bitmaps with tiled updates (e.g., NVIDIA DGX), the rendered display shows horizontal striping and blur artifacts.

## Root Cause

RDP6 `BitmapUpdate` PDUs can have different dimensions than their placement rectangles:
- `update.width` × `update.height` = actual decoded bitmap size
- `update.rectangle` = where to place the bitmap on screen

The current implementation of `apply_rgb24()` chunks the decoded RGB24 data using `rectangle.width()`, but the decoder produces data with `update.width` pixels per row. When these values differ, it causes stride mismatches and pixel data misalignment.

## Example from Real Traffic

Connecting to NVIDIA DGX server at 1920x1080:

```
32 bpp compressed RDP6_BITMAP_STREAM: source=348x11, rectangle=345x11
  update.width = 348
  Rectangle (left:788, right:1132) width = 1132 - 788 + 1 = 345
```

The decoder produces 348 pixels/row, but `apply_rgb24()` chunks at 345 pixels/row → pixel misalignment → horizontal stripes.

## Reproduction

1. Connect to any RDP server that sends RDP6 compressed bitmap tiles with `update.width != rectangle.width()`
2. Observe horizontal striping and blur in the rendered display
3. Common with: NVIDIA DGX, Windows Server with RemoteFX disabled

## Fix

See attached patch file: `0001-fix-rdp6-use-actual-bitmap-dimensions-instead-of-rec.patch`

**Changes:**
1. Add `apply_rgb24_with_dimensions()` that accepts explicit `source_width` parameter
2. Update RDP6 handler in `fast_path.rs` to use `update.width` (not `rectangle.width()`) for chunking
3. Keep `apply_rgb24()` for backward compatibility (delegates to new function)

**Testing:**
- Verified clean rendering at 1920x1080 against NVIDIA DGX server
- No regressions on other RDP servers

## Patch Application

```bash
git apply 0001-fix-rdp6-use-actual-bitmap-dimensions-instead-of-rec.patch
cargo test
```

## Files Modified

- `crates/ironrdp-session/src/fast_path.rs`: Use `update.width` for RDP6 bitmaps
- `crates/ironrdp-session/src/image.rs`: Add `apply_rgb24_with_dimensions()`
