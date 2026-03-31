# AI Providers

TFTSR supports 5 AI providers, selectable per-session. API keys are stored in the Stronghold encrypted vault.

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
| Auth | API key as `?key=` query parameter |
| Max tokens | 4096 |

**Models:** `gemini-2.0-flash`, `gemini-2.0-pro`, `gemini-1.5-pro`, `gemini-1.5-flash`

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

## Adding a New Provider

1. Create `src-tauri/src/ai/{name}.rs` implementing the `Provider` trait
2. Add a match arm in `ai/provider.rs::create_provider()`
3. Add the model list in `commands/ai.rs::list_providers()`
4. Add the TypeScript type in `src/lib/tauriCommands.ts`
5. Add a UI entry in `src/pages/Settings/AIProviders.tsx`
