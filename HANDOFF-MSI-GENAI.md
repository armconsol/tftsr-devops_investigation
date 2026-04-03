# MSI GenAI Custom Provider Integration - Handoff Document

**Date**: 2026-04-03
**Status**: In Progress - Backend schema updated, frontend and provider logic pending

---

## Context

User needs to integrate MSI GenAI API (https://genai-service.stage.commandcentral.com/app-gateway/api/v2/chat) into the application's AI Providers system.

**Problem**: The existing "Custom" provider type assumes OpenAI-compatible APIs (expects `/chat/completions` endpoint, OpenAI request/response format, `Authorization: Bearer` header). MSI GenAI has a completely different API contract:

| Aspect | OpenAI Format | MSI GenAI Format |
|--------|---------------|------------------|
| **Endpoint** | `/chat/completions` | `/api/v2/chat` (no suffix) |
| **Request** | `{"messages": [...], "model": "..."}` | `{"prompt": "...", "model": "...", "sessionId": "..."}` |
| **Response** | `{"choices": [{"message": {"content": "..."}}]}` | `{"msg": "...", "sessionId": "..."}` |
| **Auth Header** | `Authorization: Bearer <token>` | `x-msi-genai-api-key: <token>` |
| **History** | Client sends full message array | Server-side via `sessionId` |

---

## Work Completed

### 1. Updated `src-tauri/src/state.rs` - ProviderConfig Schema

Added optional fields to support custom API formats without breaking existing OpenAI-compatible providers:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(default)]
    pub provider_type: String,
    pub api_url: String,
    pub api_key: String,
    pub model: String,

    // NEW FIELDS:
    /// Optional: Custom endpoint path (e.g., "" for no path, "/v1/chat" for custom path)
    /// If None, defaults to "/chat/completions" for OpenAI compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_endpoint_path: Option<String>,

    /// Optional: Custom auth header name (e.g., "x-msi-genai-api-key")
    /// If None, defaults to "Authorization"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_auth_header: Option<String>,

    /// Optional: Custom auth value prefix (e.g., "" for no prefix, "Bearer " for OpenAI)
    /// If None, defaults to "Bearer "
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_auth_prefix: Option<String>,

    /// Optional: API format ("openai" or "msi_genai")
    /// If None, defaults to "openai"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_format: Option<String>,

    /// Optional: Session ID for stateful APIs like MSI GenAI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}
```

**Design philosophy**: Existing providers remain unchanged (all fields default to OpenAI-compatible behavior). Only when `api_format` is set to `"msi_genai"` do the custom fields take effect.

---

## Work Remaining

### 2. Update `src-tauri/src/ai/openai.rs` - Support Custom Formats

The `OpenAiProvider::chat()` method needs to conditionally handle MSI GenAI format:

**Changes needed**:
- Check `config.api_format` — if `Some("msi_genai")`, use MSI GenAI request/response logic
- Use `config.custom_endpoint_path.unwrap_or("/chat/completions")` for endpoint
- Use `config.custom_auth_header.unwrap_or("Authorization")` for header name
- Use `config.custom_auth_prefix.unwrap_or("Bearer ")` for auth prefix

**MSI GenAI request format**:
```json
{
    "model": "VertexGemini",
    "prompt": "<last user message>",
    "system": "<optional system message>",
    "sessionId": "<uuid or null for first message>",
    "userId": "user@motorolasolutions.com"
}
```

**MSI GenAI response format**:
```json
{
    "status": true,
    "sessionId": "uuid",
    "msg": "AI response text",
    "initialPrompt": true/false
}
```

**Implementation notes**:
- For MSI GenAI, convert `Vec<Message>` to a single `prompt` (concatenate or use last user message)
- Extract system message from messages array if present (role == "system")
- Store returned `sessionId` back to `config.session_id` for subsequent requests
- Extract response content from `json["msg"]` instead of `json["choices"][0]["message"]["content"]`

### 3. Update `src/lib/tauriCommands.ts` - TypeScript Types

Add new optional fields to `ProviderConfig` interface:

```typescript
export interface ProviderConfig {
  provider_type?: string;
  max_tokens?: number;
  temperature?: number;
  name: string;
  api_url: string;
  api_key: string;
  model: string;

  // NEW FIELDS:
  custom_endpoint_path?: string;
  custom_auth_header?: string;
  custom_auth_prefix?: string;
  api_format?: string;
  session_id?: string;
}
```

### 4. Update `src/pages/Settings/AIProviders.tsx` - UI Fields

**When `provider_type === "custom"`, show additional form fields**:

```tsx
{form.provider_type === "custom" && (
  <>
    <div className="space-y-2">
      <Label>API Format</Label>
      <Select
        value={form.api_format ?? "openai"}
        onValueChange={(v) => {
          const format = v;
          const defaults = format === "msi_genai"
            ? {
                custom_endpoint_path: "",
                custom_auth_header: "x-msi-genai-api-key",
                custom_auth_prefix: "",
              }
            : {
                custom_endpoint_path: "/chat/completions",
                custom_auth_header: "Authorization",
                custom_auth_prefix: "Bearer ",
              };
          setForm({ ...form, api_format: format, ...defaults });
        }}
      >
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="openai">OpenAI Compatible</SelectItem>
          <SelectItem value="msi_genai">MSI GenAI</SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div className="grid grid-cols-2 gap-4">
      <div className="space-y-2">
        <Label>Endpoint Path</Label>
        <Input
          value={form.custom_endpoint_path ?? ""}
          onChange={(e) => setForm({ ...form, custom_endpoint_path: e.target.value })}
          placeholder="/chat/completions"
        />
      </div>
      <div className="space-y-2">
        <Label>Auth Header Name</Label>
        <Input
          value={form.custom_auth_header ?? ""}
          onChange={(e) => setForm({ ...form, custom_auth_header: e.target.value })}
          placeholder="Authorization"
        />
      </div>
    </div>

    <div className="space-y-2">
      <Label>Auth Prefix</Label>
      <Input
        value={form.custom_auth_prefix ?? ""}
        onChange={(e) => setForm({ ...form, custom_auth_prefix: e.target.value })}
        placeholder="Bearer "
      />
      <p className="text-xs text-muted-foreground">
        Prefix added before API key (e.g., "Bearer " for OpenAI, "" for MSI GenAI)
      </p>
    </div>
  </>
)}
```

**Update `emptyProvider` initial state**:
```typescript
const emptyProvider: ProviderConfig = {
  name: "",
  provider_type: "openai",
  api_url: "",
  api_key: "",
  model: "",
  custom_endpoint_path: undefined,
  custom_auth_header: undefined,
  custom_auth_prefix: undefined,
  api_format: undefined,
  session_id: undefined,
};
```

---

## Testing Configuration

**For MSI GenAI**:
- **Type**: Custom
- **API Format**: MSI GenAI
- **API URL**: `https://genai-service.stage.commandcentral.com/app-gateway`
- **Model**: `VertexGemini` (or `Claude-Sonnet-4`, `ChatGPT4o`)
- **API Key**: (user's MSI GenAI API key from portal)
- **Endpoint Path**: `` (empty - URL already includes `/api/v2/chat`)
- **Auth Header**: `x-msi-genai-api-key`
- **Auth Prefix**: `` (empty - no "Bearer " prefix)

**Test command flow**:
1. Create provider with above settings
2. Test connection (should receive AI response)
3. Verify `sessionId` is returned and stored
4. Send second message (should reuse `sessionId` for conversation history)

---

## Known Issues from User's Original Error

User initially tried:
- **API URL**: `https://genai-service.stage.commandcentral.com/app-gateway/api/v2/chat`
- **Type**: Custom (no format specified)

**Result**: `Cannot POST /api/v2/chat/chat/completions` (404)

**Root cause**: OpenAI provider appends `/chat/completions` to base URL. With the new `custom_endpoint_path` field, this is now configurable.

---

## Integration with Existing Session Management

MSI GenAI uses server-side session management. Current triage flow sends full message history on every request (OpenAI style). For MSI GenAI:

- **First message**: Send `sessionId: null` or omit field
- **Store response**: Save `response.sessionId` to `config.session_id`
- **Subsequent messages**: Include `sessionId` in requests (server maintains history)

Consider storing `session_id` per conversation in the database (link to `ai_conversations.id`) rather than globally in `ProviderConfig`.

---

## Commit Strategy

**Current git state**:
- Modified by other session: `src-tauri/src/integrations/*.rs` (ADO/Confluence/ServiceNow work)
- Modified by me: `src-tauri/src/state.rs` (MSI GenAI schema)
- Untracked: `GenAI API User Guide.md`

**Recommended approach**:
1. **Other session commits first**: Commit integration changes to main
2. **Then complete MSI GenAI work**: Finish items 2-4 above, test, commit separately

**Alternative**: Create feature branch `feature/msi-genai-custom-provider`, cherry-pick only MSI GenAI changes, complete work there, merge when ready.

---

## Reference: MSI GenAI API Spec

**Documentation**: `GenAI API User Guide.md` (in project root)

**Key endpoints**:
- `POST /api/v2/chat` - Send prompt, get response
- `POST /api/v2/upload/<SESSION-ID>` - Upload files (requires session)
- `GET /api/v2/getSessionMessages/<SESSION-ID>` - Retrieve history
- `DELETE /api/v2/entry/<MSG-ID>` - Delete message

**Available models** (from guide):
- `Claude-Sonnet-4` (Public)
- `ChatGPT4o` (Public)
- `VertexGemini` (Private) - Gemini 2.0 Flash
- `ChatGPT-5_2-Chat` (Public)
- Many others (see guide section 4.1)

**Rate limits**: $50/user/month (enforced server-side)

---

## Questions for User

1. Should `session_id` be stored globally in `ProviderConfig` or per-conversation in DB?
2. Do we need to support file uploads via `/api/v2/upload/<SESSION-ID>`?
3. Should we expose model config options (temperature, max_tokens) for MSI GenAI?

---

## Contact

This handoff doc was generated for the other Claude Code session working on integration files. Once that work is committed, this MSI GenAI work can be completed as a separate commit or feature branch.
