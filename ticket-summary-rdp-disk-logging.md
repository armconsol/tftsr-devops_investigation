# Ticket Summary: Backend Disk Logging + Debug Toggle

## Description
Add backend local-disk logging to improve visibility when diagnosing RDP black screen issues, and provide a Settings toggle to switch backend logging level from normal to debug.

## Acceptance Criteria
- Backend writes logs to local disk so runtime behavior can be inspected post-failure.
- Logging defaults to normal verbosity.
- A Settings toggle exists to enable/disable backend debug logging.
- RDP troubleshooting information remains available in backend logs.
- Changes are covered with TDD-oriented tests and project checks.

## Work Implemented
- Added backend log file sink (daily rolling) at `<app_data_dir>/logs/backend.log` while retaining console logs.
- Added runtime log-level reload support and exposed `set_debug_logging_enabled`.
- Extended backend settings model with `debug_logging_enabled` (default `false`).
- Updated `update_settings` handling to apply debug-level changes live.
- Extended frontend settings contracts and Zustand store with `debug_logging_enabled`.
- Added Security settings toggle for backend debug logging.
- Synced persisted frontend toggle to backend on app startup/state changes.
- Added/updated tests for settings defaults and partial-settings debug toggle behavior.
- Updated wiki docs:
  - `docs/wiki/IPC-Commands.md`
  - `docs/wiki/Architecture.md`
  - `docs/wiki/Troubleshooting.md`

## Testing Needed
- Verify log file creation and writes on each platform target.
- Toggle debug logging in Settings and confirm backend log level changes live.
- Reproduce RDP black screen path and confirm useful telemetry appears in `backend.log`.
- Full Rust checks once IronRDP patch path is available in the environment.
