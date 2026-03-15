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
  event_type: string;
  description: string;
  created_at: number;
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
  resolution_steps: ResolutionStep[];
  conversations: AiConversation[];
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

export const chatMessageCmd = (issueId: string, message: string, providerConfig: ProviderConfig) =>
  invoke<ChatResponse>("chat_message", { issueId, message, providerConfig });

export const listProvidersCmd = () => invoke<ProviderInfo[]>("list_providers");

// ─── Analysis / PII commands ──────────────────────────────────────────────────

export const uploadLogFileCmd = (issueId: string, filePath: string) =>
  invoke<LogFile>("upload_log_file", { issueId, filePath });

export const detectPiiCmd = (logFileId: string) =>
  invoke<PiiDetectionResult>("detect_pii", { logFileId });

export const applyRedactionsCmd = (logFileId: string, approvedSpanIds: string[]) =>
  invoke<RedactedLogFile>("apply_redactions", { logFileId, approvedSpanIds });

// ─── Issue CRUD ───────────────────────────────────────────────────────────────

export const createIssueCmd = (newIssue: NewIssue) =>
  invoke<IssueDetail>("create_issue", { newIssue });

export const getIssueCmd = (issueId: string) =>
  invoke<IssueDetail>("get_issue", { issueId });

export const listIssuesCmd = (query: IssueListQuery) =>
  invoke<IssueSummary[]>("list_issues", { query });

export const updateIssueCmd = (
  issueId: string,
  updates: { title?: string; status?: string; severity?: string; description?: string; domain?: string }
) => invoke<IssueDetail>("update_issue", { issueId, ...updates });

export const deleteIssueCmd = (issueId: string) =>
  invoke<void>("delete_issue", { issueId });

export const searchIssuesCmd = (query: string) =>
  invoke<IssueSummary[]>("search_issues", { query });

export const addFiveWhyCmd = (issueId: string, whyNumber: number, question: string, answer?: string) =>
  invoke<FiveWhyEntry>("add_five_why", { issueId, whyNumber, question, answer });

export const updateFiveWhyCmd = (entryId: string, answer: string) =>
  invoke<void>("update_five_why", { entryId, answer });

export const addTimelineEventCmd = (issueId: string, eventType: string, description: string) =>
  invoke<TimelineEvent>("add_timeline_event", { issueId, eventType, description });

// ─── Document commands ────────────────────────────────────────────────────────

export const generateRcaCmd = (issueId: string) => invoke<Document_>("generate_rca", { issueId });

export const generatePostmortemCmd = (issueId: string) =>
  invoke<Document_>("generate_postmortem", { issueId });

export const updateDocumentCmd = (docId: string, contentMd: string) =>
  invoke<Document_>("update_document", { docId, contentMd });

export const exportDocumentCmd = (docId: string, format: string, outputDir: string) =>
  invoke<string>("export_document", { docId, format, outputDir });

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
