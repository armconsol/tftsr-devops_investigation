# ADR-003: Provider Trait Pattern for AI Backends

**Status**: Accepted
**Date**: 2025-Q3
**Deciders**: sarman

---

## Context

The application must support multiple AI providers (OpenAI, Anthropic, Google Gemini, Mistral, Ollama) with different API formats, authentication methods, and response structures. Provider selection must be runtime-configurable by the user without recompiling.

Additionally, enterprise environments may need custom AI endpoints (e.g., MSI GenAI gateway at `genai-service.commandcentral.com`) that speak OpenAI-compatible APIs with custom auth headers.

---

## Decision

Use a **Rust trait object** (`Box<dyn Provider>`) with a **factory function** (`create_provider(config: ProviderConfig)`) that dispatches to concrete implementations at runtime.

---

## Rationale

**The `Provider` trait:**
```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, messages: Vec<Message>, config: &ProviderConfig) -> Result<ChatResponse>;
    fn info(&self) -> ProviderInfo;
}
```

**Why trait objects over generics:**
- Provider type is not known at compile time (user configures at runtime)
- `Box<dyn Provider>` allows storing different providers in the same `AppState`
- `#[async_trait]` enables async methods on trait objects (required for `reqwest`)

**`ProviderConfig` design:**
The config struct uses `Option<String>` fields for provider-specific settings:
```rust
pub struct ProviderConfig {
    pub custom_endpoint_path: Option<String>,
    pub custom_auth_header: Option<String>,
    pub custom_auth_prefix: Option<String>,
    pub api_format: Option<String>,   // "openai" | "custom_rest"
}
```
This allows a single `OpenAiProvider` implementation to handle both standard OpenAI and arbitrary OpenAI-compatible endpoints — the user configures the auth header name and prefix to match their gateway.

---

## Adding a New Provider

1. Create `src-tauri/src/ai/<provider>.rs` implementing the `Provider` trait
2. Add a match arm in `create_provider()` in `provider.rs`
3. Register the provider type string in `ProviderConfig`
4. Add UI in `src/pages/Settings/AIProviders.tsx`

No changes to command handlers or IPC layer required.

---

## Consequences

**Positive:**
- New providers require zero changes outside `ai/`
- `ProviderConfig` is stored in the database — provider can be changed without app restart
- `test_provider_connection()` command works uniformly across all providers
- `list_providers()` returns capabilities dynamically (supports streaming, tool calling, etc.)

**Negative:**
- `dyn Provider` has a small vtable dispatch overhead (negligible for HTTP-bound operations)
- Each provider implementation must handle its own error types and response parsing
- Testing requires mocking at the `reqwest` level (via `mockito`)
