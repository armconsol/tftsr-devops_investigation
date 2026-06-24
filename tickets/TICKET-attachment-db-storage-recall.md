# Ticket: Attachment DB Storage & Cross-Incident Recall

**Branch:** `feature/attachment-db-storage-recall`
**Base:** `master`

---

## Description

Log file and image attachment records previously stored only metadata and filesystem paths, making content volatile — if the source file moved or was deleted, the attachment record became orphaned. There was also no mechanism to search or recall attachments across incidents.

This feature:
1. Stores **gzip-compressed** log text and **raw image bytes** directly in the database, making attachments fully self-contained and portable.
2. Surfaces a new **Attachments tab** on the History page for cross-incident search and recall.
3. Exposes content-retrieval commands so the AI chat context can reference log content from DB on demand, with no disk dependency.

---

## Acceptance Criteria

- [x] Uploading a log file stores gzip-compressed text in `log_files.content_compressed` (BLOB)
- [x] Uploading an image stores raw bytes in `image_attachments.image_data` (BLOB)
- [x] `get_log_file_content` returns decompressed text from DB; falls back to disk for pre-migration records
- [x] `get_image_attachment_data` returns base64 data URL from DB; falls back to disk for pre-migration records
- [x] `list_all_log_files` returns cross-incident log summaries with joined issue title, supports search and issueId filter
- [x] `list_all_image_attachments` returns cross-incident image summaries with joined issue title, supports search and issueId filter
- [x] History page shows two tabs: **Issues** (existing, unchanged) and **Attachments** (new)
- [x] Attachments tab: Log Files section with filename, incident link, date, size, type badge, View button
- [x] Attachments tab: Images section with 48px thumbnail, filename, incident link, date, View button
- [x] "View" on log file → modal showing decompressed plain text
- [x] "View" on image → modal showing full-size image
- [x] Existing records with NULL content fall back to disk read — no breakage for pre-migration data
- [x] All new DB changes tracked via migrations 020–022 with idempotency guarantees
- [x] Wiki documentation updated: IPC-Commands.md and Database.md

---

## Work Implemented

### Database (`src-tauri/src/db/`)

| File | Change |
|---|---|
| `migrations.rs` | Migrations 020 (`content_compressed BLOB`), 021 (`image_data BLOB`), 022 (views `v_log_files_with_issue` + `v_image_attachments_with_issue`). Extended duplicate-column graceful handling for new ALTER TABLE migrations. |
| `models.rs` | Added `LogFileSummary` and `ImageAttachmentSummary` structs for lightweight cross-incident list views (no BLOB fields — content stays out of IPC). |

### Rust Backend (`src-tauri/src/commands/`)

| File | Change |
|---|---|
| `analysis.rs` | Private `compress_text` / `decompress_text` helpers (flate2/miniz_oxide — pure Rust, no system binary). Updated `upload_log_file` and `upload_log_file_by_content` INSERTs to store `content_compressed`. New commands: `get_log_file_content`, `list_all_log_files`. |
| `image.rs` | Updated `upload_image_attachment`, `upload_image_attachment_by_content`, `upload_paste_image` INSERTs to store `image_data`. New commands: `get_image_attachment_data`, `list_all_image_attachments`. |
| `lib.rs` | Registered all 4 new commands. |

### Dependencies (`src-tauri/Cargo.toml`)

- Added `flate2 = { version = "1", features = ["rust_backend"] }` — pure-Rust gzip, portable cross-platform.

### Frontend (`src/`)

| File | Change |
|---|---|
| `lib/tauriCommands.ts` | Added `LogFileSummary`, `ImageAttachmentSummary` interfaces and 4 typed command wrappers. |
| `stores/attachmentStore.ts` | New Zustand store: `loadAttachments`, `searchAttachments`, `setSearchQuery`. |
| `pages/History/index.tsx` | Added tab bar; extracted `IssuesTab` (existing content, unchanged); added `AttachmentsTab` with log/image tables, search, View modals, and lazy `ImageThumbnail` component. |

### Documentation (`docs/wiki/`)

| File | Change |
|---|---|
| `IPC-Commands.md` | Documented `get_log_file_content`, `list_all_log_files`, `get_image_attachment_data`, `list_all_image_attachments` with TypeScript signatures and interface shapes. Updated upload command notes. |
| `Database.md` | Updated migration count (18 → 22). Documented migrations 020, 021, 022 with SQL, rationale, and usage notes. |

---

## Testing Needed

### Automated (already passing)

| Suite | Count | Status |
|---|---|---|
| Rust unit tests (`cargo test`) | 226 | ✅ All pass |
| Frontend unit tests (`npm run test:run`) | 103 | ✅ All pass |
| TypeScript type check (`tsc --noEmit`) | — | ✅ Clean |
| Rust clippy (`clippy -- -D warnings`) | — | ✅ Zero warnings |
| Rust format (`fmt --check`) | — | ✅ Clean |

New tests added:
- `test_compress_decompress_roundtrip`, `test_compress_large_text_is_smaller`, `test_decompress_invalid_bytes_returns_error` (Rust, `analysis.rs`)
- `test_get_image_attachment_data_base64_format` (Rust, `image.rs`)
- `test_020_log_content_compressed_column`, `test_021_image_data_column`, `test_022_attachment_views_exist`, `test_022_views_join_issue_title`, `test_020_021_idempotent` (Rust, `migrations.rs`)
- 9 attachment store tests (`tests/unit/attachmentStore.test.ts`)

### Manual Smoke Testing Required

1. **Log upload → DB content storage**
   - Create issue → upload `.log` file → inspect SQLite: `SELECT id, LENGTH(content_compressed) FROM log_files` — verify non-NULL non-zero value

2. **Content retrieval from DB**
   - History → Attachments tab → Log Files → click "View" → confirm readable decompressed text appears in modal

3. **Fallback for pre-migration records**
   - Manually `UPDATE log_files SET content_compressed = NULL WHERE id = '<id>'` → View should still load from disk path

4. **Image upload → DB byte storage**
   - Upload image → `SELECT id, LENGTH(image_data) FROM image_attachments` — verify non-NULL

5. **Image display**
   - History → Attachments tab → Images → thumbnails should render, View → full-size image modal

6. **Cross-incident search**
   - Create 2+ issues with different log files → Attachments tab → search by partial filename → correct files appear

7. **Issue link navigation**
   - Click incident title in Attachments tab → navigates to correct triage page

8. **Issue tab unchanged**
   - Verify existing Issues tab retains all functionality (search, filter, sort, open, export buttons)
