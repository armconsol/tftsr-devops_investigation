# IPC Commands

All backend commands are typed wrappers in `src/lib/tauriCommands.ts`. The Rust handlers live in `src-tauri/src/commands/`.

---

## Database Commands

### `create_issue`
```typescript
createIssueCmd(title: string, description: string, severity: string, category: string) → Issue
```
Creates a new issue. Generates UUID v7. Returns the created `Issue`.

### `get_issue`
```typescript
getIssueCmd(issueId: string) → IssueDetail
```
Returns a **nested** `IssueDetail` — use `detail.issue.title`, not `detail.title`.
```typescript
interface IssueDetail {
    issue: Issue;
    log_files: LogFile[];
    resolution_steps: ResolutionStep[];
    conversations: AiConversation[];
}
```

### `list_issues`
```typescript
listIssuesCmd(query: IssueListQuery) → IssueSummary[]
```
Paginated list. Supports filter by status, severity, category; sort by created_at/updated_at.

### `update_issue`
```typescript
updateIssueCmd(issueId: string, updates: Partial<IssueUpdate>) → IssueDetail
```
Partial update. Only provided fields are changed.

### `delete_issue`
```typescript
deleteIssueCmd(issueId: string) → void
```
Cascades: deletes log_files, pii_spans, conversations, messages, resolution_steps, documents.

### `search_issues`
```typescript
searchIssuesCmd(query: string) → IssueSummary[]
```
Full-text search via FTS5 virtual table on title + description.

### `add_five_why`
```typescript
addFiveWhyCmd(issueId: string, whyNumber: number, question: string, answer?: string) → FiveWhyEntry
```
Adds a 5-Whys entry (step 1–5). `whyNumber` maps to `step_order`.

### `update_five_why`
```typescript
updateFiveWhyCmd(entryId: string, answer: string) → void
```
Sets or updates the answer for an existing 5-Whys entry.

### `add_timeline_event`
```typescript
addTimelineEventCmd(issueId: string, eventType: string, description: string) → TimelineEvent
```
Records a timestamped event in the issue timeline.

---

## Analysis / PII Commands

### `upload_log_file`
```typescript
uploadLogFileCmd(issueId: string, filePath: string) → LogFile
```
Reads the file from disk, computes SHA-256, stores metadata in DB. Returns `LogFile` record.

### `detect_pii`
```typescript
detectPiiCmd(logFileId: string) → PiiDetectionResult
```
Runs 13 PII patterns on the file content. Returns non-overlapping `PiiSpan[]`.
```typescript
interface PiiDetectionResult {
    log_file_id: string;
    spans: PiiSpan[];
    total_found: number;
}
```

### `apply_redactions`
```typescript
applyRedactionsCmd(logFileId: string, approvedSpanIds: string[]) → RedactedLogFile
```
Rewrites file content with approved redactions. Records SHA-256 in audit log. Returns redacted content path.

---

## AI Commands

### `analyze_logs`
```typescript
analyzeLogsCmd(issueId: string, logFileIds: string[], providerConfig: ProviderConfig) → AnalysisResult
```
Sends selected (redacted) log files to the AI provider with an analysis prompt.

### `chat_message`
```typescript
chatMessageCmd(issueId: string, message: string, providerConfig: ProviderConfig) → ChatResponse
```
Sends a message in the ongoing triage conversation. Domain system prompt is injected automatically on first message. AI response is parsed for why-level indicators (1–5).

### `list_providers`
```typescript
listProvidersCmd() → ProviderInfo[]
```
Returns the list of supported providers with their available models and configuration schema.

---

## Document Commands

### `generate_rca`
```typescript
generateRcaCmd(issueId: string) → Document
```
Builds an RCA Markdown document from the issue data, 5-Whys answers, and timeline.

### `generate_postmortem`
```typescript
generatePostmortemCmd(issueId: string) → Document
```
Builds a blameless post-mortem Markdown document.

### `update_document`
```typescript
updateDocumentCmd(docId: string, contentMd: string) → Document
```
Saves edited Markdown content back to the database.

### `export_document`
```typescript
exportDocumentCmd(docId: string, format: 'md' | 'pdf', outputDir: string) → string
```
Exports document to file. Returns the absolute path of the written file. PDF generation uses `printpdf`.

---

## System / Ollama Commands

### `check_ollama_installed`
```typescript
checkOllamaInstalledCmd() → OllamaStatus
```
Checks if Ollama is running on the configured URL (default: `localhost:11434`).

### `get_ollama_install_guide`
```typescript
getOllamaInstallGuideCmd(platform: string) → InstallGuide
```
Returns platform-specific install instructions for Ollama.

### `list_ollama_models`
```typescript
listOllamaModelsCmd() → OllamaModel[]
```
Lists all locally available Ollama models.

### `pull_ollama_model`
```typescript
pullOllamaModelCmd(modelName: string) → void
```
Downloads a model from the Ollama registry. Streams progress.

### `delete_ollama_model`
```typescript
deleteOllamaModelCmd(modelName: string) → void
```
Removes a model from local storage.

### `detect_hardware`
```typescript
detectHardwareCmd() → HardwareInfo
```
Probes CPU, RAM, GPU. Returns hardware specifications.

### `recommend_models`
```typescript
recommendModelsCmd() → ModelRecommendation[]
```
Returns model recommendations based on detected hardware.
```typescript
interface ModelRecommendation {
    name: string;
    size: string;   // e.g., "2.0 GB" — a String, not a number
    reason: string;
}
```

### `get_settings`
```typescript
getSettingsCmd() → AppSettings
```
Reads app settings from the `settings` table.

### `update_settings`
```typescript
updateSettingsCmd(partial: Partial<AppSettings>) → AppSettings
```
Merges partial settings and persists to DB.

### `get_audit_log`
```typescript
getAuditLogCmd(filter: AuditLogFilter) → AuditEntry[]
```
Returns audit log entries. Filter by action, entity_type, date range.

---

## Integration Commands (v0.2 Stubs)

All 6 integration commands currently return `"not yet available"` errors.

| Command | Purpose |
|---------|---------|
| `test_confluence_connection` | Verify Confluence credentials |
| `publish_to_confluence` | Publish RCA/postmortem to Confluence space |
| `test_servicenow_connection` | Verify ServiceNow credentials |
| `create_servicenow_incident` | Create incident from issue |
| `test_azuredevops_connection` | Verify Azure DevOps credentials |
| `create_azuredevops_workitem` | Create work item from issue |
