# fix(ci): add libsodium to all build environments

## Description

All CI builds started failing with:

```
libsodium not found via pkg-config or vcpkg
```

`tauri-plugin-stronghold` depends on `libsodium-sys-stable` v1.24.0, which does **not** compile libsodium from source — it requires a pre-installed system library. None of the builder Docker images or the inline test job apt installs included `libsodium-dev`, so every build involving Rust compilation has been broken since `tauri-plugin-stronghold` was added.

The Windows cross-compile Dockerfile already pre-built libsodium from source (into `/usr/x86_64-w64-mingw32`), but the workflow never set `SODIUM_LIB_DIR` to tell the crate where to look, so it also failed via the same code path.

There is a secondary timing constraint: `build-images.yml` and `auto-tag.yml` both trigger on push to `master`. Even after Dockerfiles are fixed, the rebuilt images won't be ready in time for the concurrent release builds. Inline `apt-get install` is added to the workflow build steps to bridge that window; once images are rebuilt, the inline install becomes a harmless no-op.

## Acceptance Criteria

- [ ] `rust-fmt-check`, `rust-clippy`, and `rust-tests` CI jobs pass
- [ ] `build-linux-amd64` produces `.deb`/`.rpm` artifacts
- [ ] `build-linux-arm64` produces `.deb`/`.rpm` artifacts
- [ ] `build-windows-amd64` produces installer artifacts
- [ ] `build-macos-arm64` produces `.dmg` artifact (macOS runner assumed to have `libsodium` via Homebrew; if not, add `brew install libsodium || true` to the Build step)

## Work Implemented

| File | Change |
|---|---|
| `.docker/Dockerfile.linux-amd64` | Added `libsodium-dev` to apt packages baked into the image |
| `.docker/Dockerfile.linux-arm64` | Added `libsodium-dev` (amd64 host) in Step 1 and `libsodium-dev:arm64` (cross target) in Step 2 |
| `.gitea/workflows/test.yml` | Added `libsodium-dev` to the system deps apt install in `rust-fmt-check`, `rust-clippy`, and `rust-tests` |
| `.gitea/workflows/auto-tag.yml` | Inline `apt-get install libsodium-dev` before build (linux-amd64 and linux-arm64 jobs); `SODIUM_LIB_DIR`/`SODIUM_STATIC` env vars for Windows job |
| `.gitea/workflows/release-beta.yml` | Same three changes as `auto-tag.yml` |

## Testing Needed

1. Merge this PR to `master` — verify `Auto Tag` workflow succeeds across all four platform jobs
2. Push to `beta` — verify `Release Beta` workflow succeeds
3. After `Build CI Docker Images` workflow finishes rebuilding images, trigger a manual release run to confirm inline apt installs are redundant (both paths should work)
4. **macOS**: if `build-macos-arm64` still fails with a libsodium error, add `brew install libsodium || true` to the Build step in both `auto-tag.yml` and `release-beta.yml`
