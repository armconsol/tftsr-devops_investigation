# ADR-001: Tauri as Desktop Framework

**Status**: Accepted
**Date**: 2025-Q3
**Deciders**: sarman

---

## Context

A cross-platform desktop application is required for IT engineers who need:
- Fully offline operation (local AI via Ollama)
- Encrypted local data storage (sensitive incident details)
- Access to local filesystem (log files)
- No telemetry or cloud dependency for core functionality
- Distribution on Linux, macOS, and Windows

The main alternatives considered were **Electron**, **Flutter**, **Qt**, and a pure **web app**.

---

## Decision

Use **Tauri 2** with a **Rust backend** and **React/TypeScript frontend**.

---

## Rationale

| Criterion | Tauri 2 | Electron | Flutter | Web App |
|-----------|---------|----------|---------|---------|
| Binary size | ~8 MB | ~120+ MB | ~40 MB | N/A |
| Memory footprint | ~50 MB | ~200+ MB | ~100 MB | N/A |
| OS WebView | Yes (native) | No (bundled Chromium) | No | N/A |
| Rust backend | Yes (native perf) | No (Node.js) | No (Dart) | No |
| Filesystem access | Scoped ACL | Unrestricted by default | Limited | CORS-limited |
| Offline-first | Yes | Yes | Yes | No |
| SQLCipher integration | Via rusqlite | Via better-sqlite3 | Via plugin | No |
| Existing team skills | Rust + React | Node.js + React | Dart | TypeScript |

**Tauri's advantages for this use case:**
1. **Security model**: Capability-based ACL prevents frontend from making arbitrary system calls. The frontend can only call explicitly-declared commands.
2. **Performance**: Rust backend handles CPU-intensive work (PII regex scanning, PDF generation, SQLCipher operations) without Node.js overhead.
3. **Binary size**: Uses the OS-native WebView (WebKit on macOS/Linux, WebView2 on Windows) — no bundled browser engine.
4. **Stronghold plugin**: Built-in encrypted key-value store for credential management.
5. **IPC type safety**: `generate_handler![]` macro ensures all IPC commands are registered; `invoke()` on the frontend can be fully typed via `tauriCommands.ts`.

---

## Consequences

**Positive:**
- Small distributable (<20 MB .dmg vs 150+ MB Electron .dmg)
- Rust's memory safety prevents a class of security bugs
- Tauri's CSP enforcement and capability ACL provide defense-in-depth
- Native OS dialogs, file pickers, and notifications

**Negative:**
- WebKit/WebView2 inconsistencies require cross-browser testing
- Rust compile times are longer than Node.js (mitigated by Docker CI caching)
- Tauri 2 is relatively new — smaller ecosystem than Electron
- macOS builds require a macOS runner (no cross-compilation)

**Neutral:**
- React frontend works identically to a web app — no desktop-specific UI code needed
- TypeScript IPC wrappers (`tauriCommands.ts`) decouple frontend from Tauri details
