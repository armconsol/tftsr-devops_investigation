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
  supports_tool_calling?: boolean;
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

export const detectToolCallingSupportCmd = (providerConfig: ProviderConfig) =>
  invoke<boolean>("detect_tool_calling_support", { providerConfig });

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
  env_config?: string;
}

export interface UpdateMcpServerRequest {
  name?: string;
  url?: string;
  transport_type?: "stdio" | "http";
  transport_config?: string;
  auth_type?: "none" | "api_key" | "bearer" | "oauth2";
  auth_value?: string;
  enabled?: boolean;
  env_config?: string;
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

// ─── Updater ──────────────────────────────────────────────────────────────────

export interface UpdateCheckResult {
  updateAvailable: boolean;
  currentVersion: string;
  latestVersion: string;
  releaseUrl: string;
  releaseNotes: string;
}

export const checkAppUpdatesCmd = async (): Promise<UpdateCheckResult> =>
  invoke<UpdateCheckResult>("check_app_updates");

export const installAppUpdatesCmd = async (): Promise<void> =>
  invoke<void>("install_app_updates");

export const getUpdateChannelCmd = async (): Promise<string> =>
  invoke<string>("get_update_channel");

export const setUpdateChannelCmd = async (channel: string): Promise<void> =>
  invoke<void>("set_update_channel", { channel });

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

// ─── Shell Execution Commands ────────────────────────────────────────────────

export interface KubeconfigInfo {
  id: string;
  name: string;
  context: string;
  cluster_url?: string;
  is_active: boolean;
}

export interface CommandExecution {
  id: string;
  command: string;
  tier: number;
  approval_status: string;
  exit_code: number | null;
  stdout: string | null;
  stderr: string | null;
  execution_time_ms: number | null;
  executed_at: string;
}

export interface KubectlStatus {
  installed: boolean;
  path?: string;
  version?: string;
}

export const uploadKubeconfigCmd = (name: string, content: string) =>
  invoke<string>("upload_kubeconfig", { name, content });

export const listKubeconfigsCmd = () =>
  invoke<KubeconfigInfo[]>("list_kubeconfigs");

export const activateKubeconfigCmd = (id: string) =>
  invoke<void>("activate_kubeconfig", { id });

export const deleteKubeconfigCmd = (id: string) =>
  invoke<void>("delete_kubeconfig", { id });

export const respondToShellApprovalCmd = (approvalId: string, decision: string) =>
  invoke<void>("respond_to_shell_approval", { approvalId, decision });

export const listCommandExecutionsCmd = (issueId?: string) =>
  invoke<CommandExecution[]>("list_command_executions", {
    issueId: issueId ?? null,
  });

export const checkKubectlInstalledCmd = () =>
  invoke<KubectlStatus>("check_kubectl_installed");

export interface ClassifierRules {
  tier1_kubectl: string[];
  tier1_systemctl: string[];
  tier1_proxmox: string[];
  tier1_general: string[];
  tier2_kubectl: string[];
  tier2_systemctl: string[];
  tier2_proxmox: string[];
  tier2_general: string[];
  tier3: string[];
}

export const getClassifierRulesCmd = () =>
  invoke<ClassifierRules>("get_classifier_rules");

// ─── Kubernetes Management Types ──────────────────────────────────────────────

export interface ClusterInfo {
  id: string;
  name: string;
  context: string;
  cluster_url: string;
}

export interface ContextInfo {
  name: string;
  cluster: string;
  user: string;
}

export interface ResourceInfo {
  name: string;
  namespace: string;
  [key: string]: unknown;
}

export interface PortForwardRequest {
  cluster_id: string;
  namespace: string;
  pod: string;
  container_port: number;
  local_port?: number;
}

export interface PortForwardResponse {
  id: string;
  cluster_id: string;
  namespace: string;
  pod: string;
  container_ports: number[];
  local_ports: number[];
  status: string;
}

export interface PodInfo {
  name: string;
  namespace: string;
  status: string;
  ready: string;
  age: string;
  containers: string[];
  restarts?: number;
  ip?: string;
  node?: string;
}

export interface ClusterConnectionState {
  type: "Connected" | "Disconnected";
  error?: string;
}

export interface ClusterConnectionStatus {
  status: ClusterConnectionState;
  context: string;
}

// ─── Kubernetes Resource Discovery Types ──────────────────────────────────────

export interface NamespaceInfo {
  name: string;
  status: string;
  age: string;
}

export interface ServicePort {
  name?: string;
  port: number;
  target_port?: string;
  protocol: string;
}

export interface ServiceInfo {
  name: string;
  namespace: string;
  type: string;
  cluster_ip: string;
  external_ip?: string;
  ports: ServicePort[];
  age: string;
  selector: Record<string, string>;
}

export interface DeploymentInfo {
  name: string;
  namespace: string;
  ready: string;
  up_to_date: string;
  available: string;
  age: string;
  replicas: number;
  labels: Record<string, string>;
}

export interface StatefulSetInfo {
  name: string;
  namespace: string;
  ready: string;
  age: string;
  replicas: number;
  labels: Record<string, string>;
}

export interface DaemonSetInfo {
  name: string;
  namespace: string;
  desired: number;
  current: number;
  ready: number;
  up_to_date: number;
  available: number;
  age: string;
  labels: Record<string, string>;
}

export interface NodeMetrics {
  name: string;
  cpu_usage: string;
  memory_usage: string;
  cpu_percentage: number;
  memory_percentage: number;
  age: string;
}

export interface PodMetrics {
  name: string;
  namespace: string;
  cpu_usage: string;
  memory_usage: string;
  cpu_percentage: number;
  memory_percentage: number;
}

export interface LogResponse {
  logs: string;
}

export interface ExecResponse {
  stdout: string;
  stderr: string;
  exit_code: number | null;
}

export interface ExecSessionResponse {
  session_id: string;
  cluster_id: string;
  namespace: string;
  pod: string;
  container?: string;
  status: string;
}

// ─── Kubernetes Management Commands ───────────────────────────────────────────

export const addClusterCmd = (id: string, name: string, kubeconfigContent: string) =>
  invoke<ClusterInfo>("add_cluster", { id, name, kubeconfig_content: kubeconfigContent });

export const removeClusterCmd = (id: string) =>
  invoke<void>("remove_cluster", { id });

export const connectClusterFromKubeconfigCmd = (id: string) =>
  invoke<void>("connect_cluster_from_kubeconfig", { id });

/** Diagnostic: runs kubectl cluster-info and returns a human-readable summary. */
export const testKubectlConnectionCmd = (clusterId: string) =>
  invoke<string>("test_kubectl_connection", { clusterId });

export const listClustersCmd = () =>
  invoke<ClusterInfo[]>("list_clusters");

export const startPortForwardCmd = (request: PortForwardRequest) =>
  invoke<PortForwardResponse>("start_port_forward", { request });

export const stopPortForwardCmd = (id: string) =>
  invoke<void>("stop_port_forward", { id });

export const deletePortForwardCmd = (id: string) =>
  invoke<void>("delete_port_forward", { id });

export const listPortForwardsCmd = () =>
  invoke<PortForwardResponse[]>("list_port_forwards");

export const shutdownPortForwardsCmd = () =>
  invoke<void>("shutdown_port_forwards");

export const testClusterConnectionCmd = (clusterId: string) =>
  invoke<ClusterConnectionStatus>("test_cluster_connection", { clusterId });

export const discoverPodsCmd = (clusterId: string, namespace: string) =>
  invoke<PodInfo[]>("discover_pods", { clusterId, namespace });

// ─── Kubernetes Resource Discovery Commands ───────────────────────────────────

export const listNamespacesCmd = (clusterId: string) =>
  invoke<NamespaceInfo[]>("list_namespaces", { clusterId });

export const listPodsCmd = (clusterId: string, namespace: string) =>
  invoke<PodInfo[]>("list_pods", { clusterId, namespace });

export const listServicesCmd = (clusterId: string, namespace: string) =>
  invoke<ServiceInfo[]>("list_services", { clusterId, namespace });

export const listDeploymentsCmd = (clusterId: string, namespace: string) =>
  invoke<DeploymentInfo[]>("list_deployments", { clusterId, namespace });

export const listStatefulsetsCmd = (clusterId: string, namespace: string) =>
  invoke<StatefulSetInfo[]>("list_statefulsets", { clusterId, namespace });

export const listDaemonsetsCmd = (clusterId: string, namespace: string) =>
  invoke<DaemonSetInfo[]>("list_daemonsets", { clusterId, namespace });

// ─── Kubernetes Resource Management Commands ──────────────────────────────────

export const getPodLogsCmd = (clusterId: string, namespace: string, podName: string, containerName: string) =>
  invoke<LogResponse>("get_pod_logs", { clusterId, namespace, podName, containerName });

export const scaleDeploymentCmd = (clusterId: string, namespace: string, deploymentName: string, replicas: number) =>
  invoke<void>("scale_deployment", { clusterId, namespace, deploymentName, replicas });

export const restartDeploymentCmd = (clusterId: string, namespace: string, deploymentName: string) =>
  invoke<void>("restart_deployment", { clusterId, namespace, deploymentName });

export const deleteResourceCmd = (clusterId: string, resourceType: string, namespace: string, resourceName: string) =>
  invoke<void>("delete_resource", { clusterId, resourceType, namespace, resourceName });

export const execPodCmd = (clusterId: string, namespace: string, podName: string, containerName: string, command: string, shell?: string) =>
  invoke<ExecResponse>("exec_pod", { clusterId, namespace, podName, containerName, shell, command });

// ─── Additional Kubernetes Resource Discovery Types ───────────────────────────

export interface ReplicaSetInfo {
  name: string;
  namespace: string;
  replicas: number;
  ready: string;
  age: string;
  labels: Record<string, string>;
}

export interface JobInfo {
  name: string;
  namespace: string;
  completions: string;
  duration: string;
  age: string;
  labels: Record<string, string>;
}

export interface CronJobInfo {
  name: string;
  namespace: string;
  schedule: string;
  active: number;
  last_schedule: string;
  age: string;
  labels: Record<string, string>;
}

export interface ConfigMapInfo {
  name: string;
  namespace: string;
  data_keys: number;
  age: string;
}

export interface SecretInfo {
  name: string;
  namespace: string;
  type: string;
  data_keys: number;
  age: string;
}

export interface NodeInfo {
  name: string;
  status: string;
  roles: string;
  version: string;
  internal_ip: string;
  external_ip?: string;
  os_image: string;
  kernel_version: string;
  kubelet_version: string;
  age: string;
}

export interface EventInfo {
  name: string;
  namespace: string;
  event_type: string;
  reason: string;
  object: string;
  count: number;
  first_seen: string;
  last_seen: string;
  message: string;
}

export interface IngressInfo {
  name: string;
  namespace: string;
  class?: string;
  host: string;
  addresses: string[];
  age: string;
}

export interface PersistentVolumeClaimInfo {
  name: string;
  namespace: string;
  status: string;
  volume: string;
  capacity: string;
  access_modes: string[];
  age: string;
}

export interface PersistentVolumeInfo {
  name: string;
  status: string;
  capacity: string;
  access_modes: string[];
  reclaim_policy: string;
  storage_class: string;
  age: string;
}

export interface ServiceAccountInfo {
  name: string;
  namespace: string;
  secrets: number;
  age: string;
}

export interface RoleInfo {
  name: string;
  namespace: string;
  age: string;
}

export interface ClusterRoleInfo {
  name: string;
  age: string;
}

export interface RoleBindingInfo {
  name: string;
  namespace: string;
  role: string;
  age: string;
}

export interface ClusterRoleBindingInfo {
  name: string;
  cluster_role: string;
  age: string;
}

export interface HorizontalPodAutoscalerInfo {
  name: string;
  namespace: string;
  min_replicas: number;
  max_replicas: number;
  current_replicas: number;
  desired_replicas: number;
  age: string;
}

// ─── Additional Kubernetes Resource Discovery Commands ────────────────────────

export const listReplicasetsCmd = (clusterId: string, namespace: string) =>
  invoke<ReplicaSetInfo[]>("list_replicasets", { clusterId, namespace });

export const listJobsCmd = (clusterId: string, namespace: string) =>
  invoke<JobInfo[]>("list_jobs", { clusterId, namespace });

export const listCronjobsCmd = (clusterId: string, namespace: string) =>
  invoke<CronJobInfo[]>("list_cronjobs", { clusterId, namespace });

export const listConfigmapsCmd = (clusterId: string, namespace: string) =>
  invoke<ConfigMapInfo[]>("list_configmaps", { clusterId, namespace });

export const listSecretsCmd = (clusterId: string, namespace: string) =>
  invoke<SecretInfo[]>("list_secrets", { clusterId, namespace });

export const listNodesCmd = (clusterId: string) =>
  invoke<NodeInfo[]>("list_nodes", { clusterId });

export const listEventsCmd = (clusterId: string, namespace?: string) =>
  invoke<EventInfo[]>("list_events", { clusterId, namespace });

export const listIngressesCmd = (clusterId: string, namespace: string) =>
  invoke<IngressInfo[]>("list_ingresses", { clusterId, namespace });

export const listPersistentvolumeclaimsCmd = (clusterId: string, namespace: string) =>
  invoke<PersistentVolumeClaimInfo[]>("list_persistentvolumeclaims", { clusterId, namespace });

export const listPersistentvolumesCmd = (clusterId: string) =>
  invoke<PersistentVolumeInfo[]>("list_persistentvolumes", { clusterId });

export const listServiceaccountsCmd = (clusterId: string, namespace: string) =>
  invoke<ServiceAccountInfo[]>("list_serviceaccounts", { clusterId, namespace });

export const listRolesCmd = (clusterId: string, namespace: string) =>
  invoke<RoleInfo[]>("list_roles", { clusterId, namespace });

export const listClusterrolesCmd = (clusterId: string) =>
  invoke<ClusterRoleInfo[]>("list_clusterroles", { clusterId });

export const listRolebindingsCmd = (clusterId: string, namespace: string) =>
  invoke<RoleBindingInfo[]>("list_rolebindings", { clusterId, namespace });

export const listClusterrolebindingsCmd = (clusterId: string) =>
  invoke<ClusterRoleBindingInfo[]>("list_clusterrolebindings", { clusterId });

export const listHorizontalpodautoscalersCmd = (clusterId: string, namespace: string) =>
  invoke<HorizontalPodAutoscalerInfo[]>("list_horizontalpodautoscalers", { clusterId, namespace });

// ─── Additional Lens Resource Types ───────────────────────────────────────────

export interface StorageClassInfo {
  name: string;
  provisioner: string;
  reclaim_policy: string;
  volume_binding_mode: string;
  allow_volume_expansion: boolean;
  age: string;
}

export interface NetworkPolicyInfo {
  name: string;
  namespace: string;
  pod_selector: string;
  policy_types: string[];
  age: string;
}

export interface ResourceQuotaInfo {
  name: string;
  namespace: string;
  request_cpu: string;
  request_memory: string;
  limit_cpu: string;
  limit_memory: string;
  age: string;
}

export interface LimitRangeInfo {
  name: string;
  namespace: string;
  limit_count: number;
  age: string;
}

export const listStorageclassesCmd = (clusterId: string) =>
  invoke<StorageClassInfo[]>("list_storageclasses", { clusterId });

export const listNetworkpoliciesCmd = (clusterId: string, namespace: string) =>
  invoke<NetworkPolicyInfo[]>("list_networkpolicies", { clusterId, namespace });

export const listResourcequotasCmd = (clusterId: string, namespace: string) =>
  invoke<ResourceQuotaInfo[]>("list_resourcequotas", { clusterId, namespace });

export const listLimitrangesCmd = (clusterId: string, namespace: string) =>
  invoke<LimitRangeInfo[]>("list_limitranges", { clusterId, namespace });

// ─── Additional Kubernetes Resource Management Commands ───────────────────────

export const cordonNodeCmd = (clusterId: string, nodeName: string) =>
  invoke<void>("cordon_node", { clusterId, nodeName });

export const uncordonNodeCmd = (clusterId: string, nodeName: string) =>
  invoke<void>("uncordon_node", { clusterId, nodeName });

export const drainNodeCmd = (clusterId: string, nodeName: string) =>
  invoke<void>("drain_node", { clusterId, nodeName });

export const rollbackDeploymentCmd = (clusterId: string, namespace: string, deploymentName: string) =>
  invoke<void>("rollback_deployment", { clusterId, namespace, deploymentName });

export const createResourceCmd = (clusterId: string, namespace: string, resourceType: string, yamlContent: string) =>
  invoke<void>("create_resource", { clusterId, namespace, resourceType, yamlContent });

export const editResourceCmd = (clusterId: string, namespace: string, resourceType: string, resourceName: string, yamlContent: string) =>
  invoke<void>("edit_resource", { clusterId, namespace, resourceType, resourceName, yamlContent });

// ─── Missing Resource Types ───────────────────────────────────────────────────

export interface ReplicationControllerInfo {
  name: string;
  namespace: string;
  desired: number;
  ready: number;
  current: number;
  age: string;
}

export interface PodDisruptionBudgetInfo {
  name: string;
  namespace: string;
  min_available: string;
  max_unavailable: string;
  disruptions_allowed: number;
  age: string;
}

export interface PriorityClassInfo {
  name: string;
  value: number;
  global_default: boolean;
  age: string;
}

export interface RuntimeClassInfo {
  name: string;
  handler: string;
  age: string;
}

export interface LeaseInfo {
  name: string;
  namespace: string;
  holder: string;
  age: string;
}

export interface WebhookConfigInfo {
  name: string;
  webhooks: number;
  age: string;
}

export interface EndpointInfo {
  name: string;
  namespace: string;
  addresses: string[];
  ports: string[];
  age: string;
}

export interface EndpointSliceInfo {
  name: string;
  namespace: string;
  address_type: string;
  endpoints: number;
  ports: string[];
  age: string;
}

export interface IngressClassInfo {
  name: string;
  controller: string;
  is_default: boolean;
  age: string;
}

export interface NamespaceResourceInfo {
  name: string;
  status: string;
  age: string;
}

// ─── Helm Types ───────────────────────────────────────────────────────────────

export interface HelmRepository {
  name: string;
  url: string;
}

export interface HelmChart {
  name: string;
  chart_version: string;
  app_version: string;
  description: string;
  repository: string;
}

export interface HelmRelease {
  name: string;
  namespace: string;
  chart: string;
  chart_version: string;
  app_version: string;
  status: string;
  updated: string;
}

// ─── Custom Resource / CRD Types ─────────────────────────────────────────────

export interface PrinterColumn {
  name: string;
  json_path: string;
  type: string;
  description?: string;
  priority: number;
}

export interface CrdVersion {
  name: string;
  served: boolean;
  storage: boolean;
  printer_columns: PrinterColumn[];
}

export interface CrdInfo {
  name: string;
  group: string;
  version: string;
  versions: CrdVersion[];
  kind: string;
  plural: string;
  scope: string;
  age: string;
}

export interface CustomResourceInfo {
  name: string;
  namespace: string;
  age: string;
  additional_columns: Record<string, string>;
}

// ─── Resource Actions ─────────────────────────────────────────────────────────

export interface DescribeResponse {
  output: string;
}

export interface LogStreamConfig {
  cluster_id: string;
  namespace: string;
  pod_name: string;
  container_name: string;
  follow: boolean;
  timestamps: boolean;
  tail_lines?: number;
}

// ─── New Resource List Commands ───────────────────────────────────────────────

export const listReplicationcontrollersCmd = (clusterId: string, namespace: string) =>
  invoke<ReplicationControllerInfo[]>("list_replicationcontrollers", { clusterId, namespace });

export const listPoddisruptionbudgetsCmd = (clusterId: string, namespace: string) =>
  invoke<PodDisruptionBudgetInfo[]>("list_poddisruptionbudgets", { clusterId, namespace });

export const listPriorityclassesCmd = (clusterId: string) =>
  invoke<PriorityClassInfo[]>("list_priorityclasses", { clusterId });

export const listRuntimeclassesCmd = (clusterId: string) =>
  invoke<RuntimeClassInfo[]>("list_runtimeclasses", { clusterId });

export const listLeasesCmd = (clusterId: string, namespace: string) =>
  invoke<LeaseInfo[]>("list_leases", { clusterId, namespace });

export const listMutatingwebhookconfigurationsCmd = (clusterId: string) =>
  invoke<WebhookConfigInfo[]>("list_mutatingwebhookconfigurations", { clusterId });

export const listValidatingwebhookconfigurationsCmd = (clusterId: string) =>
  invoke<WebhookConfigInfo[]>("list_validatingwebhookconfigurations", { clusterId });

export const listEndpointsCmd = (clusterId: string, namespace: string) =>
  invoke<EndpointInfo[]>("list_endpoints", { clusterId, namespace });

export const listEndpointslicesCmd = (clusterId: string, namespace: string) =>
  invoke<EndpointSliceInfo[]>("list_endpointslices", { clusterId, namespace });

export const listIngressclassesCmd = (clusterId: string) =>
  invoke<IngressClassInfo[]>("list_ingressclasses", { clusterId });

export const listNamespacesResourceCmd = (clusterId: string) =>
  invoke<NamespaceResourceInfo[]>("list_namespaces_resource", { clusterId });

export const createNamespaceCmd = (clusterId: string, name: string) =>
  invoke<void>("create_namespace", { clusterId, name });

export const deleteNamespaceCmd = (clusterId: string, name: string) =>
  invoke<void>("delete_namespace", { clusterId, name });

// ─── Resource Action Commands ─────────────────────────────────────────────────

export const attachPodCmd = (clusterId: string, namespace: string, podName: string, containerName: string) =>
  invoke<ExecSessionResponse>("attach_pod", { clusterId, namespace, podName, containerName });

export const forceDeleteResourceCmd = (clusterId: string, resourceType: string, namespace: string, resourceName: string) =>
  invoke<void>("force_delete_resource", { clusterId, resourceType, namespace, resourceName });

export const describeResourceCmd = (clusterId: string, resourceType: string, namespace: string, resourceName: string) =>
  invoke<DescribeResponse>("describe_resource", { clusterId, resourceType, namespace, resourceName });

export const getResourceYamlCmd = (clusterId: string, resourceType: string, namespace: string, resourceName: string) =>
  invoke<string>("get_resource_yaml", { clusterId, resourceType, namespace, resourceName });

export const restartStatefulsetCmd = (clusterId: string, namespace: string, name: string) =>
  invoke<void>("restart_statefulset", { clusterId, namespace, name });

export const restartDaemonsetCmd = (clusterId: string, namespace: string, name: string) =>
  invoke<void>("restart_daemonset", { clusterId, namespace, name });

export const scaleStatefulsetCmd = (clusterId: string, namespace: string, name: string, replicas: number) =>
  invoke<void>("scale_statefulset", { clusterId, namespace, name, replicas });

export const scaleReplicasetCmd = (clusterId: string, namespace: string, name: string, replicas: number) =>
  invoke<void>("scale_replicaset", { clusterId, namespace, name, replicas });

export const scaleReplicationcontrollerCmd = (clusterId: string, namespace: string, name: string, replicas: number) =>
  invoke<void>("scale_replicationcontroller", { clusterId, namespace, name, replicas });

export const suspendCronjobCmd = (clusterId: string, namespace: string, name: string) =>
  invoke<void>("suspend_cronjob", { clusterId, namespace, name });

export const resumeCronjobCmd = (clusterId: string, namespace: string, name: string) =>
  invoke<void>("resume_cronjob", { clusterId, namespace, name });

export const triggerCronjobCmd = (clusterId: string, namespace: string, name: string) =>
  invoke<void>("trigger_cronjob", { clusterId, namespace, name });

// ─── Log Streaming Commands ───────────────────────────────────────────────────

export const streamPodLogsCmd = (config: LogStreamConfig) =>
  invoke<string>("stream_pod_logs", { config });

export const stopLogStreamCmd = (streamId: string) =>
  invoke<void>("stop_log_stream", { streamId });

// ─── Helm Commands ────────────────────────────────────────────────────────────

export const helmListReposCmd = (clusterId: string) =>
  invoke<HelmRepository[]>("helm_list_repos", { clusterId });

export const helmAddRepoCmd = (clusterId: string, name: string, url: string) =>
  invoke<void>("helm_add_repo", { clusterId, name, url });

export const helmUpdateReposCmd = (clusterId: string) =>
  invoke<void>("helm_update_repos", { clusterId });

export const helmSearchRepoCmd = (clusterId: string, query: string) =>
  invoke<HelmChart[]>("helm_search_repo", { clusterId, query });

export const helmListReleasesCmd = (clusterId: string, namespace: string) =>
  invoke<HelmRelease[]>("helm_list_releases", { clusterId, namespace });

export const helmUninstallCmd = (clusterId: string, namespace: string, releaseName: string) =>
  invoke<void>("helm_uninstall", { clusterId, namespace, releaseName });

export const helmRollbackCmd = (clusterId: string, namespace: string, releaseName: string, revision?: number) =>
  invoke<void>("helm_rollback", { clusterId, namespace, releaseName, revision });

// ─── CRD / Custom Resource Commands ──────────────────────────────────────────

export const listCrdsCmd = (clusterId: string) =>
  invoke<CrdInfo[]>("list_crds", { clusterId });

export const listCustomResourcesCmd = (clusterId: string, group: string, version: string, resource: string, namespace: string) =>
  invoke<CustomResourceInfo[]>("list_custom_resources", { clusterId, group, version, resource, namespace });

// ─── PTY Terminal Commands ────────────────────────────────────────────────────

export interface PtySessionInfo {
  session_id: string;
  cluster_id: string;
  namespace: string;
  pod_name: string;
  container_name: string | null;
  session_type: "exec" | "attach";
}

export const startPtyExecSessionCmd = (
  clusterId: string,
  namespace: string,
  podName: string,
  containerName: string | null,
  _shell: string
) =>
  invoke<string>("start_pty_exec_session", {
    clusterId,
    namespace,
    pod: podName,
    container: containerName,
  });

export const startPtyAttachSessionCmd = (
  clusterId: string,
  namespace: string,
  podName: string,
  containerName: string | null
) =>
  invoke<string>("start_pty_attach_session", {
    clusterId,
    namespace,
    pod: podName,
    container: containerName,
  });

export const sendPtyStdinCmd = (sessionId: string, data: string) =>
  invoke<void>("send_pty_stdin", { sessionId, data });

export const resizePtySessionCmd = (sessionId: string, rows: number, cols: number) =>
  invoke<void>("resize_pty_session", { sessionId, rows, cols });

export const terminatePtySessionCmd = (sessionId: string) =>
  invoke<void>("terminate_pty_session", { sessionId });

export const listPtySessionsCmd = () => invoke<PtySessionInfo[]>("list_pty_sessions", {});

// ─── Metrics ─────────────────────────────────────────────────────────────────

export interface ContainerMetrics {
  name: string;
  cpu: string;
  memory: string;
}

export interface PodMetrics {
  name: string;
  namespace: string;
  containers: ContainerMetrics[];
  cpu: string;
  memory: string;
}

export interface NodeMetrics {
  name: string;
  cpu: string;
  memory: string;
  cpu_percent: number;
  memory_percent: number;
}

export const getPodMetricsCmd = (clusterId: string, namespace: string) =>
  invoke<PodMetrics[]>("get_pod_metrics", { clusterId, namespace });

export const getNodeMetricsCmd = (clusterId: string) =>
  invoke<NodeMetrics[]>("get_node_metrics", { clusterId });
