import { invoke } from "@tauri-apps/api/core";

// ─── Types matching Rust backend models ───────────────────────────────────────

export interface ProviderConfig {
  provider_type?: string;
  max_tokens?: number;
  temperature?: number;
  name: string;
  api_url: string;
  api_key: string;
  model: string;
  custom_endpoint_path?: string;
  custom_auth_header?: string;
  custom_auth_prefix?: string;
  api_format?: string;
  session_id?: string;
  user_id?: string;
  use_datastore_upload?: boolean;
}

export interface Message {
  role: string;
  content: string;
}

export interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface ChatResponse {
  content: string;
  model: string;
  usage?: TokenUsage;
  /** What was stored in the DB — may be auto-redacted. Use this for display and history. */
  user_message?: string;
}

export interface AnalysisResult {
  summary: string;
  key_findings: string[];
  suggested_why1: string;
  severity_assessment: string;
}

export interface ProviderInfo {
  name: string;
  supports_streaming: boolean;
  models: string[];
}

export interface Issue {
  id: string;
  title: string;
  description: string;
  severity: string;
  status: string;
  category: string;
  source: string;
  created_at: string;
  updated_at: string;
  resolved_at?: string;
  assigned_to: string;
  tags: string;
}

export interface FiveWhyEntry {
  id: string;
  why_number: number;
  question: string;
  answer?: string;
  created_at: number;
}

export interface TimelineEvent {
  id: string;
  issue_id: string;
  event_type: string;
  description: string;
  metadata: string;
  created_at: string;
}

export interface AiConversation {
  id: string;
  issue_id: string;
  provider: string;
  model: string;
  created_at: string;
  title: string;
}

export interface ResolutionStep {
  id: string;
  issue_id: string;
  step_order: number;
  why_question: string;
  answer: string;
  evidence: string;
  created_at: string;
}

export interface IssueDetail {
  issue: Issue;
  log_files: LogFile[];
  image_attachments: ImageAttachment[];
  resolution_steps: ResolutionStep[];
  conversations: AiConversation[];
  timeline_events: TimelineEvent[];
}

export interface IssueSummary {
  id: string;
  title: string;
  severity: string;
  status: string;
  category: string;
  created_at: string;
  updated_at: string;
  domain?: string;
  log_count: number;
  step_count: number;
}

export interface IssueListQuery {
  status?: string;
  domain?: string;
  severity?: string;
  search?: string;
  limit?: number;
  offset?: number;
}

export interface NewIssue {
  title: string;
  domain: string;
  description?: string;
  severity?: string;
}

export interface LogFile {
  id: string;
  issue_id: string;
  file_name: string;
  file_path: string;
  file_size: number;
  mime_type: string;
  content_hash: string;
  uploaded_at: string;
  redacted: boolean;
}

export interface ImageAttachment {
  id: string;
  issue_id: string;
  file_name: string;
  file_path: string;
  file_size: number;
  mime_type: string;
  upload_hash: string;
  uploaded_at: string;
  pii_warning_acknowledged: boolean;
  is_paste: boolean;
}

export interface PiiSpan {
  id: string;
  pii_type: string;
  start: number;
  end: number;
  original: string;
  replacement: string;
}

export interface PiiDetectionResult {
  log_file_id: string;
  detections: PiiSpan[];
  total_pii_found: number;
}

export interface RedactedLogFile {
  id: string;
  original_file_id: string;
  file_name: string;
  file_hash: string;
  redaction_count: number;
}

export interface Document_ {
  id: string;
  issue_id: string;
  doc_type: string;
  title: string;
  content_md: string;
  created_at: number;
  updated_at: number;
}

export interface HardwareInfo {
  total_ram_gb: number;
  cpu_arch: string;
  gpu_vendor?: string;
  gpu_vram_gb?: number;
}

export interface ModelRecommendation {
  name: string;
  size: string;
  min_ram_gb: number;
  description: string;
  recommended: boolean;
}

export interface OllamaModel {
  name: string;
  size: number;
  modified: string;
}

export interface OllamaStatus {
  installed: boolean;
  version?: string;
  running: boolean;
}

export interface InstallGuide {
  platform: string;
  steps: string[];
  url: string;
}

export interface AuditEntry {
  id: string;
  timestamp: string;
  action: string;
  entity_type: string;
  entity_id: string;
  user_id: string;
  details: string;
}

export interface AuditFilter {
  action?: string;
  entity_type?: string;
  entity_id?: string;
  limit?: number;
}

export interface AppSettings {
  theme: string;
  ai_providers: ProviderConfig[];
  active_provider?: string;
  default_provider: string;
  default_model: string;
  ollama_url: string;
}

// ─── TriageMessage (for UI store, not a DB type) ──────────────────────────────

export interface TriageMessage {
  id: string;
  issue_id: string;
  role: string;
  content: string;
  why_level?: number;
  created_at: number;
}

// ─── AI commands ──────────────────────────────────────────────────────────────

export const analyzeLogsCmd = (issueId: string, logFileIds: string[], providerConfig: ProviderConfig) =>
  invoke<AnalysisResult>("analyze_logs", { issueId, logFileIds, providerConfig });

export const chatMessageCmd = (
  issueId: string,
  message: string,
  logFileIds: string[],
  providerConfig: ProviderConfig,
  systemPrompt?: string
) =>
  invoke<ChatResponse>("chat_message", {
    issueId,
    message,
    logFileIds: logFileIds.length > 0 ? logFileIds : undefined,
    providerConfig,
    systemPrompt: systemPrompt ?? null,
  });

export const listProvidersCmd = () => invoke<ProviderInfo[]>("list_providers");

// ─── Analysis / PII commands ──────────────────────────────────────────────────

export const uploadLogFileCmd = (issueId: string, filePath: string) =>
  invoke<LogFile>("upload_log_file", { issueId, filePath });

export const uploadLogFileByContentCmd = (issueId: string, fileName: string, content: string) =>
  invoke<LogFile>("upload_log_file_by_content", { issueId, fileName, content });

export const uploadImageAttachmentCmd = (issueId: string, filePath: string) =>
  invoke<ImageAttachment>("upload_image_attachment", { issueId, filePath });

export const uploadImageAttachmentByContentCmd = (issueId: string, fileName: string, base64Content: string) =>
  invoke<ImageAttachment>("upload_image_attachment_by_content", { issueId, fileName, base64Content });

export const uploadFileToDatastoreCmd = (providerConfig: ProviderConfig, filePath: string) =>
  invoke<string>("upload_file_to_datastore", { providerConfig, filePath });

export const uploadFileToDatastoreAnyCmd = (providerConfig: ProviderConfig, filePath: string) =>
  invoke<string>("upload_file_to_datastore_any", { providerConfig, filePath });

export const uploadPasteImageCmd = (issueId: string, base64Image: string, mimeType: string) =>
  invoke<ImageAttachment>("upload_paste_image", { issueId, base64Image, mimeType });

export const listImageAttachmentsCmd = (issueId: string) =>
  invoke<ImageAttachment[]>("list_image_attachments", { issueId });

export const deleteImageAttachmentCmd = (attachmentId: string) =>
  invoke<void>("delete_image_attachment", { attachmentId });

export const detectPiiCmd = (logFileId: string) =>
  invoke<PiiDetectionResult>("detect_pii", { logFileId });

export const scanTextForPiiCmd = (text: string) =>
  invoke<PiiDetectionResult>("scan_text_for_pii", { text });

export const applyRedactionsCmd = (logFileId: string, approvedSpanIds: string[]) =>
  invoke<RedactedLogFile>("apply_redactions", { logFileId, approvedSpanIds });

// ─── Issue CRUD ───────────────────────────────────────────────────────────────

export const testProviderConnectionCmd = (providerConfig: ProviderConfig) =>
  invoke<ChatResponse>("test_provider_connection", { providerConfig });

export const createIssueCmd = (newIssue: NewIssue) =>
  invoke<Issue>("create_issue", {
    title: newIssue.title,
    description: newIssue.description ?? "",
    severity: newIssue.severity ?? "P3",
    category: newIssue.domain,
  });

export const getIssueCmd = (issueId: string) =>
  invoke<IssueDetail>("get_issue", { issueId });

export const listIssuesCmd = (query: IssueListQuery) =>
  invoke<IssueSummary[]>("list_issues", { filter: query });

export const updateIssueCmd = (
  issueId: string,
  updates: { title?: string; status?: string; severity?: string; description?: string; domain?: string }
) => invoke<Issue>("update_issue", { issueId, updates });

export const deleteIssueCmd = (issueId: string) =>
  invoke<void>("delete_issue", { issueId });

export const searchIssuesCmd = (query: string) =>
  invoke<IssueSummary[]>("search_issues", { query });

export interface IssueMessage {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  token_count: number;
  created_at: string;
}

export const getIssueMessagesCmd = (issueId: string) =>
  invoke<IssueMessage[]>("get_issue_messages", { issueId });

export const addFiveWhyCmd = (
  issueId: string,
  stepOrder: number,
  whyQuestion: string,
  answer: string,
  evidence: string
) => invoke<ResolutionStep>("add_five_why", { issueId, stepOrder, whyQuestion, answer, evidence });

export const updateFiveWhyCmd = (entryId: string, answer: string) =>
  invoke<void>("update_five_why", { entryId, answer });

export const addTimelineEventCmd = (issueId: string, eventType: string, description: string, metadata?: string) =>
  invoke<TimelineEvent>("add_timeline_event", { issueId, eventType, description, metadata: metadata ?? null });

export const getTimelineEventsCmd = (issueId: string) =>
  invoke<TimelineEvent[]>("get_timeline_events", { issueId });

// ─── Document commands ────────────────────────────────────────────────────────

export const generateRcaCmd = (issueId: string) => invoke<Document_>("generate_rca", { issueId });

export const generatePostmortemCmd = (issueId: string) =>
  invoke<Document_>("generate_postmortem", { issueId });

export const updateDocumentCmd = (docId: string, contentMd: string) =>
  invoke<Document_>("update_document", { docId, contentMd });

export const exportDocumentCmd = (docId: string, title: string, contentMd: string, format: string, outputDir: string) =>
  invoke<string>("export_document", { title, contentMd, format, outputDir });

// ─── Ollama & System ──────────────────────────────────────────────────────────

export const checkOllamaInstalledCmd = () => invoke<OllamaStatus>("check_ollama_installed");

export const getOllamaInstallGuideCmd = (platform: string) =>
  invoke<InstallGuide>("get_ollama_install_guide", { platform });

export const listOllamaModelsCmd = () => invoke<OllamaModel[]>("list_ollama_models");

export const pullOllamaModelCmd = (modelName: string) =>
  invoke<void>("pull_ollama_model", { modelName });

export const deleteOllamaModelCmd = (modelName: string) =>
  invoke<void>("delete_ollama_model", { modelName });

export const detectHardwareCmd = () => invoke<HardwareInfo>("detect_hardware");

export const recommendModelsCmd = () => invoke<ModelRecommendation[]>("recommend_models");

// ─── Settings & Audit ─────────────────────────────────────────────────────────

export const getSettingsCmd = () => invoke<AppSettings>("get_settings");

export const updateSettingsCmd = (partialSettings: Partial<AppSettings>) =>
  invoke<AppSettings>("update_settings", { partialSettings });

export const getAuditLogCmd = (filter: AuditFilter) =>
  invoke<AuditEntry[]>("get_audit_log", { filter });

// ─── OAuth & Integrations ─────────────────────────────────────────────────────

export interface OAuthInitResponse {
  auth_url: string;
  state: string;
}

export interface ConnectionResult {
  success: boolean;
  message: string;
}

export const initiateOauthCmd = (service: string) =>
  invoke<OAuthInitResponse>("initiate_oauth", { service });

export const handleOauthCallbackCmd = (service: string, code: string, stateKey: string) =>
  invoke<void>("handle_oauth_callback", { service, code, stateKey });

export const testConfluenceConnectionCmd = (baseUrl: string, credentials: Record<string, unknown>) =>
  invoke<ConnectionResult>("test_confluence_connection", { baseUrl, credentials });

export const testServiceNowConnectionCmd = (instanceUrl: string, credentials: Record<string, unknown>) =>
  invoke<ConnectionResult>("test_servicenow_connection", { instanceUrl, credentials });

export const testAzureDevOpsConnectionCmd = (orgUrl: string, credentials: Record<string, unknown>) =>
  invoke<ConnectionResult>("test_azuredevops_connection", { orgUrl, credentials });

// ─── Webview & Token Authentication ──────────────────────────────────────────

export interface WebviewAuthResponse {
  success: boolean;
  message: string;
  webview_id: string;
}

export interface TokenAuthRequest {
  service: string;
  token: string;
  token_type: string;
  base_url: string;
}

export interface IntegrationConfig {
  service: string;
  base_url: string;
  username?: string;
  project_name?: string;
  space_key?: string;
}

export const authenticateWithWebviewCmd = (service: string, baseUrl: string, projectName?: string) =>
  invoke<WebviewAuthResponse>("authenticate_with_webview", { service, baseUrl, projectName });

export const extractCookiesFromWebviewCmd = (service: string, webviewId: string) =>
  invoke<ConnectionResult>("extract_cookies_from_webview", { service, webviewId });

export const saveManualTokenCmd = (request: TokenAuthRequest) =>
  invoke<ConnectionResult>("save_manual_token", { request });

// ─── Integration Configuration Persistence ────────────────────────────────────

export const saveIntegrationConfigCmd = (config: IntegrationConfig) =>
  invoke<void>("save_integration_config", { config });

export const getIntegrationConfigCmd = (service: string) =>
  invoke<IntegrationConfig | null>("get_integration_config", { service });

export const getAllIntegrationConfigsCmd = () =>
  invoke<IntegrationConfig[]>("get_all_integration_configs");

// ─── AI Provider Configuration ────────────────────────────────────────────────

export const saveAiProviderCmd = (config: ProviderConfig) =>
  invoke<void>("save_ai_provider", { provider: config });

export const loadAiProvidersCmd = () =>
  invoke<ProviderConfig[]>("load_ai_providers");

export const deleteAiProviderCmd = (name: string) =>
  invoke<void>("delete_ai_provider", { name });

// ─── MCP Server types ────────────────────────────────────────────────────────

export interface McpServer {
  id: string;
  name: string;
  url: string;
  transport_type: "stdio" | "http";
  transport_config: string;
  auth_type: "none" | "api_key" | "bearer" | "oauth2";
  auth_value?: string;
  enabled: boolean;
  last_discovered_at?: string;
  discovery_status: "pending" | "connected" | "unreachable" | "error";
  discovery_error?: string;
  created_at: string;
  updated_at: string;
}

export interface McpTool {
  id: string;
  server_id: string;
  name: string;
  tool_key: string;
  description?: string;
  parameters: string;
}

export interface McpResource {
  id: string;
  server_id: string;
  uri: string;
  name?: string;
  description?: string;
}

export interface McpServerStatus {
  server_id: string;
  status: "pending" | "connected" | "unreachable" | "error";
  error?: string;
  tool_count: number;
  resource_count: number;
  last_discovered_at?: string;
}

export interface CreateMcpServerRequest {
  name: string;
  url: string;
  transport_type: "stdio" | "http";
  transport_config: string;
  auth_type: "none" | "api_key" | "bearer" | "oauth2";
  auth_value?: string;
  enabled: boolean;
}

export interface UpdateMcpServerRequest {
  name?: string;
  url?: string;
  transport_type?: "stdio" | "http";
  transport_config?: string;
  auth_type?: "none" | "api_key" | "bearer" | "oauth2";
  auth_value?: string;
  enabled?: boolean;
}

// ─── MCP Commands ─────────────────────────────────────────────────────────────

export function listMcpServersCmd(): Promise<McpServer[]> {
  return invoke<McpServer[]>("list_mcp_servers");
}

export function createMcpServerCmd(request: CreateMcpServerRequest): Promise<McpServer> {
  return invoke<McpServer>("create_mcp_server", { request });
}

export function updateMcpServerCmd(id: string, request: UpdateMcpServerRequest): Promise<McpServer> {
  return invoke<McpServer>("update_mcp_server", { id, request });
}

export function deleteMcpServerCmd(id: string): Promise<void> {
  return invoke<void>("delete_mcp_server", { id });
}

export function toggleMcpServerCmd(id: string, enabled: boolean): Promise<void> {
  return invoke<void>("toggle_mcp_server", { id, enabled });
}

export function discoverMcpServerCmd(id: string): Promise<McpServerStatus> {
  return invoke<McpServerStatus>("discover_mcp_server", { id });
}

export function getMcpServerStatusCmd(id: string): Promise<McpServerStatus> {
  return invoke<McpServerStatus>("get_mcp_server_status", { id });
}

export function initiateMcpOauthCmd(id: string): Promise<void> {
  return invoke<void>("initiate_mcp_oauth", { id });
}

// ─── Sudo credential commands ─────────────────────────────────────────────────

export interface SudoConfigStatus {
  configured: boolean;
  username: string;
  updated_at: string;
}

export const setSudoPasswordCmd = (password: string, username?: string) =>
  invoke<void>("set_sudo_password", { password, username: username ?? null });

export const getSudoConfigStatusCmd = () =>
  invoke<SudoConfigStatus>("get_sudo_config_status");

export const testSudoPasswordCmd = () =>
  invoke<boolean>("test_sudo_password");

export const clearSudoPasswordCmd = () =>
  invoke<void>("clear_sudo_password");

// ─── System / Version ─────────────────────────────────────────────────────────

export const getAppVersionCmd = () =>
  invoke<string>("get_app_version");

// ─── Attachment cross-incident types ─────────────────────────────────────────

export interface LogFileSummary {
  id: string;
  issue_id: string;
  issue_title: string;
  file_name: string;
  file_path: string;
  file_size: number;
  mime_type: string;
  content_hash: string;
  uploaded_at: string;
  redacted: boolean;
}

export interface ImageAttachmentSummary {
  id: string;
  issue_id: string;
  issue_title: string;
  file_name: string;
  file_path: string;
  file_size: number;
  mime_type: string;
  upload_hash: string;
  uploaded_at: string;
  pii_warning_acknowledged: boolean;
  is_paste: boolean;
}

// ─── Attachment cross-incident commands ───────────────────────────────────────

export const getLogFileContentCmd = (logFileId: string) =>
  invoke<string>("get_log_file_content", { logFileId });

export const listAllLogFilesCmd = (search?: string, issueId?: string) =>
  invoke<LogFileSummary[]>("list_all_log_files", {
    search: search ?? null,
    issueId: issueId ?? null,
  });

export const getImageAttachmentDataCmd = (attachmentId: string) =>
  invoke<string>("get_image_attachment_data", { attachmentId });

export const listAllImageAttachmentsCmd = (search?: string, issueId?: string) =>
  invoke<ImageAttachmentSummary[]>("list_all_image_attachments", {
    search: search ?? null,
    issueId: issueId ?? null,
  });
