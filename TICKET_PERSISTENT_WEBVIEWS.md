# Ticket Summary - Persistent Browser Windows for Integration Authentication

## Description

Implement persistent browser window sessions for integration authentication (Confluence, Azure DevOps, ServiceNow). Browser windows now persist across application restarts, eliminating the need to extract HttpOnly cookies via JavaScript (which fails due to browser security restrictions).

This follows a Playwright-style "piggyback" authentication approach where the browser window maintains its own internal cookie store, allowing the user to log in once and have the session persist indefinitely until they manually close the window.

## Acceptance Criteria

- [x] Integration browser windows persist to database when created
- [x] Browser windows are automatically restored on app startup
- [x] Cookies are maintained automatically by the browser's internal store (no JavaScript extraction of HttpOnly cookies)
- [x] Windows can be manually closed by the user, which removes them from persistence
- [x] Database migration creates `persistent_webviews` table
- [x] Window close events are handled to update database and in-memory tracking

## Work Implemented

### 1. Database Migration for Persistent Webviews

**Files Modified:**
- `src-tauri/src/db/migrations.rs:154-167`

**Changes:**
- Added migration `013_create_persistent_webviews` to create the `persistent_webviews` table
- Table schema includes:
  - `id` (TEXT PRIMARY KEY)
  - `service` (TEXT with CHECK constraint for 'confluence', 'servicenow', 'azuredevops')
  - `webview_label` (TEXT - the Tauri window identifier)
  - `base_url` (TEXT - the integration base URL)
  - `last_active` (TEXT timestamp, defaults to now)
  - `window_x`, `window_y`, `window_width`, `window_height` (INTEGER - for future window position persistence)
  - UNIQUE constraint on `service` (one browser window per integration)

### 2. Webview Persistence on Creation

**Files Modified:**
- `src-tauri/src/commands/integrations.rs:531-591`

**Changes:**
- Modified `authenticate_with_webview` command to persist webview state to database after creation
- Stores service name, webview label, and base URL
- Logs persistence operation for debugging
- Sets up window close event handler to remove webview from tracking and database
- Event handler properly clones Arc fields for `'static` lifetime requirement
- Updated success message to inform user that window persists across restarts

### 3. Webview Restoration on App Startup

**Files Modified:**
- `src-tauri/src/commands/integrations.rs:793-865` - Added `restore_persistent_webviews` function
- `src-tauri/src/lib.rs:60-84` - Added `.setup()` hook to call restoration

**Changes:**
- Added `restore_persistent_webviews` async function that:
  - Queries `persistent_webviews` table for all saved webviews
  - Recreates each webview window by calling `authenticate_with_webview`
  - Updates in-memory tracking map
  - Removes from database if restoration fails
  - Logs all operations for debugging
- Updated `lib.rs` to call restoration in `.setup()` hook:
  - Clones Arc fields from `AppState` for `'static` lifetime
  - Spawns async task to restore webviews
  - Logs warnings if restoration fails

### 4. Window Close Event Handling

**Files Modified:**
- `src-tauri/src/commands/integrations.rs:559-591`

**Changes:**
- Added `on_window_event` listener to detect window close events
- On `CloseRequested` event:
  - Spawns async task to clean up
  - Removes service from in-memory `integration_webviews` map
  - Deletes entry from `persistent_webviews` database table
  - Logs all cleanup operations
- Properly handles Arc cloning to avoid lifetime issues in spawned task

### 5. Removed Auto-Close Behavior

**Files Modified:**
- `src-tauri/src/commands/integrations.rs:606-618`

**Changes:**
- Removed automatic window closing in `extract_cookies_from_webview`
- Windows now stay open after cookie extraction
- Updated success message to inform user that window persists for future use

### 6. Frontend UI Update - Removed "Complete Login" Button

**Files Modified:**
- `src/pages/Settings/Integrations.tsx:371-409` - Updated webview authentication UI
- `src/pages/Settings/Integrations.tsx:140-165` - Simplified `handleConnectWebview`
- `src/pages/Settings/Integrations.tsx:167-200` - Removed `handleCompleteWebviewLogin` function
- `src/pages/Settings/Integrations.tsx:16-26` - Removed unused `extractCookiesFromWebviewCmd` import
- `src/pages/Settings/Integrations.tsx:670-677` - Updated authentication method comparison text

**Changes:**
- Removed "Complete Login" button that tried to extract cookies via JavaScript
- Updated UI to show success message when browser opens, explaining persistence
- Removed confusing two-step flow (open browser → complete login)
- New flow: click "Open Browser" → log in → leave window open (that's it!)
- Updated description text to explain persistent window behavior
- Mark integration as "connected" immediately when browser opens
- Removed unused function and import for cookie extraction

### 7. Unused Import Cleanup

**Files Modified:**
- `src-tauri/src/integrations/webview_auth.rs:2`
- `src-tauri/src/lib.rs:13` - Added `use tauri::Manager;`

**Changes:**
- Removed unused `Listener` import from webview_auth.rs
- Added `Manager` trait import to lib.rs for `.state()` method

## Testing Needed

### Manual Testing

1. **Initial Browser Window Creation**
   - [ ] Navigate to Settings > Integrations
   - [ ] Configure a Confluence integration with base URL
   - [ ] Click "Open Browser" button
   - [ ] Verify browser window opens with Confluence login page
   - [ ] Complete login in the browser window
   - [ ] Verify window stays open after login

2. **Window Persistence Across Restarts**
   - [ ] With Confluence browser window open, close the main application
   - [ ] Relaunch the application
   - [ ] Verify Confluence browser window is automatically restored
   - [ ] Verify you are still logged in (cookies maintained)
   - [ ] Navigate to different pages in Confluence to verify session works

3. **Manual Window Close**
   - [ ] With browser window open, manually close it (X button)
   - [ ] Restart the application
   - [ ] Verify browser window does NOT reopen (removed from persistence)

4. **Database Verification**
   - [ ] Open database: `sqlite3 ~/Library/Application\ Support/trcaa/data.db`
   - [ ] Run: `SELECT * FROM persistent_webviews;`
   - [ ] Verify entry exists when window is open
   - [ ] Close window and verify entry is removed

5. **Multiple Integration Windows**
   - [ ] Open browser window for Confluence
   - [ ] Open browser window for Azure DevOps
   - [ ] Restart application
   - [ ] Verify both windows are restored
   - [ ] Close one window
   - [ ] Verify only one is removed from database
   - [ ] Restart and verify remaining window still restores

6. **Cookie Persistence (No HttpOnly Extraction Needed)**
   - [ ] Log into Confluence browser window
   - [ ] Close main application
   - [ ] Relaunch application
   - [ ] Navigate to a Confluence page that requires authentication
   - [ ] Verify you are still logged in (cookies maintained by browser)

### Automated Testing

```bash
# Type checking
npx tsc --noEmit

# Rust compilation
cargo check --manifest-path src-tauri/Cargo.toml

# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Rust linting
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

### Edge Cases to Test

- Application crash while browser window is open (verify restoration on next launch)
- Database corruption (verify graceful handling of restore failures)
- Window already exists when trying to create duplicate (verify existing window is focused)
- Network connectivity lost during window restoration (verify error handling)
- Multiple rapid window open/close cycles (verify database consistency)

## Architecture Notes

### Design Decision: Persistent Windows vs Cookie Extraction

**Problem:** HttpOnly cookies cannot be accessed via JavaScript (`document.cookie`), which broke the original cookie extraction approach for Confluence and other services.

**Solution:** Instead of extracting cookies, keep the browser window alive across app restarts:
- Browser maintains its own internal cookie store (includes HttpOnly cookies)
- Cookies are automatically sent with all HTTP requests from the browser
- No need for JavaScript extraction or manual token management
- Matches Playwright's approach of persistent browser contexts

### Lifecycle Flow

1. **Window Creation:** User clicks "Open Browser" → `authenticate_with_webview` creates window → State saved to database
2. **App Running:** Window stays open, user can browse freely, cookies maintained by browser
3. **Window Close:** User closes window → Event handler removes from database and memory
4. **App Restart:** `restore_persistent_webviews` queries database → Recreates all windows → Windows resume with original cookies

### Database Schema

```sql
CREATE TABLE persistent_webviews (
    id TEXT PRIMARY KEY,
    service TEXT NOT NULL CHECK(service IN ('confluence','servicenow','azuredevops')),
    webview_label TEXT NOT NULL,
    base_url TEXT NOT NULL,
    last_active TEXT NOT NULL DEFAULT (datetime('now')),
    window_x INTEGER,
    window_y INTEGER,
    window_width INTEGER,
    window_height INTEGER,
    UNIQUE(service)
);
```

### Future Enhancements

- [ ] Save and restore window position/size (columns already exist in schema)
- [ ] Add "last_active" timestamp updates on window focus events
- [ ] Implement "Close All Windows" command for cleanup
- [ ] Add visual indicator in main UI showing which integrations have active browser windows
- [ ] Implement session timeout logic (close windows after X days of inactivity)

## Related Files

- `src-tauri/src/db/migrations.rs` - Database schema migration
- `src-tauri/src/commands/integrations.rs` - Webview persistence and restoration logic
- `src-tauri/src/integrations/webview_auth.rs` - Browser window creation
- `src-tauri/src/lib.rs` - App startup hook for restoration
- `src-tauri/src/state.rs` - AppState structure with `integration_webviews` map

## Security Considerations

- Cookie storage remains in the browser's internal secure store (not extracted to database)
- Database only stores window metadata (service, label, URL)
- No credential information persisted beyond what the browser already maintains
- Audit log still tracks all integration API calls separately

## Migration Path

Users upgrading to this version will:
1. See new database migration `013_create_persistent_webviews` applied automatically
2. Existing integrations continue to work (migration is additive only)
3. First time opening a browser window will persist it for future sessions
4. No manual action required from users
