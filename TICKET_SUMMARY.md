# Ticket Summary - UI Fixes and Audit Log Enhancement

## Description

This ticket addresses multiple UI and functionality issues reported in the tftsr-devops_investigation application:

1. **Download Icons Visibility**: Download icons (PDF, DOCX) in RCA and Post-Mortem pages were not visible in dark theme
2. **Export File System Error**: "Read-only file system (os error 30)" error when attempting to export documents
3. **History Search Button**: Search button not visible in the History page
4. **Domain Filtering**: Domain-only filtering not working in History page
5. **Audit Log Enhancement**: Audit log showed only internal IDs, lacking actual transmitted data for security auditing

## Acceptance Criteria

- [ ] Download icons are visible in both light and dark themes on RCA and Post-Mortem pages
- [ ] Documents can be exported successfully to Downloads directory without filesystem errors
- [ ] Search button is visible with proper styling in History page
- [ ] Domain filter works independently without requiring a search query
- [ ] Audit log displays full transmitted data including:
  - AI chat messages with provider details, user message, and response preview
  - Document generation with content preview and metadata
  - All entries show properly formatted JSON with details

## Work Implemented

### 1. Download Icons Visibility Fix
**Files Modified:**
- `src/components/DocEditor.tsx:60-67`

**Changes:**
- Added `text-foreground` class to Download icons for PDF and DOCX buttons
- Ensures icons inherit the current theme's foreground color for visibility

### 2. Export File System Error Fix
**Files Modified:**
- `src-tauri/Cargo.toml:38` - Added `dirs = "5"` dependency
- `src-tauri/src/commands/docs.rs:127-170` - Rewrote `export_document` function
- `src/pages/RCA/index.tsx:53-60` - Updated error handling and user feedback
- `src/pages/Postmortem/index.tsx:52-59` - Updated error handling and user feedback

**Changes:**
- Modified `export_document` to use Downloads directory by default instead of "."
- Falls back to `app_data_dir/exports` if Downloads directory unavailable
- Added proper directory creation with error handling
- Updated frontend to show success message with file path
- Empty `output_dir` parameter now triggers default behavior

### 3. Search Button Visibility Fix
**Files Modified:**
- `src/pages/History/index.tsx:124-127`

**Changes:**
- Changed button from `variant="outline"` to default variant
- Added Search icon to button for better visibility
- Button now has proper contrast in both themes

### 4. Domain-Only Filtering Fix
**Files Modified:**
- `src-tauri/src/commands/db.rs:305-312`

**Changes:**
- Added missing `filter.domain` handling in `list_issues` function
- Domain filter now properly filters by `i.category` field
- Filter works independently of search query

### 5. Audit Log Enhancement
**Files Modified:**
- `src-tauri/src/commands/ai.rs:242-266` - Enhanced AI chat audit logging
- `src-tauri/src/commands/docs.rs:44-73` - Enhanced RCA generation audit logging
- `src-tauri/src/commands/docs.rs:90-119` - Enhanced postmortem generation audit logging
- `src/pages/Settings/Security.tsx:191-206` - Enhanced audit log display

**Changes:**
- AI chat audit now captures:
  - Provider name, model, and API URL
  - Full user message
  - Response preview (first 200 chars)
  - Token count
- Document generation audit now captures:
  - Issue ID and title
  - Document type and title
  - Content length and preview (first 300 chars)
- Security page now displays:
  - Pretty-printed JSON with proper formatting
  - Entry ID and entity type below the data
  - Better layout with whitespace handling

## Testing Needed

### Manual Testing

1. **Download Icons Visibility**
   - [ ] Open RCA page in light theme
   - [ ] Verify PDF and DOCX download icons are visible
   - [ ] Switch to dark theme
   - [ ] Verify PDF and DOCX download icons are still visible

2. **Export Functionality**
   - [ ] Generate an RCA document
   - [ ] Click "PDF" export button
   - [ ] Verify file is created in Downloads directory
   - [ ] Verify success message displays with file path
   - [ ] Check file opens correctly
   - [ ] Repeat for "MD" and "DOCX" formats
   - [ ] Test on Post-Mortem page as well

3. **History Search Button**
   - [ ] Navigate to History page
   - [ ] Verify Search button is visible
   - [ ] Verify button has search icon
   - [ ] Test button in both light and dark themes

4. **Domain Filtering**
   - [ ] Navigate to History page
   - [ ] Select a domain from dropdown (e.g., "Linux")
   - [ ] Do NOT enter any search text
   - [ ] Verify issues are filtered by selected domain
   - [ ] Change domain selection
   - [ ] Verify filtering updates correctly

5. **Audit Log**
   - [ ] Perform an AI chat interaction
   - [ ] Navigate to Settings > Security > Audit Log
   - [ ] Click "View" on a recent entry
   - [ ] Verify transmitted data shows:
     - Provider details
     - User message
     - Response preview
   - [ ] Generate an RCA or Post-Mortem
   - [ ] Check audit log for document generation entry
   - [ ] Verify content preview and metadata are visible

### Automated Testing

```bash
# Type checking
npx tsc --noEmit

# Rust compilation
cargo check --manifest-path src-tauri/Cargo.toml

# Rust linting
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Frontend tests (if applicable)
npm run test:run
```

### Edge Cases to Test

- Export when Downloads directory doesn't exist
- Export with very long document titles (special character handling)
- Domain filter with empty result set
- Audit log with very large payloads (>1000 chars)
- Audit log JSON parsing errors (malformed data)
