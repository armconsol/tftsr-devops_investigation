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

---

## Updater Commands {#updater-commands}

### `get_app_version`
```typescript
getAppVersionCmd() → string
```
Returns the running app version, read from `app.package_info().version` (populated from
`tauri.conf.json` at build time). Previously read the `APP_VERSION`/`CARGO_PKG_VERSION`
**environment variables**, which are never set in a packaged build — this made the sidebar
and Updater page report a stale version (e.g. `3.0.0` after installing `3.1.0`). See
[CI/CD Pipeline](CICD-Pipeline.md) for the matching fix to how the version is embedded at
build time.

### `check_app_updates`
```typescript
checkAppUpdatesCmd() → { updateAvailable, currentVersion, latestVersion, releaseUrl, releaseNotes }
```
Fetches releases from the Gitea API and picks the highest-versioned non-draft release,
**including prereleases** — the update-channel concept (`stable` vs `beta`) has been
removed, since installs may themselves come from a beta prerelease and the old channel
filter could hide a genuinely newer prerelease. Version comparison is prerelease-aware
(`3.1.0` > `3.1.0-beta.9` > `3.0.0`).

### `install_app_updates`
```typescript
installAppUpdatesCmd() → void
```
Opens the Gitea releases page in the system browser.

### Removed: `get_update_channel` / `set_update_channel`
These commands (and the `update_channel` field on `AppSettings`) have been removed along
with the Update Channel picker in Settings → Updater — there is no `stable`/`pre-release`
distinction anymore, only "get the latest thing that was actually published." Old
persisted settings JSON containing `update_channel` still deserializes fine; the field is
simply ignored.

---

## Database Tooling Commands (v3.0)

### SSH Tunnel Commands

```typescript
establishDbSshTunnelCmd(
  connection_id: string,
  ssh_hostname: string,
  ssh_port: number,
  ssh_username: string,
  ssh_auth_method: 'password' | 'key',
  ssh_password?: string,
  ssh_private_key?: string,
  ssh_key_passphrase?: string
) → ConnectionTestResult
```

Validates SSH connectivity/authentication and stores encrypted SSH tunnel configuration for a database connection.

```typescript
verifyDbSshTunnelCmd(connection_id: string) → boolean
```

Returns whether SSH tunneling is enabled for the connection.

```typescript
getDbSshConfigCmd(connection_id: string) → DbSshTunnelConfig
```

Returns persisted SSH tunnel metadata (`ssh_enabled`, `ssh_hostname`, `ssh_port`, `ssh_username`, `ssh_auth_method`).

### Table Browser / GUI Data Grid Commands

```typescript
browseTableDataCmd(params: {
  connection_id: string;
  database: string;
  table: string;
  pagination?: { limit: number; offset: number };
  sort?: { column: string; direction: 'ASC' | 'DESC' };
  filters?: Array<{ column: string; operator: '=' | '!=' | '>' | '<' | '>=' | '<=' | 'LIKE'; value: string | number | boolean }>;
}) → BrowseTableResponse
```

Returns paginated rows and page metadata for the selected table.
For `LIKE`, `%` and `_` are SQL wildcards. Escape literal wildcard characters in input where needed.

```typescript
getTableMetadataCmd(connection_id: string, database: string, table: string) → TableMetadata
getTableRowCountCmd(connection_id: string, database: string, table: string) → number
```

Fetches table schema/primary-key metadata and row counts used by the table browser UI.

```typescript
insertTableRowCmd(connection_id: string, database: string, table: string, row_data: RowData) → RowData
updateTableRowCmd(connection_id: string, database: string, table: string, primary_key_col: string, primary_key_value: DataValue, row_data: RowData) → RowData
deleteTableRowCmd(connection_id: string, database: string, table: string, primary_key_col: string, primary_key_value: DataValue) → void
```

CRUD primitives used by the Schema Explorer table browser dialog.
`deleteTableRowCmd` returns `void` on success and fails with an error when the target row does not exist or cannot be deleted.
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
Reads in-memory app settings from backend state.
Settings are not written to a dedicated settings table in this flow; frontend state is persisted in local storage and re-synced through `update_settings`.

`AppSettings` includes:
- `debug_logging_enabled: boolean` (default `false`)

### `update_settings`
```typescript
updateSettingsCmd(partial: Partial<AppSettings>) → AppSettings
```
Merges partial settings in backend state. If `debug_logging_enabled` is provided,
the backend tracing filter is updated live (`info` by default, `debug` when enabled).
This command is the supported toggle path (it applies both state update and live log-level reload).
A legacy `update_channel` key is silently ignored if a pre-upgrade client still sends it —
see [Removed: `get_update_channel` / `set_update_channel`](#updater-commands) below.

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

## Kubernetes Log Commands

Handlers live in `src-tauri/src/commands/kube.rs`. Used by `src/components/dock/LogsTab.tsx`'s "Download Visible" / "Download All" buttons.

### `save_log_file`
```typescript
saveLogFileCmd(path: string, content: string, clusterId: string, namespace: string, podName: string, containerName: string) → void
```
Writes `content` to `path` via `std::fs::write`. `path` must come from a user-driven save dialog (`@tauri-apps/plugin-dialog`'s `save()`) — this command writes outside the `fs` plugin's app/temp-only capability scope (`src-tauri/capabilities/default.json`), mirroring the pattern used by `export_query_results` (`src-tauri/src/commands/database.rs`). Records an `audit_log` entry (`action: "log_file_exported"`) with the destination path, byte count, and cluster/namespace/pod/container context — note pod log content is written raw with no PII redaction applied (redaction only gates text sent to AI providers).

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
- Key derivation: From `TRCAA_DB_KEY` (or legacy `TRCAA_DB_KEY`) environment variable
- Nonce: Random 96-bit per encryption
- Format: `base64(nonce || ciphertext || tag)`

**Token retrieval:**
```rust
// Backend: src-tauri/src/integrations/auth.rs
pub fn decrypt_token(encrypted: &str) -> Result<String, String>
```

---

## Proxmox VM Commands

Handlers: `src-tauri/src/commands/proxmox.rs` — client wrappers: `src/lib/proxmoxClient.ts`

### `list_proxmox_vms`
```typescript
listProxmoxVms(clusterId: string) → any[]
```
Lists all QEMU VMs across a cluster via `cluster/resources?type=vm`.

### `list_proxmox_nodes`
```typescript
listProxmoxNodes(clusterId: string) → Array<{ node: string; status: string; ... }>
```
Lists cluster nodes via `GET /nodes`. Used by the migration dialog to show real target nodes (not just nodes inferred from the VM list).

### `create_proxmox_vm`
```typescript
createProxmoxVm(clusterId, { nodeId, vmid, name, memory, cores, sockets, osType, storage, diskSize, netBridge, iso? }) → void
```
Creates a new QEMU VM on `nodeId` via `POST nodes/{node}/qemu`. Input validation applied server-side:
- `vmid`: 100–999 999 999
- `memory`: 32–1 048 576 MB
- `cores`: 1–512, `sockets`: 1–4, `diskSize`: 1–65 536 GB
- `nodeId`, `storage`, `netBridge`: DNS-label characters only (prevents URL path injection)
- `iso`: must match `storage:iso/filename` format (prevents comma-property injection)

### VM Power Commands
```typescript
startProxmoxVm(clusterId, nodeId, vmId) → void    // POST .../status/start
stopProxmoxVm(clusterId, nodeId, vmId) → void     // POST .../status/stop
rebootProxmoxVm(clusterId, nodeId, vmId) → void   // POST .../status/reboot
shutdownProxmoxVm(clusterId, nodeId, vmId) → void // POST .../status/shutdown
suspendProxmoxVm(clusterId, nodeId, vmId) → void  // POST .../status/suspend
resumeProxmoxVm(clusterId, nodeId, vmId) → void   // POST .../status/resume
```

### `list_proxmox_snapshots`
```typescript
listProxmoxSnapshots(clusterId, nodeId, vmid) → ProxmoxSnapshot[]
```
Lists snapshots for a VM via `GET nodes/{node}/qemu/{vmid}/snapshot`. Returns typed `ProxmoxSnapshot[]` with `snapname`, `vmid`, `ctime`, `parent?`, `description?`.

### `create_proxmox_snapshot`
```typescript
createProxmoxSnapshot(clusterId, nodeId, vmid, snapshotName) → void
```
Creates a VM snapshot via `POST nodes/{node}/qemu/{vmid}/snapshot`.

### `delete_proxmox_snapshot`
```typescript
deleteProxmoxSnapshot(clusterId, nodeId, vmid, snapshotName) → void
```
Deletes a VM snapshot via `DELETE nodes/{node}/qemu/{vmid}/snapshot/{snapname}`.

### `rollback_proxmox_snapshot`
```typescript
rollbackProxmoxSnapshot(clusterId, nodeId, vmid, snapshotName) → void
```
Rolls back a VM to a snapshot via `POST nodes/{node}/qemu/{vmid}/snapshot/{snapname}/rollback`.

### `list_network_interfaces`
```typescript
listNetworkInterfaces(clusterId, nodeId) → NetworkInterface[]
```
Lists network interfaces on a node via `GET nodes/{node}/network`.

### `create_network_interface`
```typescript
createNetworkInterface(clusterId, nodeId, config: NetworkInterfaceConfig) → void
```
Creates a network interface via `POST nodes/{node}/network`.

### `update_network_interface`
```typescript
updateNetworkInterface(clusterId, nodeId, iface, config: NetworkInterfaceConfig) → void
```
Updates a network interface via `PUT nodes/{node}/network/{iface}`.

### `delete_network_interface`
```typescript
deleteNetworkInterface(clusterId, nodeId, iface) → void
```
Deletes a network interface via `DELETE nodes/{node}/network/{iface}`.

### `list_iso_images`
```typescript
listIsoImages(clusterId, nodeId, storageId) → Array<{ volid: string; name?: string; size?: number }>
```
Lists ISO images in a storage pool via `GET nodes/{node}/storage/{storage}/content`, filtering for `content == "iso"`. Used by CreateVmDialog to populate the ISO dropdown.

### `upload_iso_image`
```typescript
uploadIsoImage(clusterId, nodeId, storageId, filePath) → string
```
Uploads a local `.iso` file to a Proxmox storage pool via multipart `POST nodes/{node}/storage/{storage}/upload`. `filePath` is the absolute local path from the OS file picker dialog. Returns the Proxmox task UPID. The `.iso` extension is enforced server-side before the file is read.

### `migrate_vm`
```typescript
invoke('migrate_vm', { clusterId, nodeId, vmId, targetNode, targetCluster }) → void
```
Migrates a VM to another node (same or different cluster). Online migration by default.

### `update_proxmox_firewall_rule`
```typescript
updateFirewallRule(clusterId, nodeId, ruleNum, rule) → void
```
Updates an existing firewall rule via `PUT nodes/{node}/firewall/rules/{pos}`. Uses `proto` and `enable` (integer) field names as required by the PVE API.

### `create_sdn_zone` / `update_sdn_zone` / `delete_sdn_zone`
```typescript
createSdnZone(clusterId, zone, asn, vni) → void
updateSdnZone(clusterId, zone, asn, vni) → void
deleteSdnZone(clusterId, zone) → void
```
CRUD for EVPN SDN zones via `POST/PUT/DELETE cluster/sdn/zones[/{zone}]`.

### `create_sdn_vnet` / `update_sdn_vnet` / `delete_sdn_vnet`
```typescript
createSdnVnet(clusterId, vnet, zone, l2vni) → void
updateSdnVnet(clusterId, vnet, zone, l2vni) → void
deleteSdnVnet(clusterId, vnet) → void
```
CRUD for SDN virtual networks via `POST/PUT/DELETE cluster/sdn/vnets[/{vnet}]`.

### `create_proxmox_backup_job` / `update_proxmox_backup_job` / `delete_proxmox_backup_job`
```typescript
createProxmoxBackupJob({ clusterId, storage, vmid?, mode?, schedule?, enabled? }) → void
updateProxmoxBackupJob(clusterId, jobId, updates) → void
deleteProxmoxBackupJob(clusterId, jobId) → void
```
CRUD for cluster-level backup jobs via `POST/PUT/DELETE cluster/backup[/{id}]`. Uses form-encoded POST (not JSON) to match PVE API requirements.

### `start_proxmox_container` / `stop_proxmox_container` / `reboot_proxmox_container` / `shutdown_proxmox_container` / `suspend_proxmox_container` / `resume_proxmox_container`
```typescript
startProxmoxContainer(clusterId, nodeId, vmId) → void
// (and stop/reboot/shutdown/suspend/resume variants)
```
LXC container power management via `POST nodes/{node}/lxc/{vmid}/status/{action}` with empty form body (same pattern as QEMU).

### `create_proxmox_acl` / `delete_proxmox_acl`
```typescript
createProxmoxAcl(clusterId, path, roles, users?, groups?, propagate?) → void
deleteProxmoxAcl(clusterId, path, roles, users?, groups?) → void
```
ACL management via `PUT /access/acl`. Delete passes `delete: 1` in the JSON body.

### `create_proxmox_user` / `update_proxmox_user` / `delete_proxmox_user`
```typescript
createProxmoxUser(clusterId, userid, password, comment?, email?, enabled?) → void
updateProxmoxUser(clusterId, userid, comment?, email?, enabled?) → void
deleteProxmoxUser(clusterId, userid) → void
```
PVE user management via `POST/PUT/DELETE /access/users[/{userid}]`.

### `create_proxmox_realm` / `update_proxmox_realm` / `delete_proxmox_realm`
```typescript
createProxmoxRealm(clusterId, realm, realmType, comment?) → void
updateProxmoxRealm(clusterId, realm, comment?) → void
deleteProxmoxRealm(clusterId, realm) → void
```
Authentication realm management via `POST/PUT/DELETE /access/domains[/{realm}]`.

### `list_ha_groups` / `create_ha_group` / `update_ha_group` / `delete_ha_group`
```typescript
listHaGroups(clusterId) → Array<HaGroup>
createHaGroup(clusterId, group, nodes) → void
updateHaGroup(clusterId, group, nodes) → void
deleteHaGroup(clusterId, group) → void
```
HA group management via `GET/POST/PUT/DELETE cluster/ha/groups[/{group}]`.

**PVE API field notes:**
- `HaGroup.id` is renamed from the `group` field via `#[serde(rename = "id")]` on the Rust struct — the frontend receives `id`, not `group`.
- `HaGroup.nodes` is a **comma-separated string** (`"vmhost2,vmhost4"`), not an array. `create`/`update` join the input array with `,` before sending to the API.
- Removed non-existent fields `max_failures`, `max_relocate`, and `state` (not part of the PVE HA groups API).

### `list_ha_resources` / `disable_ha_resource` / `delete_ha_resource`
```typescript
listHaResources(clusterId) → Array<HaResource>
disableHaResource(clusterId, id) → void
deleteHaResource(clusterId, id) → void
```
HA resource management via `GET cluster/ha/resources` and `POST cluster/ha/resources/{id}/disable`.

**PVE API field notes:**
- `HaResource.sid` is the correct PVE field name (previously the code read `resource`, which always returned empty).

### `list_proxmox_apt_updates`
```typescript
listProxmoxAptUpdates(clusterId, nodeId) → Array<APTUpdate>
```
Lists pending APT package updates via `GET nodes/{node}/apt/update`.

**PVE API field notes:** PVE returns capitalized field names: `Package`, `Version`, `OldVersion` (installed version), `Origin` (release/repo). The `Size` field is also capitalized.

### `list_proxmox_apt_repositories`
```typescript
listProxmoxAptRepositories(clusterId, nodeId) → Array<APTRepository>
```
Lists configured APT repositories via `GET nodes/{node}/apt/repositories`.

**PVE API response structure:** Returns `{"files": [...], "infos": [...], "standard-repos": [...]}`. Each file object nests its actual repository entries under `files[].repositories[]` (with a flat fallback for older/alternate shapes); each entry has `URIs`, `Suites`, `Components`, `Types`, and `Enabled` as arrays/bool. The endpoint is `apt/repositories`, not `apt/sources`.

`AptRepository` is now emitted with the raw array fields (`types`, `uris`, `suites`,
`components`, `enabled`, `comment`) instead of a flattened first-element view — the
previous flat shape caused the Administration → Repositories tab to crash with
`TypeError: undefined is not an object (evaluating 'e.types.join')` since the frontend
always expected arrays.

### `refresh_apt_cache`
```typescript
refreshAptCache(clusterId: string, node: string) → string // task UPID
```
Refreshes the APT package index on a node (`apt-get update`) via `POST nodes/{node}/apt/update`.
Renamed from the old `update_apt_repos`, which incorrectly POSTed to `apt/sources` (a
repository-management endpoint, not a cache-refresh one). Surfaced in the UI as
Administration → Updates → **Refresh APT Cache**, alongside an **Upgrade Node…** action
that opens a confirmed node shell running `pveupgrade` (see `open_node_shell` below).

---

## Node Administration Commands

### `get_node_dns`
```typescript
getNodeDns(clusterId: string, node: string) → NodeDns
```
Retrieves DNS configuration for a node via `GET nodes/{node}/dns`.

### `update_node_dns`
```typescript
updateNodeDns(clusterId: string, node: string, search: string, dns1?: string, dns2?: string, dns3?: string) → void
```
Sets DNS servers and search domain for a node via `PUT nodes/{node}/dns`.

### `get_node_time`
```typescript
getNodeTime(clusterId: string, node: string) → NodeTime
```
Retrieves current time and timezone for a node via `GET nodes/{node}/time`.

### `update_node_time`
```typescript
updateNodeTime(clusterId: string, node: string, timezone: string) → void
```
Sets timezone for a node via `PUT nodes/{node}/time`.

### `reboot_node`
```typescript
rebootNode(clusterId: string, node: string) → string
```
Schedules a node reboot via `POST nodes/{node}/status/reboot`. Returns UPID (Unique Process ID) for the async task.

### `shutdown_node`
```typescript
shutdownNode(clusterId: string, node: string) → string
```
Schedules a node shutdown via `POST nodes/{node}/status/shutdown`. Returns UPID.

### `get_node_journal`
```typescript
getNodeJournal(clusterId: string, node: string, lastentries?: number) → string[]
```
Retrieves systemd journal entries for a node via `GET nodes/{node}/journal`. Default `lastentries` is 200 if not specified. Tolerates both a plain array of strings and an array of `{n, t}` objects (some PVE versions/proxies wrap journal lines like syslog).

### `get_syslog`
```typescript
getSyslog(clusterId: string, node: string, limit?: number) → SyslogEntry[]
```
Retrieves recent syslog lines for a node via `GET nodes/{node}/syslog` (default `limit` 500).
`SyslogEntry` is `{ n: number, t: string }` — PVE syslog entries carry only a line number and
the full line text, there is no separate message field. Requests to this and other Proxmox
endpoints now retry once on a transient transport error (stale pooled keep-alive connection,
connection reset) and re-authenticate automatically once the cached session ticket is older
than ~90 minutes, fixing intermittent `Failed to get syslog: GET request failed: error sending
request for url (...)` errors that grew more frequent the longer the app stayed open.

### `get_node_report`
```typescript
getNodeReport(clusterId: string, node: string) → string
```
Retrieves a full diagnostic report for a node via `GET nodes/{node}/report`. Returns concatenated system information useful for debugging.

---

## Network Configuration Commands

### `reload_network_config`
```typescript
reloadNetworkConfig(clusterId: string, node: string) → string
```
Applies pending network configuration changes on a node via `POST nodes/{node}/network`. Returns UPID.

---

## VM Configuration Commands

### `get_vm_config`
```typescript
getVmConfig(clusterId: string, node: string, vmId: number) → object
```
Retrieves raw QEMU VM configuration via `GET nodes/{node}/qemu/{vmid}/config`. Returns the complete config object.

### `get_vm_pending_config`
```typescript
getVmPendingConfig(clusterId: string, node: string, vmId: number) → VmPendingEntry[]
```
Retrieves pending (not yet applied) VM configuration changes via `GET nodes/{node}/qemu/{vmid}/pending`. Each entry shows the pending key, current value, and delete flag.

```typescript
interface VmPendingEntry {
    key: string;
    value?: string;
    delete?: number;
}
```

### `remote_migrate_vm`
```typescript
remoteMigrateVm(clusterId: string, node: string, vmId: number, targetNode: string, targetStorage: string, online: boolean) → string
```
Migrates a QEMU VM to a different cluster node via `POST nodes/{node}/qemu/{vmid}/migrate`. `online: true` performs live migration. Returns UPID.

---

## Container (LXC) Configuration Commands

### `get_container_config`
```typescript
getContainerConfig(clusterId: string, node: string, vmId: number) → object
```
Retrieves raw LXC container configuration via `GET nodes/{node}/lxc/{vmid}/config`. Returns the complete config object.

### `create_proxmox_container`
```typescript
createProxmoxContainer(clusterId: string, node: string, vmid: number, ostemplate: string, hostname?: string, memory?: number, cores?: number, rootfs?: string, net0?: string, password?: string, unprivileged?: boolean, start?: boolean) → string
```
Creates a new LXC container via `POST nodes/{node}/lxc`. Input validation applied server-side for `vmid`, `memory`, `cores`, and `hostname`. Returns UPID.

---

## RRD Metrics Commands

### `get_node_rrd_data`
```typescript
getNodeRrdData(clusterId: string, node: string, timeframe: "hour" | "day" | "week" | "month" | "year") → object[]
```
Retrieves RRD (Round-Robin Database) metrics for a node via `GET nodes/{node}/rrddata`. `timeframe` determines the aggregation granularity.

### `get_vm_rrd_data`
```typescript
getVmRrdData(clusterId: string, node: string, vmId: number, timeframe: "hour" | "day" | "week" | "month" | "year") → object[]
```
Retrieves RRD metrics for a QEMU VM via `GET nodes/{node}/qemu/{vmid}/rrddata`.

### `get_storage_rrd_data`
```typescript
getStorageRrdData(clusterId: string, node: string, storage: string, timeframe: "hour" | "day" | "week" | "month" | "year") → object[]
```
Retrieves RRD metrics for a storage pool via `GET nodes/{node}/storage/{storage}/rrddata`.

---

## Ceph Advanced Commands

**Endpoint policy (v1.0.56+):** Ceph endpoints are node-scoped for parity and consistency. Commands use `nodes/{node}/ceph/*`.

### `list_ceph_pools`
```typescript
listCephPools(clusterId: string, node: string) → CephPool[]
```
Lists Ceph pools via `GET nodes/{node}/ceph/pool`.

### `list_ceph_osd`
```typescript
listCephOsd(clusterId: string, node: string) → CephOsd[]
```
Lists OSDs via `GET nodes/{node}/ceph/osd`. PVE returns the **CRUSH tree** as a nested
object (`{ root: { children: [...] } }`); the backend walks the tree and flattens every
`type == "osd"` leaf into `CephOsd { id, host, status, weight, size, used, avail, usedPercent }`.

### `get_ceph_health`
```typescript
getCephHealth(clusterId: string, node: string) → CephHealth
```
Retrieves cluster health via `GET nodes/{node}/ceph/status`.

### `list_ceph_monitors`
```typescript
listCephMonitors(clusterId: string, node: string) → CephMonitor[]
```
Lists Ceph monitor nodes via `GET nodes/{node}/ceph/mon`. PVE exposes `quorum` as a 1/0
number (coerced to bool) and the version under `ceph_version_short`.

### `list_ceph_managers`
```typescript
listCephManagers(clusterId: string, node: string) → CephMgr[]
```
Lists Ceph manager daemons on a node via `GET nodes/{node}/ceph/mgr`. Managers handle cluster balancing and module plugins.

### `list_cephfs`
```typescript
listCephfs(clusterId: string, node: string) → CephFs[]
```
Lists CephFS filesystems on a node via `GET nodes/{node}/ceph/fs`. Returns
`CephFs { name, metadataPool, dataPool }`. `dataPool` accepts either the older single
`data_pool` string PVE field or the current `data_pools` array (joined with `", "` when
there is more than one data pool) — a version mismatch that previously caused CephFS to
render no data at all.

### `get_ceph_flags`
```typescript
getCephFlags(clusterId: string, node: string) → CephFlag[]
```
Retrieves Ceph cluster flags via `GET cluster/ceph/flags`. Flags are a **cluster-level**
resource — the per-node `nodes/{node}/ceph/flags` path returns HTTP 501. Returns an array of
`CephFlag { name, value, description }` (e.g. `noscrub`, `nodeep-scrub`, `noout`).

### `set_ceph_flag`
```typescript
setCephFlag(clusterId: string, flag: string, value: boolean) → void
```
Sets or clears a cluster-level Ceph runtime flag via `PUT cluster/ceph/flags/{flag}`.
`flag` is validated against the same whitelist `get_ceph_flags` surfaces (`noout`, `noin`,
`nodown`, `noup`, `norebalance`, `norecover`, `noscrub`, `nodeep-scrub`, `nobackfill`,
`notieragent`, `pause`).

### `create_ceph_monitor` / `delete_ceph_monitor`
```typescript
createCephMonitor(clusterId: string, node: string, monid: string) → string // task UPID
deleteCephMonitor(clusterId: string, node: string, monid: string) → string  // task UPID
```
Create/destroy a Ceph monitor via `POST`/`DELETE nodes/{node}/ceph/mon/{monid}`.

### `create_ceph_manager` / `delete_ceph_manager`
```typescript
createCephManager(clusterId: string, node: string, id: string) → string // task UPID
deleteCephManager(clusterId: string, node: string, id: string) → string  // task UPID
```
Create/destroy a Ceph manager via `POST`/`DELETE nodes/{node}/ceph/mgr/{id}`.

### `ceph_service_action`
```typescript
cephServiceAction(clusterId: string, node: string, service: string, action: 'start' | 'stop' | 'restart') → string // task UPID
```
Starts, stops, or restarts a Ceph mon/mgr service via `POST nodes/{node}/ceph/{action}`
with form field `service` (e.g. `mon.vmhost1`, `mgr.vmhost1`). `service` is validated
against `^(mon|mgr)\.[A-Za-z0-9-]+$`.

### `create_ceph_pool`
```typescript
createCephPool(clusterId: string, node: string, pool: string, pgNum: number) → void
```
Creates a Ceph pool via `POST nodes/{node}/ceph/pool`. `pool` is validated by
`validate_ceph_pool_name` (alphanumeric, `-`, `_`, `.`, 1-128 chars) before being
sent, guarding against path/command injection since the name is echoed back into
later pool-scoped endpoints.

---

## Certificate & ACME Commands

### `list_certificates` / `get_certificate` / `upload_certificate`
```typescript
listCertificates(clusterId: string, nodeId: string) → object[]
getCertificate(clusterId: string, nodeId: string, certId: string) → object
uploadCertificate(clusterId: string, certificate: string, privateKey: string, name?: string) → object
```
List/get/upload custom TLS certificates via `config/certificate`. `uploadCertificate`'s
parameters were previously mismatched with the backend's `upload_certificate` command
(which takes `certificate`/`privateKey`/`name`, not a free-form `cert` object) — the
wrapper now matches the command signature exactly.

### `list_acme_accounts` / `register_acme_account` / `get_acme_challenges`
```typescript
listAcmeAccounts(clusterId: string) → object[]
registerAcmeAccount(clusterId: string, email: string, termsOfServiceAgreed: boolean) → object
getAcmeChallenges(clusterId: string, domain: string) → object[]
```
Manage ACME accounts via `config/acme/accounts` and `config/acme/challenges/{domain}`.

### `request_acme_certificate`
```typescript
requestAcmeCertificate(clusterId: string, domain: string, accountId: string) → object
```
Orders a new certificate for `domain` via `POST config/acme/certificates`, using the
given ACME account. Used by the Certificates page's "Order via ACME" action and by
per-certificate "Renew" (which re-derives the domain from the certificate's subject/SAN
and reuses the cluster's first registered ACME account, registering one on the fly from
an operator-supplied email if none exists yet).

---

## Firewall Commands

### `list_cluster_firewall_rules`
```typescript
listClusterFirewallRules(clusterId: string) → FirewallRule[]
```
Lists cluster-wide firewall rules via `GET cluster/firewall/rules`. Returns rules applicable to all cluster resources.

### `get_cluster_firewall_status`
```typescript
getClusterFirewallStatus(clusterId: string) → ClusterFirewallStatus
```
Retrieves cluster firewall enable state and default policies via `GET cluster/firewall/options`.

```typescript
interface ClusterFirewallStatus {
    enable?: number;           // 0 or 1
    policy_in?: string;        // Default input policy
    policy_out?: string;       // Default output policy
}
```

### `list_guest_firewall_rules`
```typescript
listGuestFirewallRules(clusterId: string, node: string, vmId: number) → FirewallRule[]
```
Lists firewall rules for a specific VM/container via `GET nodes/{node}/qemu/{vmid}/firewall/rules` or `/lxc/{vmid}/firewall/rules`.

### `add_guest_firewall_rule`
```typescript
addGuestFirewallRule(clusterId: string, node: string, vmId: number, action: string, proto?: string, source?: string, dest?: string, dport?: string, enable?: boolean) → void
```
Adds a firewall rule to a VM/container via `POST nodes/{node}/qemu/{vmid}/firewall/rules` or `/lxc/{vmid}/firewall/rules`. `action` is typically `ACCEPT`, `DROP`, or `REJECT`.

### `delete_guest_firewall_rule`
```typescript
deleteGuestFirewallRule(clusterId: string, node: string, vmId: number, pos: number) → void
```
Deletes a firewall rule by position via `DELETE nodes/{node}/qemu/{vmid}/firewall/rules/{pos}` or `/lxc/{vmid}/firewall/rules/{pos}`.

---

## Task Log Commands

### `get_proxmox_task_log`
```typescript
getProxmoxTaskLog(clusterId: string, node: string, upid: string) → TaskLogEntry[]
```
Retrieves the full log of a single task via `GET nodes/{node}/tasks/{upid}/log`.
`TaskLogEntry` is `{ n: number, t: string }` (line number + full line text — PVE task
logs, like syslog, have no separate message field). `upid` is validated (must start with
`UPID:`, no path-traversal or shell metacharacters) before being interpolated into the
request path.

### `search_task_logs`
```typescript
searchTaskLogs(clusterId: string, query: string, targets: { node: string; upid: string }[]) → TaskLogSearchResult[]
```
Searches the logs of multiple tasks for a case-insensitive substring, powering the search
field on Proxmox → Tasks. Fetches each task's log concurrently (capped at 5 in flight via
`buffer_unordered`), so one slow/failing task doesn't block the rest — a failed fetch
degrades to an empty-match result with an `error` field rather than failing the whole
search. Capped at 100 targets per call; `query` must be at least 2 characters.
`TaskLogSearchResult` is `{ node, upid, matches: TaskLogEntry[], error?: string }`.

---

## Two-Factor Authentication (TFA) Commands

### `list_tfa_entries`
```typescript
listTfaEntries(clusterId: string) → TfaEntry[]
```
Lists all TFA entries visible to the authenticated user via `GET /access/tfa`. Returns TOTP, WebAuthn, recovery codes, and Yubico entries.

```typescript
interface TfaEntry {
    id: string;
    userid: string;
    type: string;              // totp | webauthn | recovery | yubico
    description?: string;
    created: number;           // Unix timestamp
}
```

### `add_tfa_entry`
```typescript
addTfaEntry(clusterId: string, userid: string, tfaType: string, description?: string, totp?: string, value?: string, key?: string) → object
```
Adds a TFA entry for a user via `POST /access/tfa`. `tfaType` must be one of: `totp`, `webauthn`, `recovery`, or `yubico`. Returns the created entry with any generated secrets.

### `delete_tfa_entry`
```typescript
deleteTfaEntry(clusterId: string, userid: string, id: string) → void
```
Deletes a specific TFA entry via `DELETE /access/tfa/{userid}/{id}`.

---

## User API Token Commands

### `list_user_tokens`
```typescript
listUserTokens(clusterId: string, userid: string) → UserToken[]
```
Lists API tokens for a user via `GET /access/users/{userid}/tokens`. Each token shows creation date and expiration (if set).

```typescript
interface UserToken {
    tokenid: string;
    issuedate: number;         // Unix timestamp
    expire?: number;           // Unix timestamp (null if no expiration)
    privsep?: number;          // 0 or 1 (privilege separation)
}
```

### `create_user_token`
```typescript
createUserToken(clusterId: string, userid: string, tokenname: string, comment?: string, privsep?: boolean, expire?: number) → UserTokenCreateResult
```
Creates an API token for a user via `POST /access/users/{userid}/tokens`. **Important:** The token value is only returned once upon creation and cannot be retrieved later.

```typescript
interface UserTokenCreateResult {
    tokenid: string;
    value: string;             // Token secret (only shown once)
    issuedate: number;
    expire?: number;
    privsep?: number;
}
```

### `delete_user_token`
```typescript
deleteUserToken(clusterId: string, userid: string, tokenname: string) → void
```
Revokes an API token via `DELETE /access/users/{userid}/tokens/{tokenname}`.

---

## Proxmox Backup Server (PBS) Management Commands

### `list_pbs_datastores`
```typescript
listPbsDatastores(clusterId: string) → PbsDatastore[]
```
Lists datastores on a PBS cluster via `GET nodes/{node}/storage`. Filters for `type == "pbs"`. Returns datastore name, content, and status.

### `get_pbs_datastore_status`
```typescript
getPbsDatastoreStatus(clusterId: string, store: string) → object
```
Retrieves usage and status for a PBS datastore via `GET nodes/{node}/storage/{store}/status`. Returns capacity, used space, and available space in bytes.

### `list_pbs_namespaces`
```typescript
listPbsNamespaces(clusterId: string, store: string) → PbsNamespace[]
```
Lists namespaces in a PBS datastore via `GET nodes/{node}/pbs/{store}/namespaces`. Returns empty array if the PBS server version does not support namespaces.

```typescript
interface PbsNamespace {
    name: string;
    backup_count?: number;     // Number of backups in this namespace
}
```

### `list_pbs_snapshots`
```typescript
listPbsSnapshots(clusterId: string, store: string, ns?: string) → PbsSnapshot[]
```
Lists backup snapshots in a PBS datastore via `GET nodes/{node}/pbs/{store}/snapshots`. Optional `ns` parameter filters to a specific namespace.

```typescript
interface PbsSnapshot {
    backup_id: string;
    backup_time: number;       // Unix timestamp
    backup_type: string;       // vm | ct
    size: number;              // Bytes
    files?: string[];          // Individual file names
}
```

### `list_pbs_tasks`
```typescript
listPbsTasksCmd(clusterId: string, node: string) → PbsTask[]
```
Lists task history on a PBS node via `GET nodes/{node}/pbs/tasks`. Returns backup job, prune, and verify task records.

### `get_pbs_node_status`
```typescript
getPbsNodeStatus(clusterId: string, node: string) → object
```
Retrieves status of a PBS node via `GET nodes/{node}/status`. Returns uptime, load average, CPU, memory, and disk usage.

---

## Subscription Commands

### `update_subscription`
```typescript
updateSubscription(clusterId: string, node: string, key: string) → void
```
Updates or validates the Proxmox subscription key via `POST nodes/{node}/subscription`. The key is validated and stored on the server. Subscription status can then be queried for entitlement levels and support status.

---

## VM Console & Migration Commands

### `open_vnc_console` / `open_lxc_console`
```typescript
openVncConsole(clusterId: string, node: string, vmId: number) → VncConsoleSession
openLxcConsole(clusterId: string, node: string, vmId: number) → VncConsoleSession
// VncConsoleSession: { local_url: string; ticket: string; local_port: number }
```
Opens an in-app noVNC graphical console. The backend requests a `vncproxy`
ticket (`POST nodes/{node}/qemu|lxc/{vmid}/vncproxy?websocket=1`) and starts a
local WebSocket proxy on `127.0.0.1` that bridges the in-app noVNC client to the
PVE `vncwebsocket` endpoint, injecting the `PVEAuthCookie` and accepting the
node's self-signed TLS certificate. The frontend connects noVNC to `local_url`
using `ticket` as the RFB password (route `/proxmox/console/:clusterId/:node/:vmid/:kind`).

### `open_node_shell`
```typescript
openNodeShell(clusterId: string, node: string, cmd?: 'login' | 'upgrade') → NodeShellSession
// NodeShellSession: { kind: "novnc" | "xterm"; localUrl: string; ticket: string;
//                     localPort: number; password: string | null; user: string }
```
Opens a host (node) shell for a stored remote, reusing the local WebSocket proxy
(`start_vnc_proxy`). The renderer is chosen by the remote's `cluster_type`:
- **PVE** → `POST nodes/{node}/vncshell?websocket=1` (graphical RFB) rendered with
  noVNC. PVE returns a separate `password` used as the RFB password; the `ticket`
  is the `vncticket`. Cookie: `PVEAuthCookie`.
- **PBS** → `POST nodes/{node}/termproxy` (text terminal) rendered with xterm.js
  (PBS has no `vncshell`). The frontend speaks the term-proxy wire protocol
  (login line `"<user>:<ticket>\n"`, framed `0:<len>:<data>` / `1:<cols>:<rows>:`
  / ping `2`). Cookie: `PBSAuthCookie`; PBS requires a `user@pam` ticket.

`cmd` selects the shell command PVE runs, validated against a whitelist (`login` the
default interactive shell, or `upgrade` which runs `pveupgrade` — exactly what the
official PVE web UI's node "Upgrade" button does). Passed through as the `cmd` form
field on `vncshell`/`termproxy`. The route `/proxmox/shell/:clusterId/:node` reads
`?cmd=` from the query string (Administration → Updates → **Upgrade Node…**, behind a
confirmation dialog).

The local proxy injects the correct auth cookie and accepts the node's
self-signed TLS certificate. From **Proxmox | Remotes**, the "Console (Shell)"
action shows a node picker (auto-skipped for single-node remotes) and navigates
to `/proxmox/shell/:clusterId/:node`.

### `start_remote_migration`
```typescript
startRemoteMigration(clusterId, node, vmId, destClusterId, targetNode,
  targetStorage, targetBridge, online) → RemoteMigrationStart
// RemoteMigrationStart: { upid, source_node, dest_cluster_id, dest_userid, dest_tokenname }
```
Performs a true cross-datacenter (remote) migration via
`POST nodes/{node}/qemu/{vmid}/remote-migrate`. The backend auto-creates a
temporary API token on the destination remote, resolves the destination TLS
fingerprint (stored `ssl_fingerprint` override or auto-fetched from
`/nodes/{node}/certificates/info`), builds the `target-endpoint` property
string, and issues the migration. The caller polls `get_task_status` on the
source node and deletes the temporary token (`delete_user_token`) once the task
finishes. `migrate_vm` (intra-cluster) returns the task so the UI can poll and
surface the real Proxmox exit status instead of a false success.
