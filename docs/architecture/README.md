# TRCAA Architecture Documentation

**Troubleshooting and RCA Assistant** — C4-model architecture documentation using Mermaid diagrams.

---

## Table of Contents

1. [System Context (C4 Level 1)](#system-context)
2. [Container Architecture (C4 Level 2)](#container-architecture)
3. [Component Architecture (C4 Level 3)](#component-architecture)
4. [Data Architecture](#data-architecture)
5. [Security Architecture](#security-architecture)
6. [AI Provider Architecture](#ai-provider-architecture)
7. [Integration Architecture](#integration-architecture)
8. [Deployment Architecture](#deployment-architecture)
9. [Key Data Flows](#key-data-flows)
10. [Architecture Decision Records](#architecture-decision-records)

---

## System Context

The system context diagram shows TRCAA in relation to its users and external systems.

```mermaid
C4Context
    title System Context — Troubleshooting and RCA Assistant

    Person(it_eng, "IT Engineer", "Diagnoses incidents and conducts root cause analysis")

    System(trcaa, "TRCAA Desktop App", "Structured AI-backed assistant for IT troubleshooting, 5-whys RCA, and post-mortem documentation")

    System_Ext(ollama, "Ollama (Local)", "Runs open-source LLMs locally (llama3, mistral, phi3)")
    System_Ext(openai, "OpenAI API", "GPT-4o, GPT-4o-mini for cloud AI inference")
    System_Ext(anthropic, "Anthropic API", "Claude 3.5 Sonnet, Claude Haiku")
    System_Ext(gemini, "Google Gemini API", "Gemini Pro for cloud AI inference")
    System_Ext(custom_rest, "Custom REST Gateway", "Enterprise AI gateway (custom REST format)")

    System_Ext(confluence, "Confluence", "Atlassian wiki — publish RCA docs")
    System_Ext(servicenow, "ServiceNow", "ITSM platform — create incident tickets")
    System_Ext(ado, "Azure DevOps", "Work item tracking and collaboration")

    Rel(it_eng, trcaa, "Uses", "Desktop app (Tauri WebView)")
    Rel(trcaa, ollama, "AI inference", "HTTP/JSON (local)")
    Rel(trcaa, openai, "AI inference", "HTTPS/REST")
    Rel(trcaa, anthropic, "AI inference", "HTTPS/REST")
    Rel(trcaa, gemini, "AI inference", "HTTPS/REST")
    Rel(trcaa, custom_rest, "AI inference", "HTTPS/REST")
    Rel(trcaa, confluence, "Publish RCA docs", "HTTPS/REST + OAuth2")
    Rel(trcaa, servicenow, "Create incidents", "HTTPS/REST + OAuth2")
    Rel(trcaa, ado, "Create work items", "HTTPS/REST + OAuth2")
```

---

## Container Architecture

TRCAA is a single-process Tauri 2 desktop application. The "containers" are logical boundaries within the process.

```mermaid
C4Container
    title Container Architecture — TRCAA

    Person(user, "IT Engineer")

    System_Boundary(trcaa, "TRCAA Desktop Process") {
        Container(webview, "React Frontend", "React 18 + TypeScript + Vite", "Renders UI via OS WebView (WebKit/WebView2). Manages ephemeral session state and persisted settings.")
        Container(tauri_core, "Tauri Core / IPC Bridge", "Rust / Tauri 2", "Routes invoke() calls between WebView and backend command handlers. Enforces capability ACL.")
        Container(rust_backend, "Rust Backend", "Rust / Tokio async", "Command handlers, AI provider clients, PII engine, document generation, integration clients, audit logging.")
        ContainerDb(db, "SQLCipher Database", "SQLite + SQLCipher AES-256", "All persistent data: issues, logs, messages, audit trail, credentials, AI provider configs.")
        ContainerDb(stronghold, "Stronghold Key Store", "tauri-plugin-stronghold", "Encrypted key-value store for symmetric key material.")
        ContainerDb(local_fs, "Local Filesystem", "App data directory", "Redacted log files, .dbkey, .enckey, exported documents.")
    }

    System_Ext(ai_providers, "AI Providers", "OpenAI, Anthropic, Gemini, Mistral, Ollama")
    System_Ext(integrations, "Integrations", "Confluence, ServiceNow, Azure DevOps")

    Rel(user, webview, "Interacts with", "Mouse/keyboard via OS WebView")
    Rel(webview, tauri_core, "IPC calls", "invoke() / Tauri JS bridge")
    Rel(tauri_core, rust_backend, "Dispatches commands", "Rust function calls")
    Rel(rust_backend, db, "Reads/writes", "rusqlite (sync, mutex-guarded)")
    Rel(rust_backend, stronghold, "Reads/writes keys", "Plugin API")
    Rel(rust_backend, local_fs, "Reads/writes files", "std::fs")
    Rel(rust_backend, ai_providers, "AI inference", "reqwest async HTTP")
    Rel(rust_backend, integrations, "API calls", "reqwest async HTTP + OAuth2")
```

---

## Component Architecture

### Shell Execution System (v1.0.0+)

**Status**: Production-ready agentic shell command execution with three-tier safety classification.

**Architecture**: Three-tier safety system with automatic classification, approval gates, and audit logging.

```mermaid
graph TB
    subgraph "Shell Execution Architecture"
        AI[AI Agent] -->|tool_call| ToolRegistry[Tool Registry]
        ToolRegistry -->|execute_shell_command| Classifier[Command Classifier]
        
        Classifier -->|analyze| Parser[Command Parser]
        Parser -->|components| RiskAnalyzer[Risk Analyzer]
        
        RiskAnalyzer -->|Tier 1| AutoExec[Auto Execute]
        RiskAnalyzer -->|Tier 2| ApprovalGate[Approval Gate]
        RiskAnalyzer -->|Tier 3| Deny[Always Deny]
        
        ApprovalGate -->|user decision| ApprovalModal[Approval Modal UI]
        ApprovalModal -->|allow| Executor[Command Executor]
        ApprovalModal -->|deny| AuditLog[Audit Log]
        
        AutoExec --> Executor
        Deny --> AuditLog
        
        Executor -->|kubectl| KubectlBinary[kubectl Binary v1.30.0]
        Executor -->|shell| SystemShell[System Shell]
        
        Executor --> ExecutionRecord[Execution Record]
        ExecutionRecord --> AuditLog
        ExecutionRecord --> Database[(Database)]
        
        Database --> ExecutionHistory[Execution History UI]
    end
    
    style Classifier fill:#e1f5ff
    style ApprovalGate fill:#fff4e6
    style Deny fill:#ffe6e6
    style AutoExec fill:#e6f7e6
    style KubectlBinary fill:#f0e6ff
```

**Three-Tier Safety Classification**:

- **Tier 1 (Auto-execute)**: Read-only operations with no side effects
  - Examples: `kubectl get`, `kubectl describe`, `kubectl logs`, `cat`, `grep`, `ls`, `pvecm status`
  - Executes immediately without user interaction

- **Tier 2 (User approval required)**: Potentially mutating operations
  - Examples: `kubectl apply`, `kubectl delete`, `kubectl scale`, `chmod`, `systemctl restart`, `ssh`
  - Shows real-time approval modal with command details
  - Supports "Allow Once", "Allow for Session", and "Deny"

- **Tier 3 (Always deny)**: Destructive operations
  - Examples: `rm -rf`, `shutdown`, `mkfs`, `dd`, `:(){:|:&};:`
  - Automatically rejected with explanation to user

**Key Modules**:

| Module | Responsibility | Key Features |
|--------|---------------|--------------|
| `shell/classifier.rs` | Command safety classification | 19 unit tests, pipe/chain analysis, command substitution detection |
| `shell/executor.rs` | Execution flow with approval gates | Timeout handling, kubeconfig injection, exit code capture |
| `shell/kubectl.rs` | kubectl binary management | Cross-platform binary bundling, version v1.30.0 |
| `shell/kubeconfig.rs` | Kubeconfig parsing and encryption | AES-256-GCM encryption, context extraction, cluster URL parsing |
| `commands/shell.rs` | 7 Tauri IPC commands | kubeconfig CRUD, execution, history retrieval |
| `ai/tools.rs` | Tool registration | `execute_shell_command` tool definition with parameters |

**Database Schema** (Migrations 024-027):

```sql
-- Pre-defined command templates with tier definitions
CREATE TABLE shell_commands (
    id TEXT PRIMARY KEY,
    command_template TEXT NOT NULL,
    tier INTEGER NOT NULL CHECK(tier IN (1, 2, 3)),
    description TEXT,
    category TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Encrypted kubeconfig storage
CREATE TABLE kubeconfig_files (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    encrypted_content TEXT NOT NULL,
    context TEXT NOT NULL,
    cluster_url TEXT,
    is_active INTEGER NOT NULL DEFAULT 0,
    uploaded_at TEXT NOT NULL
);

-- Full audit trail for all executions
CREATE TABLE command_executions (
    id TEXT PRIMARY KEY,
    issue_id TEXT,
    command TEXT NOT NULL,
    tier INTEGER NOT NULL,
    approval_status TEXT NOT NULL,
    kubeconfig_id TEXT,
    exit_code INTEGER,
    stdout TEXT,
    stderr TEXT,
    execution_time_ms INTEGER,
    executed_at TEXT NOT NULL
);

-- Session-based approval preferences
CREATE TABLE approval_decisions (
    id TEXT PRIMARY KEY,
    command_pattern TEXT NOT NULL,
    decision TEXT NOT NULL CHECK(decision IN ('allow_once', 'allow_session', 'deny')),
    session_id TEXT,
    decided_at TEXT NOT NULL,
    expires_at TEXT
);
```

**Security Features**:
- AES-256-GCM encryption for kubeconfig files
- Command tier escalation for pipes and command substitution
- Full audit logging of all commands (approved, denied, executed)
- Session-based approval memory with expiration
- kubectl binary bundled and verified (no system dependency)

**Frontend Components**:
- `ShellApprovalModal.tsx`: Real-time approval UI with command preview
- `Settings/ShellExecution.tsx`: Settings and execution history viewer
- `Settings/KubeconfigManager.tsx`: Multi-cluster kubeconfig management

**Documentation**: `docs/wiki/Shell-Execution.md`

---

### MCP Server Integration (v1.0.0+)

**Status**: Production-ready Model Context Protocol integration for external tool protocols.

**Architecture**: Client-server protocol adapter for stdio and HTTP transports.

```mermaid
graph TB
    subgraph "MCP Integration Architecture"
        AI[AI Agent] -->|needs tools| Adapter[MCP Adapter]
        
        Adapter -->|fetch tools| Store[MCP Store]
        Store -->|load servers| Database[(Database)]
        
        Adapter -->|for each enabled server| Discovery[Discovery Service]
        Discovery -->|connect| Client[MCP Client]
        
        Client -->|stdio| StdioTransport[Stdio Transport]
        Client -->|http| HttpTransport[HTTP Transport]
        
        StdioTransport -->|spawn process| ExternalServer1[MCP Server Process]
        HttpTransport -->|HTTP POST| ExternalServer2[MCP HTTP Server]
        
        Client -->|list_tools| ServerCapabilities[Server Capabilities]
        ServerCapabilities -->|return tools| ToolRegistry[Tool Registry]
        
        AI -->|call tool| ToolExecutor[Tool Executor]
        ToolExecutor -->|invoke| Client
        Client -->|call_tool| ExternalServer1
        ExternalServer1 -->|result| Client
        Client -->|30s timeout| ToolExecutor
        
        Discovery -->|update status| Database
    end
    
    style Discovery fill:#e1f5ff
    style Client fill:#fff4e6
    style Database fill:#e6f7e6
```

**Key Modules**:

| Module | Responsibility | Key Features |
|--------|---------------|--------------|
| `mcp/client.rs` | Connect to MCP servers | Stdio/HTTP transports, 30s tool call timeout |
| `mcp/adapter.rs` | Tool registry integration | Fetch tools from all enabled servers, merge with static tools |
| `mcp/discovery.rs` | Server health checks | Connection status updates, error tracking |
| `mcp/store.rs` | Database CRUD | Server config, tool/resource persistence |
| `mcp/models.rs` | Data models | McpServer, McpTool, McpResource types |
| `mcp/transport/stdio.rs` | Stdio transport | Process spawning, environment variables |
| `mcp/transport/http.rs` | HTTP transport | Custom headers, auth support |
| `mcp/commands.rs` | 7 Tauri IPC commands | Server CRUD, discovery, tool/resource listing |

**Database Schema** (Migration 018):

```sql
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    transport_type TEXT NOT NULL CHECK(transport_type IN ('stdio', 'http')),
    transport_config TEXT NOT NULL DEFAULT '{}',
    auth_type TEXT NOT NULL CHECK(auth_type IN ('none', 'api_key', 'bearer', 'oauth2')),
    auth_value TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_discovered_at TEXT,
    discovery_status TEXT NOT NULL DEFAULT 'pending'
        CHECK(discovery_status IN ('pending','connected','unreachable','error')),
    discovery_error TEXT,
    env_config TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE mcp_tools (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    name TEXT NOT NULL,
    tool_key TEXT NOT NULL,
    description TEXT,
    parameters TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY(server_id) REFERENCES mcp_servers(id) ON DELETE CASCADE
);

CREATE TABLE mcp_resources (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    uri TEXT NOT NULL,
    name TEXT,
    description TEXT,
    FOREIGN KEY(server_id) REFERENCES mcp_servers(id) ON DELETE CASCADE
);
```

**Tool Calling Flow**:
1. AI agent requests available tools
2. Adapter fetches static tools (`ai/tools.rs::get_available_tools()`)
3. Adapter fetches MCP tools from all enabled servers
4. Tools merged and returned to AI agent
5. AI agent calls tool by name (e.g., `server_name.tool_name`)
6. Adapter routes to correct MCP client
7. Client invokes tool with 30-second timeout
8. Result returned to AI agent

**Security**:
- Auth credentials stored with AES-256-GCM encryption
- Environment variables isolated per server process
- 30-second hard timeout prevents indefinite hangs
- Server connection status tracked and displayed

**Frontend Components**:
- `Settings/MCPServers.tsx`: Server configuration and discovery UI
- `Settings/MCPTools.tsx`: Tool browser and tester

---

### AI Tool Calling & Auto-Detection (v1.0.8+)

**Status**: Production-ready automatic tool calling support detection.

**Architecture**: Test-based detection with graceful degradation.

```mermaid
graph TB
    subgraph "Tool Calling Detection"
        User[User] -->|clicks detect| UI[Auto-Detect Button]
        UI -->|invoke| Command[detect_tool_calling_support]
        
        Command -->|create test tool| TestTool[test_tool definition]
        Command -->|override config| DetectionConfig[Detection Config]
        
        DetectionConfig -->|max_tokens: 100| Optimization1[Cost Optimization]
        DetectionConfig -->|temperature: 0.0| Optimization2[Deterministic]
        
        Command -->|send test call| Provider[AI Provider]
        Provider -->|response| Parser[Response Parser]
        
        Parser -->|has tool_calls array?| Decision{Supports Tools?}
        Decision -->|yes, contains test_tool| Success[Return true]
        Decision -->|no tool_calls| NotSupported[Return false]
        Decision -->|503 / tool error| Blocked[Return false]
        Decision -->|connection error| Error[Throw error]
        
        Success -->|update UI| Checkbox[Enable Checkbox]
        NotSupported -->|update UI| DisableCheckbox[Disable Checkbox]
        Blocked -->|update UI| DisableCheckbox
        Error -->|display| ErrorMessage[Error Message]
    end
    
    style Success fill:#e6f7e6
    style NotSupported fill:#fff4e6
    style Blocked fill:#ffe6e6
    style Error fill:#ffe6e6
```

**Test Tool Definition**:

```rust
Tool {
    name: "test_tool".to_string(),
    description: "A test tool that returns 'success'. Call this tool with no arguments.".to_string(),
    parameters: ToolParameters {
        param_type: "object".to_string(),
        properties: HashMap::new(),
        required: vec![],
    },
}
```

**Detection Criteria**:

| Scenario | Result | Action |
|----------|--------|--------|
| Provider returns `tool_calls` array with `test_tool` | ✅ Supported | Enable checkbox, show success message |
| Provider responds without `tool_calls` | ⚠️ Not supported | Disable checkbox, show warning |
| Gateway returns 503 / "tool" error (e.g., MSI GenAI) | ⚠️ Blocked | Disable checkbox, show warning |
| Connection/auth/timeout error | ❌ Error | Show error message, don't change checkbox |

**Optimizations**:
- `max_tokens: 100` (reduces cost for detection test)
- `temperature: 0.0` (deterministic responses)
- Error pattern matching for gateway-level blocks

**Key Files**:
- `commands/ai.rs::detect_tool_calling_support()`: Backend detection logic (5 unit tests)
- `pages/Settings/AIProviders.tsx::handleAutoDetectToolCalling()`: Frontend UI (7 unit tests)
- `lib/tauriCommands.ts::detectToolCallingSupportCmd()`: TypeScript wrapper

**Database**: Uses `ai_providers.supports_tool_calling` column (Migration 028)

**Documentation**: `docs/wiki/AI-Providers.md` section "Tool Calling Auto-Detection"

---

### Backend Components

```mermaid
graph TD
    subgraph "Tauri IPC Layer"
        IPC[IPC Command Router\nlib.rs generate_handler!]
    end

    subgraph "Command Handlers (commands/)"
        CMD_DB[db.rs\nIssue CRUD\nTimeline Events\n5-Whys Entries]
        CMD_AI[ai.rs\nChat Message\nLog Analysis\nProvider Test\nTool Calling Detection]
        CMD_ANALYSIS[analysis.rs\nLog Upload\nPII Detection\nRedaction Apply]
        CMD_DOCS[docs.rs\nRCA Generation\nPostmortem Gen\nDocument Export]
        CMD_INTEGRATIONS[integrations.rs\nConfluence\nServiceNow\nAzure DevOps\nOAuth Flow]
        CMD_SYSTEM[system.rs\nSettings CRUD\nOllama Mgmt\nAI Provider Mgmt\nAudit Log]
        CMD_SHELL[shell.rs\nKubeconfig CRUD\nCommand Execution\nExecution History]
        CMD_MCP[mcp/commands.rs\nMCP Server CRUD\nDiscovery\nTool/Resource Listing]
    end

    subgraph "Domain Services"
        AI[AI Layer\nai/provider.rs\nTrait + Factory]
        TOOLS[AI Tools\nai/tools.rs\nStatic Tools Registry]
        AGENTS[AI Agents\nai/agents.rs\nAgent Registry]
        PII[PII Engine\npii/detector.rs\n12 Pattern Detectors]
        AUDIT[Audit Logger\naudit/log.rs\nHash-chained entries]
        DOCS_GEN[Doc Generator\ndocs/rca.rs\ndocs/postmortem.rs]
        SHELL[Shell System\nshell/classifier.rs\nshell/executor.rs\nshell/kubectl.rs]
        MCP[MCP Integration\nmcp/client.rs\nmcp/adapter.rs\nmcp/discovery.rs]
    end

    subgraph "AI Providers (ai/)"
        ANTHROPIC[anthropic.rs\nClaude API]
        OPENAI[openai.rs\nOpenAI + Custom REST]
        OLLAMA[ollama.rs\nLocal Models]
        GEMINI[gemini.rs\nGoogle Gemini]
        MISTRAL[mistral.rs\nMistral API]
    end

    subgraph "Integration Clients (integrations/)"
        CONFLUENCE[confluence.rs\nconfluence_search.rs]
        SERVICENOW[servicenow.rs\nservicenow_search.rs]
        AZUREDEVOPS[azuredevops.rs\nazuredevops_search.rs]
        AUTH[auth.rs\nAES-256-GCM\nToken Encryption]
        WEBVIEW_AUTH[webview_auth.rs\nOAuth WebView\nCallback Server]
    end

    subgraph "Data Layer (db/)"
        MIGRATIONS[migrations.rs\n28 Schema Versions]
        MODELS[models.rs\nIssue / LogFile\nAiMessage / Document\nAuditEntry / Credential\nShellCommand / KubeconfigFile\nCommandExecution\nMcpServer / McpTool]
        CONNECTION[connection.rs\nSQLCipher Connect\nKey Auto-gen\nPlain→Encrypted Migration]
    end

    IPC --> CMD_DB
    IPC --> CMD_AI
    IPC --> CMD_ANALYSIS
    IPC --> CMD_DOCS
    IPC --> CMD_INTEGRATIONS
    IPC --> CMD_SYSTEM
    IPC --> CMD_SHELL
    IPC --> CMD_MCP

    CMD_AI --> AI
    CMD_AI --> TOOLS
    CMD_ANALYSIS --> PII
    CMD_DOCS --> DOCS_GEN
    CMD_SHELL --> SHELL
    CMD_MCP --> MCP
    CMD_INTEGRATIONS --> CONFLUENCE
    CMD_INTEGRATIONS --> SERVICENOW
    CMD_INTEGRATIONS --> AZUREDEVOPS
    CMD_INTEGRATIONS --> AUTH
    CMD_INTEGRATIONS --> WEBVIEW_AUTH

    AI --> ANTHROPIC
    AI --> OPENAI
    AI --> OLLAMA
    AI --> GEMINI
    AI --> MISTRAL
    
    TOOLS --> SHELL
    TOOLS --> MCP
    MCP --> AGENTS

    CMD_DB --> MODELS
    CMD_AI --> AUDIT
    CMD_ANALYSIS --> AUDIT
    CMD_SHELL --> AUDIT
    MODELS --> MIGRATIONS
    MIGRATIONS --> CONNECTION

    style IPC fill:#4a90d9,color:#fff
    style AI fill:#7b68ee,color:#fff
    style PII fill:#e67e22,color:#fff
    style AUDIT fill:#c0392b,color:#fff
```

### Frontend Components

```mermaid
graph TD
    subgraph "React Application (src/)"
        APP[App.tsx\nSidebar + Router\nTheme Provider]
    end

    subgraph "Pages (src/pages/)"
        DASHBOARD[Dashboard\nStats + Quick Actions]
        NEW_ISSUE[NewIssue\nCreate Form]
        LOG_UPLOAD[LogUpload\nFile Upload + PII Review]
        TRIAGE[Triage\n5-Whys AI Chat]
        RESOLUTION[Resolution\nStep Tracking]
        RCA[RCA\nDocument Editor]
        POSTMORTEM[Postmortem\nDocument Editor]
        HISTORY[History\nSearch + Filter]
        SETTINGS[Settings\nProviders / Ollama\nIntegrations / Security]
    end

    subgraph "Components (src/components/)"
        CHAT_WIN[ChatWindow\nStreaming Messages]
        DOC_EDITOR[DocEditor\nMarkdown Editor]
        PII_DIFF[PiiDiffViewer\nSide-by-side Diff]
        HW_REPORT[HardwareReport\nSystem Specs]
        MODEL_SEL[ModelSelector\nProvider Dropdown]
        TRIAGE_PROG[TriageProgress\n5-Whys Steps]
    end

    subgraph "State (src/stores/)"
        SESSION[sessionStore\nEphemeral — NOT persisted\nCurrentIssue / Messages\nPiiSpans / WhyLevel]
        SETTINGS_STORE[settingsStore\nPersisted to localStorage\nTheme / ActiveProvider\nPiiPatterns]
        HISTORY_STORE[historyStore\nCached issue list\nSearch results]
    end

    subgraph "IPC Layer (src/lib/)"
        IPC[tauriCommands.ts\nTyped invoke() wrappers\nAll Tauri commands]
        PROMPTS[domainPrompts.ts\n8 Domain System Prompts]
    end

    APP --> DASHBOARD
    APP --> TRIAGE
    APP --> LOG_UPLOAD
    APP --> HISTORY
    APP --> SETTINGS

    TRIAGE --> CHAT_WIN
    TRIAGE --> TRIAGE_PROG
    LOG_UPLOAD --> PII_DIFF
    RCA --> DOC_EDITOR
    POSTMORTEM --> DOC_EDITOR
    SETTINGS --> HW_REPORT
    SETTINGS --> MODEL_SEL

    TRIAGE --> SESSION
    TRIAGE --> SETTINGS_STORE
    HISTORY --> HISTORY_STORE
    SETTINGS --> SETTINGS_STORE

    CHAT_WIN --> IPC
    LOG_UPLOAD --> IPC
    RCA --> IPC
    SETTINGS --> IPC

    IPC --> PROMPTS

    style SESSION fill:#e74c3c,color:#fff
    style SETTINGS_STORE fill:#27ae60,color:#fff
    style IPC fill:#4a90d9,color:#fff
```

---

## Data Architecture

### Database Schema

```mermaid
erDiagram
    issues {
        TEXT id PK
        TEXT title
        TEXT description
        TEXT severity
        TEXT status
        TEXT category
        TEXT source
        TEXT assigned_to
        TEXT tags
        TEXT created_at
        TEXT updated_at
    }
    log_files {
        TEXT id PK
        TEXT issue_id FK
        TEXT file_name
        TEXT content_hash
        TEXT mime_type
        INTEGER size_bytes
        INTEGER redacted
        TEXT created_at
    }
    pii_spans {
        TEXT id PK
        TEXT log_file_id FK
        INTEGER start_offset
        INTEGER end_offset
        TEXT original_value
        TEXT replacement
        TEXT pattern_type
        INTEGER approved
    }
    ai_conversations {
        TEXT id PK
        TEXT issue_id FK
        TEXT provider_name
        TEXT model_name
        TEXT created_at
    }
    ai_messages {
        TEXT id PK
        TEXT conversation_id FK
        TEXT role
        TEXT content
        INTEGER token_count
        TEXT created_at
    }
    resolution_steps {
        TEXT id PK
        TEXT issue_id FK
        INTEGER step_order
        TEXT question
        TEXT answer
        TEXT evidence
        TEXT created_at
    }
    documents {
        TEXT id PK
        TEXT issue_id FK
        TEXT doc_type
        TEXT title
        TEXT content_md
        TEXT created_at
        TEXT updated_at
    }
    audit_log {
        TEXT id PK
        TEXT action
        TEXT entity_type
        TEXT entity_id
        TEXT prev_hash
        TEXT entry_hash
        TEXT details
        TEXT created_at
    }
    credentials {
        TEXT id PK
        TEXT service UNIQUE
        TEXT token_type
        TEXT encrypted_token
        TEXT token_hash
        TEXT expires_at
        TEXT created_at
    }
    integration_config {
        TEXT id PK
        TEXT service UNIQUE
        TEXT base_url
        TEXT username
        TEXT project_name
        TEXT space_key
        INTEGER auto_create
    }
    ai_providers {
        TEXT id PK
        TEXT name UNIQUE
        TEXT provider_type
        TEXT api_url
        TEXT encrypted_api_key
        TEXT model
        TEXT config_json
        INTEGER supports_tool_calling
    }
    issues_fts {
        TEXT rowid FK
        TEXT title
        TEXT description
    }
    shell_commands {
        TEXT id PK
        TEXT command_template
        INTEGER tier
        TEXT description
        TEXT category
    }
    kubeconfig_files {
        TEXT id PK
        TEXT name
        TEXT encrypted_content
        TEXT context
        TEXT cluster_url
        INTEGER is_active
    }
    command_executions {
        TEXT id PK
        TEXT issue_id FK
        TEXT command
        INTEGER tier
        TEXT approval_status
        TEXT kubeconfig_id FK
        INTEGER exit_code
        TEXT stdout
        TEXT stderr
        INTEGER execution_time_ms
        TEXT executed_at
    }
    approval_decisions {
        TEXT id PK
        TEXT command_pattern
        TEXT decision
        TEXT session_id
        TEXT decided_at
        TEXT expires_at
    }
    mcp_servers {
        TEXT id PK
        TEXT name
        TEXT url
        TEXT transport_type
        TEXT auth_type
        TEXT auth_value
        INTEGER enabled
        TEXT discovery_status
        TEXT env_config
    }
    mcp_tools {
        TEXT id PK
        TEXT server_id FK
        TEXT name
        TEXT tool_key
        TEXT description
        TEXT parameters
    }
    mcp_resources {
        TEXT id PK
        TEXT server_id FK
        TEXT uri
        TEXT name
        TEXT description
    }

    issues ||--o{ log_files : "has"
    issues ||--o{ ai_conversations : "has"
    issues ||--o{ resolution_steps : "has"
    issues ||--o{ documents : "has"
    issues ||--o{ command_executions : "has"
    issues ||--|| issues_fts : "indexed by"
    log_files ||--o{ pii_spans : "contains"
    ai_conversations ||--o{ ai_messages : "contains"
    command_executions }o--|| kubeconfig_files : "uses"
    mcp_servers ||--o{ mcp_tools : "exposes"
    mcp_servers ||--o{ mcp_resources : "exposes"
```

### Data Flow — Issue Triage Lifecycle

```mermaid
sequenceDiagram
    participant U as User
    participant FE as React Frontend
    participant IPC as Tauri IPC
    participant BE as Rust Backend
    participant PII as PII Engine
    participant AI as AI Provider
    participant DB as SQLCipher DB

    U->>FE: Create new issue
    FE->>IPC: create_issue(title, severity)
    IPC->>BE: cmd::db::create_issue()
    BE->>DB: INSERT INTO issues
    DB-->>BE: Issue{id, ...}
    BE-->>FE: Issue

    U->>FE: Upload log file
    FE->>IPC: upload_log_file(issue_id, path)
    IPC->>BE: cmd::analysis::upload_log_file()
    BE->>BE: Read file, SHA-256 hash
    BE->>DB: INSERT INTO log_files
    BE->>PII: detect(content)
    PII-->>BE: Vec<PiiSpan>
    BE->>DB: INSERT INTO pii_spans
    BE-->>FE: {log_file, spans}

    U->>FE: Approve redactions
    FE->>IPC: apply_redactions(log_file_id, span_ids)
    IPC->>BE: cmd::analysis::apply_redactions()
    BE->>DB: UPDATE pii_spans SET approved=1
    BE->>BE: Write .redacted file
    BE->>DB: UPDATE log_files SET redacted=1
    BE->>DB: INSERT INTO audit_log (hash-chained)

    U->>FE: Start AI triage
    FE->>IPC: analyze_logs(issue_id, ...)
    IPC->>BE: cmd::ai::analyze_logs()
    BE->>DB: SELECT redacted log content
    BE->>AI: POST /chat/completions (redacted content)
    AI-->>BE: {summary, findings, why1, severity}
    BE->>DB: INSERT ai_messages
    BE-->>FE: AnalysisResult

    loop 5-Whys Iteration
        U->>FE: Ask "Why?" question
        FE->>IPC: chat_message(conversation_id, msg)
        IPC->>BE: cmd::ai::chat_message()
        BE->>DB: SELECT conversation history
        BE->>AI: POST /chat/completions
        AI-->>BE: Response with why level detection
        BE->>DB: INSERT ai_messages
        BE-->>FE: ChatResponse{content, why_level}
        FE->>FE: Auto-advance why level (1→5)
    end

    U->>FE: Generate RCA
    FE->>IPC: generate_rca(issue_id)
    IPC->>BE: cmd::docs::generate_rca()
    BE->>DB: SELECT issue + steps + conversations
    BE->>BE: Build markdown template
    BE->>DB: INSERT INTO documents
    BE-->>FE: Document{content_md}
```

---

## Security Architecture

### Security Layers

```mermaid
graph TB
    subgraph "Layer 1: Network Security"
        CSP[Content Security Policy\nallow-list of external hosts]
        TLS[TLS Enforcement\nreqwest HTTPS only]
        CAP[Tauri Capability ACL\nLeast-privilege permissions]
    end

    subgraph "Layer 2: Data Encryption"
        SQLCIPHER[SQLCipher AES-256\nFull database encryption\nPBKDF2-SHA512, 256k iterations]
        AES_GCM[AES-256-GCM\nCredential token encryption\nUnique nonce per encrypt]
        STRONGHOLD[Tauri Stronghold\nKey derivation + storage\nArgon2 password hashing]
    end

    subgraph "Layer 3: Key Management"
        DB_KEY[.dbkey file\nPer-install random 256-bit key\nMode 0600 — owner only]
        ENC_KEY[.enckey file\nPer-install random 256-bit key\nMode 0600 — owner only]
        ENV_OVERRIDE[TRCAA_DB_KEY / TRCAA_ENCRYPTION_KEY\nOptional env var override]
    end

    subgraph "Layer 4: PII Protection"
        PII_DETECT[12-Pattern PII Detector\nEmail / IP / Phone / SSN\nTokens / Passwords / MAC]
        USER_APPROVE[User Approval Gate\nManual review before AI send]
        AUDIT[Hash-chained Audit Log\nprev_hash → entry_hash\nTamper detection]
    end

    subgraph "Layer 5: Credential Storage"
        TOKEN_HASH[Token Hash Storage\nSHA-256 hash in credentials table]
        TOKEN_ENC[Token Encrypted Storage\nAES-256-GCM ciphertext]
        NO_BROWSER[No Browser Storage\nAPI keys never in localStorage]
    end

    SQLCIPHER --> DB_KEY
    AES_GCM --> ENC_KEY
    DB_KEY --> ENV_OVERRIDE
    ENC_KEY --> ENV_OVERRIDE
    TOKEN_ENC --> AES_GCM
    TOKEN_HASH --> AUDIT

    style SQLCIPHER fill:#c0392b,color:#fff
    style AES_GCM fill:#c0392b,color:#fff
    style AUDIT fill:#e67e22,color:#fff
    style PII_DETECT fill:#e67e22,color:#fff
    style USER_APPROVE fill:#27ae60,color:#fff
```

### Shell Execution Security (v1.0.0+)

**Three-tier safety classification protects against accidental or malicious command execution.**

```mermaid
flowchart TD
    A[AI Agent calls execute_shell_command] --> B[Parse command string]
    B --> C{Contains pipes or command substitution?}
    C -->|Yes| D[Parse into components]
    C -->|No| E[Single command]
    
    D --> F[Classify each component]
    E --> F
    
    F --> G{Highest tier?}
    
    G -->|Tier 1| H[Read-only operations]
    G -->|Tier 2| I[Mutating operations]
    G -->|Tier 3| J[Destructive operations]
    
    H --> K[Execute automatically]
    K --> L[Record to command_executions]
    L --> M[Return output to AI]
    
    I --> N[Show approval modal to user]
    N --> O{User decision?}
    O -->|Allow Once| K
    O -->|Allow for Session| P[Store approval_decision]
    O -->|Deny| Q[Record denial]
    P --> K
    Q --> R[Return error to AI]
    
    J --> S[Always reject]
    S --> Q
    
    L --> T[Audit Log]
    Q --> T
    
    style H fill:#e6f7e6
    style I fill:#fff4e6
    style J fill:#ffe6e6
    style S fill:#c0392b,color:#fff
```

**Tier Classification Rules**:

| Tier | Safety Level | Examples | Action |
|------|--------------|----------|--------|
| Tier 1 | Read-only | `kubectl get`, `cat`, `grep`, `ls`, `pvecm status` | Auto-execute |
| Tier 2 | Mutating | `kubectl apply`, `chmod`, `systemctl restart`, `ssh` | User approval |
| Tier 3 | Destructive | `rm -rf`, `shutdown`, `mkfs`, `dd`, fork bombs | Always deny |

**Escalation Rules**:
- Command with pipe (`|`) or chain (`&&`, `||`, `;`) → highest tier wins
- Command substitution (`` `...` `` or `$(...)`) → escalate Tier 1 to Tier 2
- Single Tier 3 command in chain → entire command becomes Tier 3

**Kubeconfig Encryption**:
- All kubeconfig files encrypted with AES-256-GCM before storage
- Decrypted on-demand for kubectl execution
- Encryption key from `TRCAA_ENCRYPTION_KEY` env var or `.enckey` file

**Audit Trail**:
- All commands logged to `command_executions` table
- Includes: command text, tier, approval status, exit code, stdout, stderr, execution time
- Linked to issue_id for incident context
- Session-based approval decisions stored separately with expiration

---

### Authentication Flow — OAuth2 Integration

```mermaid
sequenceDiagram
    participant U as User
    participant FE as Frontend
    participant BE as Rust Backend
    participant WV as WebView Window
    participant CB as Callback Server\n(warp, port 8765)
    participant EXT as External Service\n(Confluence/ADO)

    U->>FE: Click "Connect" for integration
    FE->>BE: initiate_oauth(service)
    BE->>BE: Generate PKCE code_verifier + code_challenge
    BE->>CB: Start warp server (localhost:8765)
    BE->>WV: Open auth URL in new WebView window
    WV->>EXT: GET /oauth/authorize?code_challenge=...
    EXT-->>WV: Login page
    U->>WV: Enter credentials
    WV->>EXT: POST credentials
    EXT-->>WV: Redirect to localhost:8765/callback?code=xxx
    WV->>CB: GET /callback?code=xxx
    CB->>BE: Signal auth code received
    BE->>EXT: POST /oauth/token (code + code_verifier)
    EXT-->>BE: access_token + refresh_token
    BE->>BE: encrypt_token(access_token)
    BE->>DB: INSERT credentials (encrypted_token, token_hash)
    BE->>DB: INSERT audit_log
    BE-->>FE: OAuth complete
    FE->>FE: Show "Connected" status
```

---

## AI Provider Architecture

### Provider Trait Pattern

```mermaid
classDiagram
    class Provider {
        <<trait>>
        +name() String
        +chat(messages, config) Future~ChatResponse~
        +info() ProviderInfo
    }

    class AnthropicProvider {
        -api_key: String
        -model: String
        +chat(messages, config)
        +name() "anthropic"
    }

    class OpenAiProvider {
        -api_url: String
        -api_key: String
        -model: String
        -api_format: ApiFormat
        +chat(messages, config)
        +name() "openai"
    }

    class OllamaProvider {
        -base_url: String
        -model: String
        +chat(messages, config)
        +name() "ollama"
    }

    class GeminiProvider {
        -api_key: String
        -model: String
        +chat(messages, config)
        +name() "gemini"
    }

    class MistralProvider {
        -api_key: String
        -model: String
        +chat(messages, config)
        +name() "mistral"
    }

    class ProviderFactory {
        +create_provider(config: ProviderConfig) Box~dyn Provider~
    }

    class ProviderConfig {
        +name: String
        +provider_type: String
        +api_url: String
        +api_key: String
        +model: String
        +max_tokens: Option~u32~
        +temperature: Option~f64~
        +custom_endpoint_path: Option~String~
        +custom_auth_header: Option~String~
        +custom_auth_prefix: Option~String~
        +api_format: Option~String~
    }

    Provider <|.. AnthropicProvider
    Provider <|.. OpenAiProvider
    Provider <|.. OllamaProvider
    Provider <|.. GeminiProvider
    Provider <|.. MistralProvider
    ProviderFactory --> Provider : creates
    ProviderFactory --> ProviderConfig : consumes
```

### Tool Calling Flow (Azure DevOps)

```mermaid
sequenceDiagram
    participant U as User
    participant FE as Frontend
    participant BE as Rust Backend
    participant AI as AI Provider
    participant ADO as Azure DevOps API

    U->>FE: Chat message mentioning ADO work item
    FE->>BE: chat_message(conversation_id, msg, provider_config)
    BE->>BE: Inject get_available_tools() into request
    BE->>AI: POST /chat/completions {messages, tools: [add_ado_comment]}
    AI-->>BE: {tool_calls: [{function: "add_ado_comment", args: {work_item_id, comment_text}}]}
    BE->>BE: Parse tool_calls from response
    BE->>BE: Validate tool name matches registered tools
    BE->>ADO: PATCH /wit/workitems/{id}?api-version=7.0 (add comment)
    ADO-->>BE: 200 OK
    BE->>BE: Format tool result message
    BE->>AI: POST /chat/completions {messages, tool_result}
    AI-->>BE: Final response to user
    BE->>DB: INSERT ai_messages (tool call + result)
    BE-->>FE: ChatResponse{content}
```

---

## Integration Architecture

```mermaid
graph LR
    subgraph "Integration Layer (integrations/)"
        AUTH[auth.rs\nToken Encryption\nOAuth + PKCE\nCookie Extraction]

        subgraph "Confluence"
            CF[confluence.rs\nPublish Documents\nSpace Management]
            CF_SEARCH[confluence_search.rs\nContent Search\nPersistent WebView]
        end

        subgraph "ServiceNow"
            SN[servicenow.rs\nCreate Incidents\nUpdate Records]
            SN_SEARCH[servicenow_search.rs\nIncident Search\nKnowledge Base]
        end

        subgraph "Azure DevOps"
            ADO[azuredevops.rs\nWork Items CRUD\nComments (AI tool)]
            ADO_SEARCH[azuredevops_search.rs\nWork Item Search\nPersistent WebView]
        end

        subgraph "Auth Infrastructure"
            WV_AUTH[webview_auth.rs\nOAuth WebView\nLogin Flow]
            CB_SERVER[callback_server.rs\nwarp HTTP Server\nlocalhost:8765]
            NAT_COOKIES[native_cookies*.rs\nPlatform Cookie\nExtraction]
        end
    end

    subgraph "External Services"
        CF_EXT[Atlassian Confluence\nhttps://*.atlassian.net]
        SN_EXT[ServiceNow\nhttps://*.service-now.com]
        ADO_EXT[Azure DevOps\nhttps://dev.azure.com]
    end

    AUTH --> CF
    AUTH --> SN
    AUTH --> ADO
    WV_AUTH --> CB_SERVER
    WV_AUTH --> NAT_COOKIES

    CF --> CF_EXT
    CF_SEARCH --> CF_EXT
    SN --> SN_EXT
    SN_SEARCH --> SN_EXT
    ADO --> ADO_EXT
    ADO_SEARCH --> ADO_EXT

    style AUTH fill:#c0392b,color:#fff
```

---

## Deployment Architecture

### CI/CD Pipeline

```mermaid
graph TB
    subgraph "Source Control"
        GOGS[Gogs / Gitea\ngogs.trcaa.com\nSarman Repository]
    end

    subgraph "CI/CD Triggers"
        PR_TRIGGER[PR Opened/Updated\ntest.yml workflow]
        MASTER_TRIGGER[Push to master\nauto-tag.yml workflow]
        DOCKER_TRIGGER[.docker/ changes\nbuild-images.yml workflow]
    end

    subgraph "Test Runner — amd64-docker-runner"
        RUSTFMT[1. rustfmt\nFormat Check]
        CLIPPY[2. clippy\n-D warnings]
        CARGO_TEST[3. cargo test\n64 Rust tests]
        TSC[4. tsc --noEmit\nType Check]
        VITEST[5. vitest run\n13 JS tests]
    end

    subgraph "Release Builders (Parallel)"
        AMD64[linux/amd64\nDocker: trcaa-linux-amd64\n.deb .rpm .AppImage]
        WINDOWS[windows/amd64\nDocker: trcaa-windows-cross\n.exe .msi]
        ARM64[linux/arm64\narm64 native runner\n.deb .rpm .AppImage]
        MACOS[macOS arm64\nnative macOS runner\n.app .dmg]
    end

    subgraph "Artifact Storage"
        RELEASE[Gitea Release\nv0.x.x tags\nAll platform assets]
        REGISTRY[Gitea Container Registry\n172.0.0.29:3000\nCI Docker images]
    end

    GOGS --> PR_TRIGGER
    GOGS --> MASTER_TRIGGER
    GOGS --> DOCKER_TRIGGER

    PR_TRIGGER --> RUSTFMT
    RUSTFMT --> CLIPPY
    CLIPPY --> CARGO_TEST
    CARGO_TEST --> TSC
    TSC --> VITEST

    MASTER_TRIGGER --> AMD64
    MASTER_TRIGGER --> WINDOWS
    MASTER_TRIGGER --> ARM64
    MASTER_TRIGGER --> MACOS

    AMD64 --> RELEASE
    WINDOWS --> RELEASE
    ARM64 --> RELEASE
    MACOS --> RELEASE

    DOCKER_TRIGGER --> REGISTRY

    style VITEST fill:#27ae60,color:#fff
    style RELEASE fill:#4a90d9,color:#fff
```

### Runtime Architecture (per Platform)

```mermaid
graph TB
    subgraph "macOS Runtime"
        MAC_PROC[trcaa process\nMach-O arm64 binary]
        WEBKIT[WKWebView\nSafari WebKit engine]
        MAC_DATA[~/Library/Application Support/trcaa/\n.dbkey mode 0600\n.enckey mode 0600\ntrcaa.db SQLCipher]
        MAC_KUBECTL[Bundled kubectl v1.30.0\narm64 binary]
        MAC_BUNDLE[Troubleshooting and RCA Assistant.app\n/Applications/]
    end

    subgraph "Linux Runtime"
        LINUX_PROC[trcaa process\nELF amd64/arm64]
        WEBKIT2[WebKitGTK WebView\nwebkit2gtk4.1]
        LINUX_DATA[~/.local/share/trcaa/\n.dbkey .enckey\ntrcaa.db]
        LINUX_KUBECTL[Bundled kubectl v1.30.0\namd64/arm64 binary]
        LINUX_PKG[.deb / .rpm / .AppImage]
    end

    subgraph "Windows Runtime"
        WIN_PROC[trcaa.exe\nPE amd64]
        WEBVIEW2[Microsoft WebView2\nChromium-based]
        WIN_DATA[%APPDATA%\trcaa\\\n.dbkey .enckey\ntrcaa.db]
        WIN_KUBECTL[Bundled kubectl.exe v1.30.0\namd64 binary]
        WIN_PKG[NSIS .exe / .msi]
    end

    MAC_BUNDLE --> MAC_PROC
    MAC_PROC --> WEBKIT
    MAC_PROC --> MAC_DATA
    MAC_PROC --> MAC_KUBECTL

    LINUX_PKG --> LINUX_PROC
    LINUX_PROC --> WEBKIT2
    LINUX_PROC --> LINUX_DATA
    LINUX_PROC --> LINUX_KUBECTL

    WIN_PKG --> WIN_PROC
    WIN_PROC --> WEBVIEW2
    WIN_PROC --> WIN_DATA
    WIN_PROC --> WIN_KUBECTL
```

---

## Key Data Flows

### PII Detection and Redaction

```mermaid
flowchart TD
    A[User uploads log file] --> B[Read file contents\nmax 50MB]
    B --> C[Compute SHA-256 hash]
    C --> D[Store metadata in log_files table]
    D --> E[Run PII Detection Engine]

    subgraph "PII Engine"
        E --> F{12 Pattern Detectors}
        F --> G[Email Regex]
        F --> H[IPv4/IPv6 Regex]
        F --> I[Bearer Token Regex]
        F --> J[Password Regex]
        F --> K[SSN / Phone / CC]
        F --> L[MAC / Hostname]
        G & H & I & J & K & L --> M[Collect all spans]
        M --> N[Sort by start offset]
        N --> O[Remove overlaps\nlongest span wins]
    end

    O --> P[Store pii_spans in DB\nwith UUID per span]
    P --> Q[Return spans to UI]
    Q --> R[PiiDiffViewer\nSide-by-side diff]
    R --> S{User reviews}
    S -->|Approve| T[apply_redactions\nMark spans approved]
    S -->|Dismiss| U[Remove from approved set]
    T --> V[Write .redacted log file\nreplace spans with placeholders]
    V --> W[Update log_files.redacted = 1]
    W --> X[Append to audit_log\nhash-chained entry]
    X --> Y[Log now safe for AI send]
```

### Encryption Key Lifecycle

```mermaid
flowchart TD
    A[App Launch] --> B{TRCAA_DB_KEY env var set?}
    B -->|Yes| C[Use env var key]
    B -->|No| D{Release build?}
    D -->|Debug| E[Use hardcoded dev key]
    D -->|Release| F{.dbkey file exists?}
    F -->|Yes| G[Load key from .dbkey]
    F -->|No| H[Generate 32 random bytes\nhex-encode → 64 char key]
    H --> I[Write to .dbkey\nmode 0600]
    I --> J[Use generated key]

    G --> K{Open database}
    C --> K
    E --> K
    J --> K

    K --> L{SQLCipher decrypt success?}
    L -->|Yes| M[Run migrations\nDatabase ready]
    L -->|No| N{File is plain SQLite?}
    N -->|Yes| O[migrate_plain_to_encrypted\nCreate .db.plain-backup\nATTACH + sqlcipher_export]
    N -->|No| P[Fatal error\nDatabase corrupt]
    O --> M

    style H fill:#27ae60,color:#fff
    style O fill:#e67e22,color:#fff
    style P fill:#c0392b,color:#fff
```

---

## Architecture Decision Records

See the [adrs/](./adrs/) directory for all Architecture Decision Records.

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](./adrs/ADR-001-tauri-desktop-framework.md) | Tauri as Desktop Framework | Accepted |
| [ADR-002](./adrs/ADR-002-sqlcipher-encrypted-database.md) | SQLCipher for Encrypted Storage | Accepted |
| [ADR-003](./adrs/ADR-003-provider-trait-pattern.md) | Provider Trait Pattern for AI Backends | Accepted |
| [ADR-004](./adrs/ADR-004-pii-regex-aho-corasick.md) | Regex + Aho-Corasick for PII Detection | Accepted |
| [ADR-005](./adrs/ADR-005-auto-generate-encryption-keys.md) | Auto-generate Encryption Keys at Runtime | Accepted |
| [ADR-006](./adrs/ADR-006-zustand-state-management.md) | Zustand for Frontend State Management | Accepted |
| [ADR-007](./adrs/ADR-007-three-tier-shell-safety.md) | Three-Tier Shell Command Safety Classification | Accepted |
| [ADR-008](./adrs/ADR-008-mcp-protocol-integration.md) | Model Context Protocol for External Tools | Accepted |
| [ADR-009](./adrs/ADR-009-bundled-kubectl-binary.md) | Bundle kubectl Binary for Cross-Platform Consistency | Accepted |
