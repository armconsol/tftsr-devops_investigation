# TICKET: PII Detection Bypass in AI Chat

**Branch**: `feature/attachment-db-storage-recall`

---

## Description

Two PII detection bypasses were identified and fixed in the AI triage chat interface.

### Bypass 1 — File Attachments (Critical)

When a user attached a file to a chat message, its content was read via `readTextFile()`, sliced to 8 KB, and embedded directly into the AI message string — bypassing the existing PII pipeline entirely. The message was forwarded to the configured AI provider in plaintext. The audit log recorded the full file content without a SHA-256 hash or redaction marker.

**Root cause**: `handleAttach` stored raw file content in React state. `handleSend` concatenated it into `aiMessage` with no call to `detectPiiCmd` or `applyRedactionsCmd`. The backend `chat_message` command applied no validation.

### Bypass 2 — Typed Chat Messages (High)

Plain typed chat messages were sent to the AI provider without any PII scan. A user typing `How secure is my password: abc123!!` would have the password forwarded to the AI and persisted in the audit log in plaintext.

### Related Fix — Wrong Return Type on `detect_pii`

`detect_pii` was serialising `pii::PiiDetectionResult` (`spans`, `original_text`) while the TypeScript interface expected `db::models::PiiDetectionResult` (`detections`, `total_pii_found`). All frontend code reading `result.detections` received `undefined`, meaning the LogUpload PII review workflow was silently broken. Fixed as part of this work.

---

## Acceptance Criteria

- [x] Attaching a text file containing PII blocks send with a descriptive error naming the file and PII types
- [x] The user is directed to use Log Analysis to redact before attaching
- [x] Attaching a clean text file proceeds normally
- [x] Attaching an image (binary, `content === null`) skips PII scan and proceeds
- [x] Typing a message containing PII triggers a one-time warning; sending the same message a second time proceeds (explicit acknowledgment)
- [x] After a successful send the warning state is cleared
- [x] Backend `chat_message` independently rejects attachment content containing PII, regardless of frontend state
- [x] Backend attachment parser catches headers both with and without trailing `---`
- [x] `detectPiiCmd` returns `detections: PiiSpan[]` and `total_pii_found: number` matching the TypeScript contract
- [x] All Rust and frontend tests pass; zero clippy warnings; tsc clean

---

## Work Implemented

### `src-tauri/src/commands/analysis.rs`
- Fixed `detect_pii` to return `db::models::PiiDetectionResult` (`detections`, `total_pii_found`) instead of `pii::PiiDetectionResult` (`spans`, `original_text`)
- Added `scan_text_for_pii(text: String)` command: scans arbitrary text for PII without creating DB records

### `src-tauri/src/commands/ai.rs`
- Added defence-in-depth PII scan inside `chat_message` before user message is appended to the AI request
- Extracts body content from all `--- Attached:` blocks; catches headers with and without trailing `---`
- Returns a hard error if PII spans are found in attachment content

### `src-tauri/src/lib.rs`
- Registered `scan_text_for_pii` in `generate_handler![]`

### `src/lib/tauriCommands.ts`
- Added `scanTextForPiiCmd(text: string)` wrapper

### `src/pages/Triage/index.tsx`
- Updated `PendingFile` type: added `logFileId: string`
- Stored `logFile.id` when attaching files
- Added attachment PII gate in `handleSend`: calls `detectPiiCmd` on each text attachment; hard-blocks if PII found
- Added message PII warning in `handleSend`: calls `scanTextForPiiCmd` on typed message; warns once; second send with same message proceeds
- Added `piiWarnedMessageRef` to track prior warning state; cleared in `finally` after every send attempt

---

## Testing Needed

1. Attach a file containing `password: secret123` → send is blocked; error names the file and PII type
2. Attach a clean text file → send proceeds
3. Attach an image (.png) → no PII scan, send proceeds
4. Type `My password is abc123!!` in chat → first send shows PII warning
5. Send the same message again → proceeds (acknowledgment)
6. Send a different message → prior warning cleared, sends normally
7. On LogUpload page, upload a file with a known IP/email → confirm PII spans appear in the review UI (was previously broken due to wrong struct)
8. Directly call `chat_message` IPC with a message containing `--- Attached: test ---\npassword: secret` → backend returns error
9. `cargo test` → 228/228 pass; `npm run test:run` → 103/103 pass
