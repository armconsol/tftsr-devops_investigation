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

**PVE API response structure:** Returns `{"files": [...], "infos": [...], "standard-repos": [...]}`. The implementation reads from the `files` array, where each entry has `URIs`, `Suites`, `Components`, `Types`, and `Enabled` as arrays/bool. The endpoint is `apt/repositories`, not `apt/sources`.

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
Retrieves systemd journal entries for a node via `GET nodes/{node}/journal`. Default `lastentries` is 200 if not specified.

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
Lists OSDs via `GET nodes/{node}/ceph/osd`.

### `get_ceph_health`
```typescript
getCephHealth(clusterId: string, node: string) → CephHealth
```
Retrieves cluster health via `GET nodes/{node}/ceph/status`.

### `list_ceph_monitors`
```typescript
listCephMonitors(clusterId: string, node: string) → CephMonitor[]
```
Lists Ceph monitor nodes via `GET nodes/{node}/ceph/mon`. Returns monitor status and quorum information.

### `list_ceph_managers`
```typescript
listCephManagers(clusterId: string, node: string) → CephMgr[]
```
Lists Ceph manager daemons on a node via `GET nodes/{node}/ceph/mgr`. Managers handle cluster balancing and module plugins.

### `list_cephfs`
```typescript
listCephfs(clusterId: string, node: string) → CephFs[]
```
Lists CephFS filesystems on a node via `GET nodes/{node}/ceph/fs`. Returns mounted filesystem information.

### `get_ceph_flags`
```typescript
getCephFlags(clusterId: string, node: string) → object
```
Retrieves Ceph OSD cluster flags via `GET nodes/{node}/ceph/flags`. Returns an object with boolean flag states (e.g., `noscrub`, `nodeep-scrub`, `pauserd`, `pausewr`).

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
openNodeShell(clusterId: string, node: string) → NodeShellSession
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
