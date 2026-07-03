# PR #196 Review Findings - Action Plan

## Summary of Legitimate Issues

After reviewing all 33 batches of automated review comments, here are the **legitimate issues** that require action:

---

## 1. SQL Injection Risk in build_update_statement
**Severity**: WARNING  
**File**: `src-tauri/src/commands/database.rs:1722`  
**Issue**: Table and column names directly interpolated without validation

**Fix**: Add validation regex before interpolation
```rust
fn validate_identifier(name: &str) -> Result<(), String> {
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(format!("Invalid identifier: {}", name));
    }
    Ok(())
}
```

**Status**: ✅ WILL FIX

---

## 2. Unsafe Array Access in Cassandra Driver
**Severity**: WARNING  
**Files**: 
- `src-tauri/src/db_drivers/cassandra/schema.rs:36`
- `src-tauri/src/db_drivers/cassandra/schema.rs:69`
- `src-tauri/src/db_drivers/cassandra/schema.rs:95`

**Issue**: Direct array access without bounds checking: `row.columns[0]`

**Fix**: Use safe pattern: `row.columns.get(0).ok_or("Missing column")?`

**Status**: ✅ WILL FIX

---

## 3. Ignored Query Parameters in Cassandra Driver
**Severity**: BLOCKER  
**File**: `src-tauri/src/db_drivers/cassandra/driver.rs:154`  
**Issue**: `session.query(query, &[])` ignores `_params`

**Fix**: Implement proper parameter binding or document limitation

**Status**: ✅ WILL FIX

---

## 4. Incorrect Date Offset in Cassandra Types
**Severity**: BLOCKER  
**File**: `src-tauri/src/db_drivers/cassandra/types.rs:62`  
**Issue**: `days - 2_i64.pow(31)` causes massive date shift

**Fix**: Remove incorrect offset: `chrono::Duration::days(days)`

**Status**: ✅ WILL FIX

---

## 5. File Path Validation Missing
**Severity**: WARNING  
**File**: `src-tauri/src/commands/database.rs:1369`  
**Issue**: No path traversal protection in CSV/JSON preview

**Fix**: Validate paths against allowed directories

**Status**: ✅ WILL FIX

---

## 6. Cargo.toml.backup Committed
**Severity**: WARNING  
**File**: `src-tauri/Cargo.toml.backup`  
**Issue**: Backup file in repository

**Fix**: Remove file and add to .gitignore

**Status**: ✅ WILL FIX

---

## False Positives (No Action Required)

### ❌ Scope Mismatch
Reviewer confused this PR with different feature (Remote/RDP)

### ❌ Missing encrypted_password Field
Field is intentionally not exposed to frontend (security by design)

### ❌ Credential Storage Concern
Credentials ARE encrypted with AES-256-GCM (as documented)

### ❌ IronRDP Path Dependencies
Already handled by CI clone steps

### ❌ SQL UPDATE for Non-SQL Databases
Update functionality correctly returns errors for unsupported databases

---

## Implementation Plan (TDD Approach)

### Phase 1: Write Failing Tests
1. Test identifier validation rejects malicious input
2. Test Cassandra array access handles missing columns
3. Test Cassandra parameters are used
4. Test Cassandra date conversion
5. Test file path validation rejects traversal

### Phase 2: Implement Fixes
1. Add identifier validation
2. Fix unsafe array access
3. Implement Cassandra parameter binding
4. Fix date offset calculation
5. Add path validation

### Phase 3: Verify
1. All new tests pass
2. Existing tests still pass
3. Clippy clean
4. fmt clean

---

## Next Steps

1. Create test file: `src-tauri/tests/pr196_fixes.rs`
2. Write failing tests for each issue
3. Implement fixes
4. Commit with descriptive message
5. Update this document with results
