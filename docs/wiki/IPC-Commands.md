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

### `get_timeline_events`
```typescript
getTimelineEventsCmd(issueId: string) → TimelineEvent[]
```
Retrieves all timeline events for an issue, ordered by created_at ascending.
```typescript
interface TimelineEvent {
    id: string;
    issue_id: string;
    event_type: string;          // One of: triage_started, log_uploaded, why_level_advanced, etc.
    description: string;
    metadata?: Record<string, any>;  // Event-specific JSON data
    created_at: string;               // UTC timestamp
}
```

### `add_timeline_event`
```typescript
addTimelineEventCmd(issueId: string, eventType: string, description: string, metadata?: Record<string, any>) → TimelineEvent
```
Records a timestamped event in the issue timeline. Dual-writes to both `timeline_events` (for document generation) and `audit_log` (for security audit trail).

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

### `get_log_file_content`
```typescript
getLogFileContentCmd(logFileId: string) → string
```
Returns the plain-text content of a log file. Primary path: reads gzip-compressed BLOB from the `content_compressed` column and decompresses in-process (no external binary required). Fallback: reads from `file_path` on disk for records uploaded before migration 020.

Used by the triage chat context loader and the "View" modal in the Attachments tab.

### `list_all_log_files`
```typescript
listAllLogFilesCmd(search?: string, issueId?: string) → LogFileSummary[]
```
Cross-incident log file listing via `v_log_files_with_issue`. Optional `search` performs `file_name LIKE '%q%'`; optional `issueId` filters to a single incident. Ordered by `uploaded_at DESC`. Never includes the compressed content blob — content is fetched separately via `get_log_file_content`.
```typescript
interface LogFileSummary {
    id: string;
    issue_id: string;
    issue_title: string;   // joined from issues table
    file_name: string;
    file_path: string;
    file_size: number;
    mime_type: string;
    content_hash: string;
    uploaded_at: string;
    redacted: boolean;
}
```

---

## Image Attachment Commands

### `upload_image_attachment`
```typescript
uploadImageAttachmentCmd(issueId: string, filePath: string, piiWarningAcknowledged: boolean) → ImageAttachment
```
Uploads an image file. Computes SHA-256, stores metadata in DB. Returns `ImageAttachment` record.

### `list_image_attachments`
```typescript
listImageAttachmentsCmd(issueId: string) → ImageAttachment[]
```
Lists all image attachments for an issue.

### `delete_image_attachment`
```typescript
deleteImageAttachmentCmd(imageId: string) → void
```
Deletes an image attachment from disk and database.

### `upload_paste_image`
```typescript
uploadPasteImageCmd(issueId: string, base64Data: string, fileName: string, piiWarningAcknowledged: boolean) → ImageAttachment
```
Uploads an image from clipboard paste (base64). Returns `ImageAttachment` record.

**Note (v0.4+):** All three upload commands (`upload_image_attachment`, `upload_image_attachment_by_content`, `upload_paste_image`) now also store the raw image bytes in the `image_data` column of `image_attachments`, enabling retrieval without requiring the source file on disk.

### `get_image_attachment_data`
```typescript
getImageAttachmentDataCmd(attachmentId: string) → string
```
Returns image content as a base64 data URL (`data:<mime>;base64,...`). Primary path: reads raw bytes from the `image_data` BLOB column. Fallback: reads from `file_path` on disk for records uploaded before migration 021.

Suitable for use directly as an `<img src>` value or in the "View" modal.

### `list_all_image_attachments`
```typescript
listAllImageAttachmentsCmd(search?: string, issueId?: string) → ImageAttachmentSummary[]
```
Cross-incident image listing via `v_image_attachments_with_issue`. Optional `search` performs `file_name LIKE '%q%'`; optional `issueId` filters to a single incident. Ordered by `uploaded_at DESC`. Never includes the raw image bytes blob.
```typescript
interface ImageAttachmentSummary {
    id: string;
    issue_id: string;
    issue_title: string;   // joined from issues table
    file_name: string;
    file_path: string;
    file_size: number;
    mime_type: string;
    upload_hash: string;
    uploaded_at: string;
    pii_warning_acknowledged: boolean;
    is_paste: boolean;
}
```

---

## AI Commands

### `analyze_logs`
```typescript
analyzeLogsCmd(issueId: string, logFileIds: string[], providerConfig: ProviderConfig) → AnalysisResult
```
Sends selected (redacted) log files to the AI provider with an analysis prompt.

### `chat_message`
```typescript
chatMessageCmd(issueId: string, message: string, providerConfig: ProviderConfig, systemPrompt?: string) → ChatResponse
```
Sends a message in the ongoing triage conversation. Optional `systemPrompt` parameter allows prepending domain expertise before conversation history. If not provided, the domain-specific system prompt for the issue category is injected automatically on first message. AI response is parsed for why-level indicators (1–5).

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
Builds an RCA Markdown document from the issue data, 5-Whys answers, and timeline events. Uses real incident response timeline (log uploads, why-level progression, root cause identification) instead of placeholders.

### `generate_postmortem`
```typescript
generatePostmortemCmd(issueId: string) → Document
```
Builds a blameless post-mortem Markdown document. Incorporates timeline events to show the full incident lifecycle: detection, diagnosis, resolution, and post-incident review phases.

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

## Integration Commands

> **Status:** ✅ **Fully Implemented** (v0.2.3+)

All integration commands are production-ready with complete OAuth2/authentication flows.

### OAuth2 Commands

### `initiate_oauth`
```typescript
initiateOauthCmd(service: "confluence" | "servicenow" | "azuredevops") → OAuthInitResponse
```
Starts OAuth2 PKCE flow. Returns authorization URL and state key. Opens browser window for user authentication.

```typescript
interface OAuthInitResponse {
    auth_url: string;  // URL to open in browser
    state: string;     // State key for callback verification
}
```

**Flow:**
1. Generates PKCE challenge
2. Starts local callback server on `http://localhost:8765`
3. Opens authorization URL in browser
4. User authenticates with service
5. Service redirects to callback server
6. Callback server triggers `handle_oauth_callback`

### `handle_oauth_callback`
```typescript
handleOauthCallbackCmd(service: string, code: string, stateKey: string) → void
```
Exchanges authorization code for access token. Encrypts token with AES-256-GCM and stores in database.

### Confluence Commands

### `test_confluence_connection`
```typescript
testConfluenceConnectionCmd(baseUrl: string, credentials: Record<string, unknown>) → ConnectionResult
```
Verifies Confluence connection by calling `/rest/api/user/current`.

### `list_confluence_spaces`
```typescript
listConfluenceSpacesCmd(config: ConfluenceConfig) → Space[]
```
Lists all accessible Confluence spaces.

### `search_confluence_pages`
```typescript
searchConfluencePagesCmd(config: ConfluenceConfig, query: string, spaceKey?: string) → Page[]
```
Searches pages using CQL (Confluence Query Language). Optional space filter.

### `publish_to_confluence`
```typescript
publishToConfluenceCmd(config: ConfluenceConfig, spaceKey: string, title: string, contentHtml: string, parentPageId?: string) → PublishResult
```
Creates a new page in Confluence. Returns page ID and URL.

### `update_confluence_page`
```typescript
updateConfluencePageCmd(config: ConfluenceConfig, pageId: string, title: string, contentHtml: string, version: number) → PublishResult
```
Updates an existing page. Requires current version number.

### ServiceNow Commands

### `test_servicenow_connection`
```typescript
testServiceNowConnectionCmd(instanceUrl: string, credentials: Record<string, unknown>) → ConnectionResult
```
Verifies ServiceNow connection by querying incident table.

### `search_servicenow_incidents`
```typescript
searchServiceNowIncidentsCmd(config: ServiceNowConfig, query: string) → Incident[]
```
Searches incidents by short description. Returns up to 10 results.

### `create_servicenow_incident`
```typescript
createServiceNowIncidentCmd(config: ServiceNowConfig, shortDesc: string, description: string, urgency: string, impact: string) → TicketResult
```
Creates a new incident. Returns incident number and URL.

```typescript
interface TicketResult {
    id: string;           // sys_id (UUID)
    ticket_number: string; // INC0010001
    url: string;          // Direct link to incident
}
```

### `get_servicenow_incident`
```typescript
getServiceNowIncidentCmd(config: ServiceNowConfig, incidentId: string) → Incident
```
Retrieves incident by sys_id or incident number (e.g., `INC0010001`).

### `update_servicenow_incident`
```typescript
updateServiceNowIncidentCmd(config: ServiceNowConfig, sysId: string, updates: Record<string, any>) → TicketResult
```
Updates incident fields. Uses JSON-PATCH format.

### Azure DevOps Commands

### `test_azuredevops_connection`
```typescript
testAzureDevOpsConnectionCmd(orgUrl: string, credentials: Record<string, unknown>) → ConnectionResult
```
Verifies Azure DevOps connection by querying project info.

### `search_azuredevops_workitems`
```typescript
searchAzureDevOpsWorkItemsCmd(config: AzureDevOpsConfig, query: string) → WorkItem[]
```
Searches work items using WIQL (Work Item Query Language).

### `create_azuredevops_workitem`
```typescript
createAzureDevOpsWorkItemCmd(config: AzureDevOpsConfig, title: string, description: string, workItemType: string, severity: string) → TicketResult
```
Creates a work item (Bug, Task, User Story). Returns work item ID and URL.

**Work Item Types:**
- `Bug` — Software defect
- `Task` — Work assignment
- `User Story` — Feature request
- `Issue` — Problem or blocker
- `Incident` — Production incident

### `get_azuredevops_workitem`
```typescript
getAzureDevOpsWorkItemCmd(config: AzureDevOpsConfig, workItemId: number) → WorkItem
```
Retrieves work item by ID.

### `update_azuredevops_workitem`
```typescript
updateAzureDevOpsWorkItemCmd(config: AzureDevOpsConfig, workItemId: number, updates: Record<string, any>) → TicketResult
```
Updates work item fields. Uses JSON-PATCH format.

---

## MCP Server Commands

> **Status:** Fully Implemented (v0.3.0+)

### `list_mcp_servers`
```typescript
listMcpServersCmd() → McpServer[]
```
Returns all registered MCP servers. `auth_value` is always `null` in responses (scrubbed server-side).
```typescript
interface McpServer {
    id: string;
    name: string;
    url: string;
    transport_type: "stdio" | "http";
    transport_config: string;          // JSON
    auth_type: "none" | "api_key" | "bearer" | "oauth2";
    auth_value?: string;               // Always null in responses
    enabled: boolean;
    last_discovered_at?: string;
    discovery_status: "pending" | "connected" | "unreachable" | "error";
    discovery_error?: string;
    created_at: string;
    updated_at: string;
}
```

### `create_mcp_server`
```typescript
createMcpServerCmd(request: CreateMcpServerRequest) → McpServer
```
Creates a new MCP server record. Auth value is encrypted with AES-256-GCM before persistence.
```typescript
interface CreateMcpServerRequest {
    name: string;
    url: string;
    transport_type: "stdio" | "http";
    transport_config: string;
    auth_type: "none" | "api_key" | "bearer" | "oauth2";
    auth_value?: string;
    enabled: boolean;
}
```

### `update_mcp_server`
```typescript
updateMcpServerCmd(id: string, request: UpdateMcpServerRequest) → McpServer
```
Partial update. Only provided fields are changed. If `auth_value` is provided, it replaces the encrypted value.
```typescript
interface UpdateMcpServerRequest {
    name?: string;
    url?: string;
    transport_type?: "stdio" | "http";
    transport_config?: string;
    auth_type?: "none" | "api_key" | "bearer" | "oauth2";
    auth_value?: string;
    enabled?: boolean;
}
```

### `delete_mcp_server`
```typescript
deleteMcpServerCmd(id: string) → void
```
Deletes the server record and all associated tools/resources (cascade). Also removes the live connection from memory.

### `toggle_mcp_server`
```typescript
toggleMcpServerCmd(id: string, enabled: boolean) → void
```
Enables or disables a server. Disabled servers are excluded from AI tool injection and startup discovery.

### `discover_mcp_server`
```typescript
discoverMcpServerCmd(id: string) → McpServerStatus
```
Connects to the server, enumerates its tools and resources, and persists them. Returns the updated status.
```typescript
interface McpServerStatus {
    server_id: string;
    status: "pending" | "connected" | "unreachable" | "error";
    error?: string;
    tool_count: number;
    resource_count: number;
    last_discovered_at?: string;
}
```

### `get_mcp_server_status`
```typescript
getMcpServerStatusCmd(id: string) → McpServerStatus
```
Returns current discovery status, tool count, and resource count without triggering a new connection.

### `initiate_mcp_oauth`
```typescript
initiateMcpOauthCmd(id: string) → void
```
Opens a WebView window for OAuth2 authentication. Requires `auth_type = "oauth2"` and `transport_config` containing `auth_endpoint`, `token_endpoint`, and `client_id`. After successful authentication, the access token is encrypted and stored.

---

## Common Types

### `ConnectionResult`
```typescript
interface ConnectionResult {
    success: boolean;
    message: string;
}
```

### `PublishResult`
```typescript
interface PublishResult {
    id: string;     // Page ID or document ID
    url: string;    // Direct link to published content
}
```

### `TicketResult`
```typescript
interface TicketResult {
    id: string;           // sys_id or work item ID
    ticket_number: string; // Human-readable number
    url: string;          // Direct link
}
```

---

## Authentication Storage

All integration credentials are stored in the `credentials` table:

```sql
CREATE TABLE credentials (
    id TEXT PRIMARY KEY,
    service TEXT NOT NULL CHECK(service IN ('confluence','servicenow','azuredevops')),
    token_hash TEXT NOT NULL,        -- SHA-256 for audit
    encrypted_token TEXT NOT NULL,   -- AES-256-GCM encrypted
    created_at TEXT NOT NULL,
    expires_at TEXT
);
```

**Encryption:**
- Algorithm: AES-256-GCM
- Key derivation: From `TFTSR_DB_KEY` environment variable
- Nonce: Random 96-bit per encryption
- Format: `base64(nonce || ciphertext || tag)`

**Token retrieval:**
```rust
// Backend: src-tauri/src/integrations/auth.rs
pub fn decrypt_token(encrypted: &str) -> Result<String, String>
```
