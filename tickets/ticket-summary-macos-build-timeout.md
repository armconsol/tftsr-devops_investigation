# Description

The macOS release build was hitting the runner limit after a long cold compile. The workflow now has the same cache treatment as the Linux release jobs plus target caching and a longer timeout.

# Acceptance Criteria

- `release-beta.yml` gives `build-macos-arm64` more time and caches Cargo/npm/target artifacts.
- `auto-tag.yml` applies the same macOS build hardening.
- The CI wiki reflects the workflow change.

# Work Implemented

- Added `timeout-minutes: 120` to the macOS release jobs.
- Added Cargo, npm, and target cache steps to both macOS release workflows.
- Updated `docs/wiki/CICD-Pipeline.md` to document the cache version and macOS build key.

# Testing Needed

- Validate the workflow YAML syntax.
- Confirm the macOS release job still publishes the DMG.
