use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Remote Protocol Enum ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RemoteProtocol {
    Rdp,
    Vnc,
}

impl std::fmt::Display for RemoteProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteProtocol::Rdp => write!(f, "rdp"),
            RemoteProtocol::Vnc => write!(f, "vnc"),
        }
    }
}

// ─── Issue ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub category: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
    pub assigned_to: String,
    pub tags: String,
}

impl Issue {
    pub fn new(title: String, description: String, severity: String, category: String) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Issue {
            id: Uuid::now_v7().to_string(),
            title,
            description,
            severity,
            status: "open".to_string(),
            category,
            source: "manual".to_string(),
            created_at: now.clone(),
            updated_at: now,
            resolved_at: None,
            assigned_to: String::new(),
            tags: "[]".to_string(),
        }
    }
}

/// Full detail view returned by get_issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueDetail {
    pub issue: Issue,
    pub log_files: Vec<LogFile>,
    pub image_attachments: Vec<ImageAttachment>,
    pub resolution_steps: Vec<ResolutionStep>,
    pub conversations: Vec<AiConversation>,
    pub timeline_events: Vec<TimelineEvent>,
}

/// Lightweight row returned by list/search commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSummary {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub status: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
    pub log_count: i64,
    pub step_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueFilter {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub domain: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewIssue {
    pub title: String,
    pub domain: String,
    pub description: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
    pub domain: Option<String>,
    pub assigned_to: Option<String>,
    pub tags: Option<String>,
}

// ─── 5-Whys ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiveWhyEntry {
    pub id: String,
    pub why_number: i32,
    pub question: String,
    pub answer: Option<String>,
    pub created_at: i64,
}

// ─── Timeline ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub issue_id: String,
    pub event_type: String,
    pub description: String,
    pub metadata: String,
    pub created_at: String,
}

impl TimelineEvent {
    pub fn new(
        issue_id: String,
        event_type: String,
        description: String,
        metadata: String,
    ) -> Self {
        TimelineEvent {
            id: Uuid::now_v7().to_string(),
            issue_id,
            event_type,
            description,
            metadata,
            created_at: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        }
    }
}

// ─── Log File ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFile {
    pub id: String,
    pub issue_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub content_hash: String,
    pub uploaded_at: String,
    pub redacted: bool,
}

impl LogFile {
    pub fn new(issue_id: String, file_name: String, file_path: String, file_size: i64) -> Self {
        LogFile {
            id: Uuid::now_v7().to_string(),
            issue_id,
            file_name,
            file_path,
            file_size,
            mime_type: "text/plain".to_string(),
            content_hash: String::new(),
            uploaded_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            redacted: false,
        }
    }
}

// ─── PII ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiSpanRecord {
    pub id: String,
    pub log_file_id: String,
    pub pii_type: String,
    pub start_offset: i64,
    pub end_offset: i64,
    pub original_value: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiDetectionResult {
    pub log_file_id: String,
    pub detections: Vec<crate::pii::PiiSpan>,
    pub total_pii_found: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedLogFile {
    pub id: String,
    pub original_file_id: String,
    pub file_name: String,
    pub file_hash: String,
    pub redaction_count: usize,
}

// ─── AI Conversation ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConversation {
    pub id: String,
    pub issue_id: String,
    pub provider: String,
    pub model: String,
    pub created_at: String,
    pub title: String,
}

impl AiConversation {
    pub fn new(issue_id: String, provider: String, model: String) -> Self {
        AiConversation {
            id: Uuid::now_v7().to_string(),
            issue_id,
            provider,
            model,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            title: "Untitled".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub token_count: i64,
    pub created_at: String,
}

impl AiMessage {
    pub fn new(conversation_id: String, role: String, content: String) -> Self {
        AiMessage {
            id: Uuid::now_v7().to_string(),
            conversation_id,
            role,
            content,
            token_count: 0,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

// ─── Resolution Step ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionStep {
    pub id: String,
    pub issue_id: String,
    pub step_order: i64,
    pub why_question: String,
    pub answer: String,
    pub evidence: String,
    pub created_at: String,
}

impl ResolutionStep {
    pub fn new(
        issue_id: String,
        step_order: i64,
        why_question: String,
        answer: String,
        evidence: String,
    ) -> Self {
        ResolutionStep {
            id: Uuid::now_v7().to_string(),
            issue_id,
            step_order,
            why_question,
            answer,
            evidence,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

// ─── Document ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub issue_id: String,
    pub doc_type: String,
    pub title: String,
    pub content_md: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Document {
    pub fn new(issue_id: String, doc_type: String, title: String, content_md: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Document {
            id: Uuid::now_v7().to_string(),
            issue_id,
            doc_type,
            title,
            content_md,
            created_at: now,
            updated_at: now,
        }
    }
}

// ─── Audit ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub user_id: String,
    pub details: String,
}

impl AuditEntry {
    pub fn new(action: String, entity_type: String, entity_id: String, details: String) -> Self {
        AuditEntry {
            id: Uuid::now_v7().to_string(),
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            action,
            entity_type,
            entity_id,
            user_id: "local".to_string(),
            details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditFilter {
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ─── Settings ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingRecord {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

// ─── Integrations ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub service: String,
    pub token_hash: String,
    pub encrypted_token: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

impl Credential {
    pub fn new(service: String, token_hash: String, encrypted_token: String) -> Self {
        Credential {
            id: Uuid::now_v7().to_string(),
            service,
            token_hash,
            encrypted_token,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            expires_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub id: String,
    pub service: String,
    pub base_url: String,
    pub username: Option<String>,
    pub project_name: Option<String>,
    pub space_key: Option<String>,
    pub auto_create_enabled: bool,
    pub updated_at: String,
}

impl IntegrationConfig {
    pub fn new(service: String, base_url: String) -> Self {
        IntegrationConfig {
            id: Uuid::now_v7().to_string(),
            service,
            base_url,
            username: None,
            project_name: None,
            space_key: None,
            auto_create_enabled: false,
            updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

// ─── Image Attachment ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAttachment {
    pub id: String,
    pub issue_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub upload_hash: String,
    pub uploaded_at: String,
    pub pii_warning_acknowledged: bool,
    pub is_paste: bool,
}

// ─── Attachment Summaries (cross-incident list views) ───────────────────────

/// Lightweight log-file row joined with the parent issue title.
/// Returned by `list_all_log_files` — never contains the compressed content blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileSummary {
    pub id: String,
    pub issue_id: String,
    pub issue_title: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub content_hash: String,
    pub uploaded_at: String,
    pub redacted: bool,
}

/// Lightweight image-attachment row joined with the parent issue title.
/// Returned by `list_all_image_attachments` — never contains the raw image bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAttachmentSummary {
    pub id: String,
    pub issue_id: String,
    pub issue_title: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub upload_hash: String,
    pub uploaded_at: String,
    pub pii_warning_acknowledged: bool,
    pub is_paste: bool,
}

// ─── Kubernetes Cluster ─────────────────────────────────────────────────────

/// Represents a Kubernetes cluster configuration stored in the database.
/// The kubeconfig content is stored directly in the clusters table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: String,
    pub name: String,
    pub context: String,
    pub server_url: Option<String>,
    pub kubeconfig_content: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Cluster {
    pub fn new(
        name: String,
        context: String,
        server_url: Option<String>,
        kubeconfig_content: String,
    ) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Cluster {
            id: Uuid::now_v7().to_string(),
            name,
            context,
            server_url,
            kubeconfig_content,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Lightweight summary for cluster list views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSummary {
    pub id: String,
    pub name: String,
    pub context: String,
    pub server_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub port_forward_count: i64,
}

// ─── Port Forward ───────────────────────────────────────────────────────────

/// Represents a port forwarding session for a Kubernetes cluster.
/// The ports and local_ports are stored as JSON arrays of u16.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub id: String,
    pub cluster_id: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub ports: Vec<u16>,
    pub local_ports: Vec<u16>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl PortForward {
    pub fn new(
        cluster_id: String,
        namespace: String,
        pod: String,
        container: Option<String>,
        ports: Vec<u16>,
        local_ports: Vec<u16>,
    ) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        PortForward {
            id: Uuid::now_v7().to_string(),
            cluster_id,
            namespace,
            pod,
            container,
            ports,
            local_ports,
            status: "Active".to_string(),
            error_message: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Lightweight summary for port forward list views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardSummary {
    pub id: String,
    pub cluster_id: String,
    pub cluster_name: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub ports: Vec<u16>,
    pub local_ports: Vec<u16>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Filter for listing clusters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClusterFilter {
    pub name: Option<String>,
    pub context: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Filter for listing port forwards.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortForwardFilter {
    pub cluster_id: Option<String>,
    pub status: Option<String>,
    pub namespace: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// New cluster data for creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCluster {
    pub name: String,
    pub context: String,
    pub server_url: String,
    pub kubeconfig_content: String,
}

/// Update for existing cluster.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClusterUpdate {
    pub name: Option<String>,
    pub context: Option<String>,
    pub server_url: Option<String>,
    pub kubeconfig_content: Option<String>,
}

/// New port forward data for creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPortForward {
    pub cluster_id: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub ports: Vec<u16>,
    pub local_ports: Vec<u16>,
}

/// Update for existing port forward.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortForwardUpdate {
    pub status: Option<String>,
    pub error_message: Option<String>,
}

impl ImageAttachment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        issue_id: String,
        file_name: String,
        file_path: String,
        file_size: i64,
        mime_type: String,
        upload_hash: String,
        pii_warning_acknowledged: bool,
        is_paste: bool,
    ) -> Self {
        ImageAttachment {
            id: Uuid::now_v7().to_string(),
            issue_id,
            file_name,
            file_path,
            file_size,
            mime_type,
            upload_hash,
            uploaded_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            pii_warning_acknowledged,
            is_paste,
        }
    }
}

// ─── Remote Connections ─────────────────────────────────────────────────────

/// Represents a remote desktop connection (RDP, VNC, etc.)
/// Credentials are stored separately in the remote_credentials table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConnection {
    pub id: String,
    pub name: String,
    pub protocol: RemoteProtocol,
    pub hostname: String,
    pub port: u16,
    pub username: Option<String>,
    pub domain: Option<String>,
    // SSH tunnel configuration (non-sensitive)
    pub ssh_enabled: bool,
    pub ssh_hostname: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_username: Option<String>,
    // Display settings
    pub resolution: String,
    pub color_depth: u32,
    pub clipboard_sync: bool,
    pub drive_redirect: bool,
    pub multi_monitor: bool,
    pub compression: bool,
    pub quality: u32,
    pub auto_resize: bool,
    pub stretch_to_fill: bool,
    // Metadata
    pub created_at: String,
    pub updated_at: String,
    pub last_connected_at: Option<String>,
}

/// Input for creating a new remote connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRemoteConnection {
    pub name: String,
    pub protocol: RemoteProtocol,
    pub hostname: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: String, // Will be encrypted and stored separately
    pub domain: Option<String>,
    // SSH tunnel configuration
    pub ssh_enabled: bool,
    pub ssh_hostname: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_username: Option<String>,
    pub ssh_password: Option<String>,
    pub ssh_key_data: Option<String>,
    pub ssh_key_passphrase: Option<String>,
    // Display settings
    pub resolution: Option<String>,
    pub color_depth: Option<u32>,
    pub clipboard_sync: Option<bool>,
    pub drive_redirect: Option<bool>,
    pub multi_monitor: Option<bool>,
    pub compression: Option<bool>,
    pub quality: Option<u32>,
    pub auto_resize: bool,
    pub stretch_to_fill: bool,
}

/// Update for an existing remote connection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteConnectionUpdate {
    pub name: Option<String>,
    pub protocol: Option<RemoteProtocol>,
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub username: Option<Option<String>>,
    pub domain: Option<Option<String>>,
    // SSH tunnel configuration
    pub ssh_enabled: Option<bool>,
    pub ssh_hostname: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_username: Option<String>,
    // Display settings
    pub resolution: Option<String>,
    pub color_depth: Option<u32>,
    pub clipboard_sync: Option<bool>,
    pub drive_redirect: Option<bool>,
    pub multi_monitor: Option<bool>,
    pub compression: Option<bool>,
    pub quality: Option<u32>,
    pub auto_resize: Option<bool>,
    pub stretch_to_fill: Option<bool>,
}

/// Lightweight summary for listing remote connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConnectionSummary {
    pub id: String,
    pub name: String,
    pub protocol: RemoteProtocol,
    pub hostname: String,
    pub port: u16,
    pub username: Option<String>,
    pub status: String,
    pub ssh_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_connected_at: Option<String>,
}

/// Filter for listing remote connections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteConnectionFilter {
    pub protocol: Option<RemoteProtocol>,
    pub name: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Encrypted credentials for remote connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCredentials {
    pub id: String,
    pub connection_id: String,
    pub rdp_password_encrypted: Option<String>,
    pub ssh_password_encrypted: Option<String>,
    pub ssh_key_encrypted: Option<String>,
    pub ssh_key_passphrase_encrypted: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl RemoteCredentials {
    pub fn new(
        connection_id: String,
        rdp_password: Option<String>,
        ssh_password: Option<String>,
        ssh_key: Option<String>,
        ssh_key_passphrase: Option<String>,
    ) -> Result<Self, String> {
        use crate::integrations::auth::encrypt_token;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let rdp_password_encrypted = if let Some(pwd) = rdp_password {
            Some(encrypt_token(&pwd).map_err(|e| e.to_string())?)
        } else {
            None
        };

        let ssh_password_encrypted = if let Some(pwd) = ssh_password {
            Some(encrypt_token(&pwd).map_err(|e| e.to_string())?)
        } else {
            None
        };

        let ssh_key_encrypted = if let Some(key) = ssh_key {
            Some(encrypt_token(&key).map_err(|e| e.to_string())?)
        } else {
            None
        };

        let ssh_key_passphrase_encrypted = if let Some(pass) = ssh_key_passphrase {
            Some(encrypt_token(&pass).map_err(|e| e.to_string())?)
        } else {
            None
        };

        Ok(RemoteCredentials {
            id: Uuid::now_v7().to_string(),
            connection_id,
            rdp_password_encrypted,
            ssh_password_encrypted,
            ssh_key_encrypted,
            ssh_key_passphrase_encrypted,
            created_at: now.clone(),
            updated_at: now,
        })
    }
}

/// Input for creating remote credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRemoteCredentials {
    pub connection_id: String,
    pub rdp_password: Option<String>,
    pub ssh_password: Option<String>,
    pub ssh_key: Option<String>,
    pub ssh_key_passphrase: Option<String>,
}

/// Update for existing remote credentials
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteCredentialsUpdate {
    pub rdp_password: Option<String>,
    pub ssh_password: Option<String>,
    pub ssh_key: Option<String>,
    pub ssh_key_passphrase: Option<String>,
}
