# AI Providers

TRCAA supports 6+ AI providers, including custom providers with flexible authentication and API formats. API keys are stored encrypted with AES-256-GCM.

## Provider Factory

`ai/provider.rs::create_provider(config)` dispatches on `config.name` to the matching implementation. Adding a provider requires implementing the `Provider` trait and adding a match arm.

```rust
pub trait Provider {
    async fn chat(&self, messages: Vec<Message>, config: &ProviderConfig) -> Result<ChatResponse>;
    fn name(&self) -> &str;
}
```

---

## Supported Providers

### 1. OpenAI-Compatible

Covers: OpenAI, Azure OpenAI, LM Studio, vLLM, **LiteLLM (AWS Bedrock)**, and any OpenAI-API-compatible endpoint.

| Field | Value |
|-------|-------|
| `config.name` | `"openai"` |
| Default URL | `https://api.openai.com/v1/chat/completions` |
| Auth | `Authorization: Bearer <api_key>` |
| Max tokens | 4096 |

**Models:** `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo`

**Custom endpoint:** Set `config.base_url` to any OpenAI-compatible API:
- LM Studio: `http://localhost:1234/v1`
- **LiteLLM (AWS Bedrock):** `http://localhost:8000/v1` — See [LiteLLM + Bedrock Setup](LiteLLM-Bedrock-Setup) for full configuration guide

---

### 2. Anthropic Claude

| Field | Value |
|-------|-------|
| `config.name` | `"anthropic"` |
| URL | `https://api.anthropic.com/v1/messages` |
| Auth | `x-api-key: <api_key>` + `anthropic-version: 2023-06-01` |
| Max tokens | 4096 |

**Models:** `claude-sonnet-4-20250514`, `claude-haiku-4-20250414`, `claude-3-5-sonnet-20241022`

---

### 3. Google Gemini

| Field | Value |
|-------|-------|
| `config.name` | `"gemini"` |
| URL | `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent` |
| Auth | `x-goog-api-key: <api_key>` header |
| Max tokens | 4096 |

**Models:** `gemini-2.0-flash`, `gemini-2.0-pro`, `gemini-1.5-pro`, `gemini-1.5-flash`

---

## Transport Security Notes

- Provider clients use TLS certificate verification via `reqwest`
- Provider calls are configured with explicit request timeouts to avoid indefinite hangs
- Credentials are sent in headers (not URL query strings)

---

### 4. Mistral AI

| Field | Value |
|-------|-------|
| `config.name` | `"mistral"` |
| Default URL | `https://api.mistral.ai/v1/chat/completions` |
| Auth | `Authorization: Bearer <api_key>` |
| Max tokens | 4096 |

**Models:** `mistral-large-latest`, `mistral-medium-latest`, `mistral-small-latest`, `open-mistral-nemo`

Uses OpenAI-compatible request/response format.

---

### 5. Ollama (Local / Offline)

| Field | Value |
|-------|-------|
| `config.name` | `"ollama"` |
| Default URL | `http://localhost:11434/api/chat` |
| Auth | None |
| Max tokens | No limit enforced |

**Models:** Any model pulled locally — `llama3.1`, `llama3`, `mistral`, `codellama`, `phi3`, etc.

Fully offline. Responses include `eval_count` / `prompt_eval_count` token stats.

**Custom URL:** Change the Ollama URL in Settings → AI Providers → Ollama (stored in `settingsStore.ollama_url`).

---

## Domain System Prompts

Each triage conversation is pre-loaded with a domain-specific expert system prompt from `src/lib/domainPrompts.ts`.

| Domain | Key areas covered |
|--------|------------------|
| **Linux** | systemd, filesystem, memory, networking, kernel, performance |
| **Windows** | Event Viewer, Active Directory, IIS, Group Policy, clustering |
| **Network** | DNS, firewalls, load balancers, BGP/OSPF, Layer 2, VPN |
| **Kubernetes** | Pod failures, service mesh, ingress, storage, Helm |
| **Databases** | Connection pools, slow queries, indexes, replication, MongoDB/Redis |
| **Virtualization** | vMotion, storage (VMFS), HA, snapshots; KVM/QEMU |
| **Hardware** | RAID, SMART data, ECC memory errors, thermal, BIOS/firmware |
| **Observability** | Prometheus/Grafana, ELK/OpenSearch, tracing, SLO/SLI burn rates |

The domain prompt is injected as the first `system` role message in every new conversation.

---

## 6. Custom Provider (Custom REST & Others)

**Status:** ✅ **Implemented** (v0.2.6)

Custom providers allow integration with non-OpenAI-compatible APIs. The application supports two API formats:

### Format: OpenAI Compatible (Default)

Standard OpenAI `/chat/completions` endpoint with Bearer authentication.

| Field | Default Value |
|-------|--------------|
| `api_format` | `"openai"` |
| `custom_endpoint_path` | `/chat/completions` |
| `custom_auth_header` | `Authorization` |
| `custom_auth_prefix` | `Bearer ` |

**Use cases:**
- Self-hosted LLMs with OpenAI-compatible APIs
- Custom proxy services
- Enterprise gateways

---

### Format: Custom REST

**Enterprise AI Gateway** — For AI platforms that use a non-OpenAI request/response format with centralized cost tracking and model access.

| Field | Value |
|-------|-------|
| `config.provider_type` | `"custom"` |
| `config.api_format` | `"custom_rest"` |
| API URL | Your gateway's base URL |
| Auth Header | Your gateway's auth header name |
| Auth Prefix | `` (empty if no prefix needed) |
| Endpoint Path | `` (empty if URL already includes full path) |

**Request Format:**
```json
{
  "model": "model-name",
  "prompt": "User's latest message",
  "system": "Optional system prompt",
  "sessionId": "uuid-for-conversation-continuity",
  "userId": "user@example.com"
}
```

**Response Format:**
```json
{
  "status": true,
  "sessionId": "uuid",
  "msg": "AI response text",
  "initialPrompt": false
}
```

**Key Differences from OpenAI:**
- **Single prompt** instead of message array (server manages history via `sessionId`)
- **Response in `msg` field** instead of `choices[0].message.content`
- **Session-based** conversation continuity (no need to resend history)
- **Cost tracking** via `userId` field (optional)

**Configuration (Settings → AI Providers → Add Provider):**
```
Name:             Custom REST Gateway
Type:             Custom
API Format:       Custom REST
API URL:          https://your-gateway/api/v2/chat
Model:            your-model-name
API Key:          (your API key)
User ID:          user@example.com (optional, for cost tracking)
Endpoint Path:    (leave empty if URL includes full path)
Auth Header:      x-custom-api-key
Auth Prefix:      (leave empty if no prefix)
```

**Troubleshooting:**

| Error | Cause | Solution |
|-------|-------|----------|
| 403 Forbidden | Invalid API key or insufficient permissions | Verify key in your gateway portal, check model access |
| Missing `userId` field | Configuration not saved | Ensure UI shows User ID field when `api_format=custom_rest` |
| No conversation history | `sessionId` not persisted | Session ID stored in `ProviderConfig.session_id` — currently per-provider, not per-conversation |

**Implementation Details:**
- Backend: `src-tauri/src/ai/openai.rs::chat_custom_rest()`
- Schema: `src-tauri/src/state.rs::ProviderConfig` (added `user_id`, `api_format`, custom auth fields)
- Frontend: `src/pages/Settings/AIProviders.tsx` (conditional UI for Custom REST + model dropdown)

---

## Custom Provider Configuration Fields

All providers support the following optional configuration fields (v0.2.6+):

| Field | Type | Purpose | Default |
|-------|------|---------|---------|
| `custom_endpoint_path` | `Option<String>` | Override endpoint path | `/chat/completions` |
| `custom_auth_header` | `Option<String>` | Custom auth header name | `Authorization` |
| `custom_auth_prefix` | `Option<String>` | Prefix before API key | `Bearer ` |
| `api_format` | `Option<String>` | API format (`openai` or `custom_rest`) | `openai` |
| `session_id` | `Option<String>` | Session ID for stateful APIs | None |
| `user_id` | `Option<String>` | User ID for cost tracking (Custom REST gateways) | None |

**Backward Compatibility:**
All fields are optional and default to OpenAI-compatible behavior. Existing provider configurations are unaffected.

---

## Adding a New Provider

1. Create `src-tauri/src/ai/{name}.rs` implementing the `Provider` trait
2. Add a match arm in `ai/provider.rs::create_provider()`
3. Add the model list in `commands/ai.rs::list_providers()`
4. Add the TypeScript type in `src/lib/tauriCommands.ts`
5. Add a UI entry in `src/pages/Settings/AIProviders.tsx`
