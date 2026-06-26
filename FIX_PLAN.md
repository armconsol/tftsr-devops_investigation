# Kubectl Runtime Implementation Fix Plan

## Issues Identified

### CRITICAL BLOCKERS

1. **std::mem::drop(child.kill()) ignores async Kill future** (kube.rs:532-540)
   - `child.kill()` returns a `Future<Output = ()>` that must be awaited
   - Current code drops the future without awaiting, leaving process in undefined state

2. **Arc<Mutex<Child>> is not Send/Sync** (kube.rs:500, portforward.rs:14)
   - `tokio::process::Child` is NOT `Send` or `Sync`
   - `std::sync::Mutex` provides no `Send` guarantee for its contents
   - Cannot safely share `Child` across async boundaries

3. **No error propagation from kubectl subprocess** (kube.rs:530-531, 548)
   - stderr/stdout from kubectl subprocess are completely ignored
   - No way to detect kubectl errors or capture error messages
   - Session state never updated with error information

4. **std::sync::Mutex<Child> in PortForwardSession** (portforward.rs:23, 87, 103)
   - Same issues as #2, plus `Drop` implementation can't await

### WARNING ISSUES

5. **validate_resource_name regex not cached** (kube.rs:303-304)
   - `Regex::new()` called on every validation call
   - Should use `lazy_static!` or `once_cell::sync::Lazy<Regex>`

6. **Temp kubeconfig not cleaned on all paths** (kube.rs:524-534)
   - `TempFileCleanup` struct exists but only used in `discover_pods`
   - `start_port_forward` and `test_cluster_connection` don't clean up

7. **Tests don't verify subprocess exists** (cluster_management.rs:278-290)
   - No mock Command framework or subprocess verification

## Implementation Plan

### Phase 1: Core Architecture Fix

**Goal:** Replace unsafe `Arc<Mutex<Child>>` with proper async-safe storage

**Approach:**
1. Store `JoinHandle<()>` instead of `Child` directly
2. Spawn background task to wait on child and update session state
3. Use `tokio::sync::Mutex` for session state access
4. Implement proper async cleanup in `stop()` and `Drop`

### Phase 2: Error Handling

**Goal:** Capture and propagate kubectl subprocess errors

**Approach:**
1. Background task waits on child and captures exit status
2. Update session state with error messages on failure
3. Store stderr/stdout for debugging
4. Propagate errors to UI via session status

### Phase 3: Cleanup Improvements

**Goal:** Ensure temp files are always cleaned up

**Approach:**
1. Use RAII pattern consistently across all functions
2. Add cleanup hooks for panic/early-return paths
3. Store temp path in session struct for later cleanup

### Phase 4: Regex Caching

**Goal:** Cache compiled regex for performance

**Approach:**
1. Define `static ref NAME_PATTERN_REGEX: Lazy<Regex> = ...`
2. Replace `Regex::new()` call with static reference

## Files to Modify

1. `src-tauri/src/kube/portforward.rs` - Core architecture fix
2. `src-tauri/src/commands/kube.rs` - Integration and fixes
3. `src-tauri/tests/integration/kube/cluster_management.rs` - Add subprocess verification
4. `src-tauri/tests/integration/kube/port_forwarding.rs` - Add subprocess verification

## Test Strategy

After fixes:
1. Run `cargo test --lib` - expect 325 tests passing
2. Run `cargo clippy` - expect no warnings
3. Run type check: `npx tsc --noEmit` - expect no errors
4. Run frontend tests: `npm run test:run` - expect 98 tests passing
