# ADR-006: Zustand for Frontend State Management

**Status**: Accepted
**Date**: 2025-Q3
**Deciders**: sarman

---

## Context

The React frontend manages three distinct categories of state:
1. **Ephemeral session state**: Current issue, AI chat messages, PII spans, 5-whys progress — exists for the duration of one triage session, should not survive page reload
2. **Persisted settings**: Theme, active AI provider, PII pattern toggles — should survive app restart, stored locally
3. **Cached server data**: Issue history, search results — loaded from DB on demand, invalidated on changes

---

## Decision

Use **Zustand** for all three state categories, with selective persistence via `localStorage` for settings only.

---

## Rationale

**Alternatives considered:**

| Option | Pros | Cons |
|--------|------|------|
| **Zustand** (chosen) | Minimal boilerplate, built-in persist middleware, TypeScript-first | Smaller ecosystem than Redux |
| Redux Toolkit | Battle-tested, DevTools support | Verbose boilerplate for simple state |
| React Context | No dependency | Performance issues with frequent updates (chat messages) |
| Jotai | Atomic state, minimal | Less familiar pattern |
| TanStack Query | Excellent for async server state | Overkill for Tauri IPC (not HTTP) |

**Store architecture decisions:**

**`sessionStore`** — NOT persisted:
- Chat messages accumulate quickly; persisting would bloat localStorage
- Session is per-issue; loading a different issue should reset all session state
- `reset()` method called on navigation away from triage

**`settingsStore`** — Persisted to localStorage as `"tftsr-settings"`:
- Theme, active provider, PII pattern toggles — user preference, should survive restart
- AI providers themselves are NOT persisted here — only `active_provider` string
- Actual `ProviderConfig` (with encrypted API keys) lives in the backend DB, loaded via `load_ai_providers()`

**`historyStore`** — NOT persisted (server-cache pattern):
- Always loaded fresh from DB on History page mount
- Search results replaced on each query
- No stale-data risk

---

## Persistence Details

The settings store persists to localStorage:
```typescript
persist(
  (set, get) => ({ ...storeImpl }),
  {
    name: 'tftsr-settings',
    partialize: (state) => ({
      theme: state.theme,
      active_provider: state.active_provider,
      pii_enabled_patterns: state.pii_enabled_patterns,
      // NOTE: ai_providers excluded — stored in encrypted backend DB
    })
  }
)
```

**Why localStorage and not a Tauri store plugin:**
- Settings are non-sensitive (theme, provider name, pattern toggles)
- `tauri-plugin-store` would add IPC overhead for every settings read
- localStorage survives across WebView reloads without async overhead

---

## Consequences

**Positive:**
- Minimal boilerplate — stores are ~50 LOC each
- `zustand/middleware/persist` handles localStorage serialization
- Subscribing to partial state prevents unnecessary re-renders
- No Provider wrapping required — stores accessed via hooks anywhere

**Negative:**
- No Redux DevTools integration (Zustand has its own devtools but less mature)
- localStorage persistence means settings are WebView-profile-scoped (fine for single-user app)
- Manual cache invalidation in `historyStore` after issue create/delete
