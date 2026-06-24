# TICKET: PII Detection Bypass in AI Chat

**Branch**: `fix/pii-detection-bypass`

---

## Description

Two PII detection bypasses were identified and fixed in the AI triage chat interface.

### Bypass 1 — File Attachments (Critical)

When a user attached a file to a chat message, its content was read via `readTextFile()`, sliced to 8 KB, and embedded directly into the AI message string — bypassing the PII pipeline entirely. The message was forwarded to the configured AI provider in plaintext with no redaction marker in the audit log.

**Root cause**: `handleAttach` stored raw file content in React state. `handleSend` concatenated it into `aiMessage` with no PII check. The backend `chat_message` command applied no validation.

### Bypass 2 — Typed Chat Messages (High)

Plain typed chat messages were sent to the AI provider without any PII scan. A user typing `How secure is my password: abc123!!` would have the password forwarded to the AI and persisted in the audit log in plaintext.

### Related Fix — Wrong Return Type on `detect_pii`

`detect_pii` was serialising `pii::PiiDetectionResult` (`spans`, `original_text`) while the TypeScript interface expected `db::models::PiiDetectionResult` (`detections`, `total_pii_found`). All frontend code reading `result.detections` received `undefined`, meaning the LogUpload PII review workflow was silently broken.

---

## Design Decision: Auto-Redact, Not Block

After initial implementation explored a blocking/warn-then-proceed approach, the product decision was made to **auto-redact PII in-place and send**:

- File attachments: PII is detected on full file content and replaced with type tokens (`[Password]`, `[Email]`, etc.) before the content is embedded in the AI message. The redacted form is stored in the DB and audit log.
- Typed messages: Same auto-redact applied to the user's typed text before the message is sent to the AI provider.
- The user's chat bubble is updated after the response to show the redacted form — users can see exactly what reached the AI.
- The audit log records `was_pii_redacted: bool` and `pii_types_redacted: [...]` alongside the redacted message.
- No user blocking or acknowledgment flow. PII is handled transparently.

---

## Acceptance Criteria

- [x] Attaching a text file containing PII sends successfully; content is auto-redacted before the AI sees it
- [x] Attaching a clean text file proceeds normally with no modification
- [x] PII detection runs on the full file content before truncating to the 8 KB embed limit (no PII straddling the boundary)
- [x] Typed messages containing PII are auto-redacted before being sent to the AI provider
- [x] The chat bubble is updated post-send to show the redacted form of the user's message
- [x] The audit log records `was_pii_redacted`, `pii_types_redacted`, and the full redacted `user_message`
- [x] `detectPiiCmd` returns `detections: PiiSpan[]` and `total_pii_found: number` matching the TypeScript contract
- [x] `chatMessageCmd` passes `logFileIds` as `undefined` (not `null`) when no files are attached
- [x] `scan_text_for_pii` rejects inputs over 32 KB to prevent DoS
- [x] `response.user_message ?? message` used as bubble fallback — no `"undefined..."` concatenation
- [x] All Rust and frontend tests pass; zero clippy warnings; `cargo fmt --check` clean; tsc clean

---

## Work Implemented

### `src-tauri/src/ai/mod.rs`
- Added `user_message: Option<String>` to `ChatResponse` — set by `chat_message`, absent from direct provider calls

### `src-tauri/src/ai/anthropic.rs`, `gemini.rs`, `mistral.rs`, `ollama.rs`, `openai.rs`
- Added `user_message: None` to all `ChatResponse { ... }` constructors

### `src-tauri/src/commands/ai.rs`
- `chat_message` now accepts `log_file_ids: Option<Vec<String>>`
- Step 1: auto-redacts the typed message text with `PiiDetector` + `apply_redactions`
- Step 2: loads each attachment from DB, detects PII on **full file content**, applies redactions, then truncates to 8 KB at a valid UTF-8 char boundary
- Tracks `was_pii_redacted` and `redacted_pii_types` across both steps
- Audit log includes `was_pii_redacted: bool` and `pii_types_redacted: [...]`
- Returns `user_message: Some(stored_user_message)` in `ChatResponse`

### `src-tauri/src/commands/analysis.rs`
- Fixed `detect_pii` return type from `pii::PiiDetectionResult` to `db::models::PiiDetectionResult`
- Added `scan_text_for_pii(text: String)` with 32 KB input cap

### `src-tauri/src/lib.rs`
- Registered `scan_text_for_pii`

### `src/lib/tauriCommands.ts`
- `ChatResponse` interface: added `user_message?: string`
- `chatMessageCmd` signature: added `logFileIds: string[]`; passes `undefined` when empty
- Added `scanTextForPiiCmd` wrapper

### `src/stores/sessionStore.ts`
- Added `updateMessageContent(id, content)` action

### `src/pages/Triage/index.tsx`
- `PendingFile` type: `{ name: string; logFileId: string }` — no raw content stored
- `handleAttach`: only uploads the file and stores `logFileId`; no `readTextFile`
- `handleSend`: passes `logFileIds` to backend; after response updates the bubble with `(response.user_message ?? message) + suffix`

---

## Testing Needed

1. Attach a file containing `password: secret123` → message sends; chat bubble shows `[Password]` in the embedded content; no plaintext credential in bubble or DB
2. Attach a clean text file → content appears unmodified in the chat context
3. Attach a file where PII appears near the 8000-byte mark → content is fully redacted before truncation
4. Type `My password is abc123!!` → message sends; bubble shows `My [Password] is [Password]`
5. On LogUpload page, upload a file with a known IP/email → PII spans appear in the review UI
6. Check audit log after a PII-containing message: `was_pii_redacted: true`, `pii_types_redacted` populated
7. Check audit log after a clean message: `was_pii_redacted: false`, `pii_types_redacted: []`
8. `cargo test` → 228/228 pass; `npm run test:run` → 103/103 pass; `cargo fmt --check` clean; `npx tsc --noEmit` clean
