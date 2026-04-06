# Ticket Summary — UI Fixes + Ollama Bundling + Theme Toggle

**Branch**: `feat/ui-fixes-ollama-bundle-theme`

---

## Description

Multiple UI issues were identified and resolved following the arm64 build stabilization:

- `custom_rest` provider showed a disabled model input instead of the live dropdown already present lower in the form
- Auth Header Name auto-filled with an internal vendor-specific key name on format selection
- "User ID (CORE ID)" label and placeholder exposed internal organizational terminology
- Refresh buttons on the Ollama and Dashboard pages had near-zero contrast against dark card backgrounds
- PII detection toggles in Security settings silently reset to all-enabled on every app restart (no persistence)
- Ollama required manual installation; no offline install path existed
- No light/dark theme toggle UI existed despite the infrastructure already being wired up

Additionally, a new `install_ollama_from_bundle` Tauri command allows the app to copy a bundled Ollama binary to the system install path, enabling offline-first deployment. CI was updated to download the appropriate Ollama binary for each platform during the release build.

---

## Acceptance Criteria

- [ ] **Custom REST model**: Selecting Type=Custom + API Format=Custom REST causes the top-level Model row to disappear; the dropdown at the bottom is visible and populated with all models
- [ ] **Auth Header**: Field is blank by default when Custom REST format is selected (no internal values)
- [ ] **User ID label**: Reads "Email Address" with placeholder `user@example.com` and a generic description
- [ ] **Auth Header description**: No longer references internal key name examples
- [ ] **Refresh buttons**: Visually distinct (border + background) against dark card backgrounds on Dashboard and Ollama pages
- [ ] **PII toggles**: Toggling patterns off, navigating away, and returning preserves the disabled state across app restarts
- [ ] **Theme toggle**: Sun/Moon icon button in the sidebar footer switches between light and dark themes; works when sidebar is collapsed
- [ ] **Install Ollama (Offline)**: Button appears in the "Ollama Not Detected" card; clicking it copies the bundled binary and refreshes status
- [ ] **CI**: Each platform build job downloads the correct Ollama binary before `tauri build` and places it in `src-tauri/resources/ollama/`
- [ ] `npx tsc --noEmit` — zero errors
- [ ] `npm run test:run` — 51/51 tests pass
- [ ] `cargo check` — zero errors
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `python3 -c "import yaml; yaml.safe_load(open('.gitea/workflows/auto-tag.yml'))"` — YAML valid

---

## Work Implemented

### Phase 1 — Frontend (6 files)

**`src/pages/Settings/AIProviders.tsx`**
- Removed the disabled Model `<Input>` shown when Custom REST is active; the grid row is now hidden via conditional render — the dropdown further down the form handles model selection for this format
- Removed `custom_auth_header: "x-msi-genai-api-key"` prefill on format switch; field now starts empty
- Replaced example in Auth Header description from internal key name to generic `"x-api-key"`
- Renamed "User ID (CORE ID)" → "Email Address"; updated placeholder from `your.name@motorolasolutions.com` → `user@example.com`; removed Motorola-specific description text

**`src/pages/Dashboard/index.tsx`**
- Added `className="border-border text-foreground bg-card hover:bg-accent"` to Refresh `<Button>` for contrast against dark backgrounds

**`src/pages/Settings/Ollama.tsx`**
- Added same contrast classes to Refresh button
- Added `installOllamaFromBundleCmd` import
- Added `isInstallingBundle` state + `handleInstallFromBundle` async handler
- Added "Install Ollama (Offline)" primary `<Button>` alongside the existing "Download Ollama" link button in the "Ollama Not Detected" card

**`src/stores/settingsStore.ts`**
- Added `pii_enabled_patterns: Record<string, boolean>` field to `SettingsState` interface and store initializer (defaults all 8 patterns to `true`)
- Added `setPiiPattern(id, enabled)` action; both are included in the `persist` serialization so state survives app restarts

**`src/pages/Settings/Security.tsx`**
- Removed local `enabledPatterns` / `setEnabledPatterns` state and `togglePattern` function
- Added `useSettingsStore` import; reads `pii_enabled_patterns` / `setPiiPattern` from the persisted store
- Toggle button uses `setPiiPattern` directly on click

**`src/App.tsx`**
- Added `Sun`, `Moon` to lucide-react imports
- Extracted `setTheme` from `useSettingsStore` alongside `theme`
- Replaced static version `<div>` in sidebar footer with a flex row containing the version string and a Sun/Moon icon button; button is always visible even when sidebar is collapsed

### Phase 2 — Backend (4 files)

**`src-tauri/src/commands/system.rs`**
- Added `install_ollama_from_bundle(app: AppHandle) → Result<String, String>` command
- Resolves bundled binary via `app.path().resource_dir()`, copies to `/usr/local/bin/ollama` (Unix) or `%LOCALAPPDATA%\Programs\Ollama\ollama.exe` (Windows), sets 0o755 permissions on Unix
- Added `use tauri::Manager` import required by `app.path()`

**`src-tauri/src/lib.rs`**
- Registered `commands::system::install_ollama_from_bundle` in `tauri::generate_handler![]`

**`src/lib/tauriCommands.ts`**
- Added `installOllamaFromBundleCmd` typed wrapper: `() => invoke<string>("install_ollama_from_bundle")`

**`src-tauri/tauri.conf.json`**
- Changed `"resources": []` → `"resources": ["resources/ollama/*"]`
- Created `src-tauri/resources/ollama/.gitkeep` placeholder so Tauri's glob doesn't fail on builds without a bundled binary

### Phase 3 — CI + Docs (3 files)

**`.gitea/workflows/auto-tag.yml`**
- Added "Download Ollama" step to `build-linux-amd64`: downloads `ollama-linux-amd64.tgz`, extracts binary to `src-tauri/resources/ollama/ollama`
- Added "Download Ollama" step to `build-windows-amd64`: downloads `ollama-windows-amd64.zip`, extracts `ollama.exe`; added `unzip` to the Install dependencies step
- Added "Download Ollama" step to `build-macos-arm64`: downloads `ollama-darwin` universal binary directly
- Added "Download Ollama" step to `build-linux-arm64`: downloads `ollama-linux-arm64.tgz`, extracts binary

**`docs/wiki/IPC-Commands.md`**
- Added `install_ollama_from_bundle` entry under System/Ollama Commands section documenting parameters, return value, platform-specific install paths, and privilege requirement note

---

## Testing Needed

### Automated
```bash
npx tsc --noEmit                                                          # TS: zero errors
npm run test:run                                                          # Vitest: 51/51 pass
cargo check --manifest-path src-tauri/Cargo.toml                         # Rust: zero errors
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings         # Clippy: zero warnings
python3 -c "import yaml; yaml.safe_load(open('.gitea/workflows/auto-tag.yml'))" && echo OK
```

### Manual
1. **Custom REST model dropdown**: Settings → AI Providers → Add Provider → Type=Custom → API Format=Custom REST — the top Model row should disappear; the dropdown at the bottom should be visible and populated with all 19 models. Auth Header Name should be empty.
2. **Label rename**: Confirm "Email Address" label, `user@example.com` placeholder, no Motorola references.
3. **PII persistence**: Security page → toggle off "Email Addresses" and "IP Addresses" → navigate away → return → both should still be off. Restart the app → toggles should remain in the saved state.
4. **Refresh button contrast**: Dashboard and Ollama pages → confirm Refresh button border is visible on dark background.
5. **Theme toggle**: Sidebar footer → click Sun/Moon icon → theme should switch. Collapse sidebar → icon should still be accessible.
6. **Install Ollama (Offline)**: On a machine without Ollama, go to Settings → Ollama → "Ollama Not Detected" card should show "Install Ollama (Offline)" button. (Full test requires a release build with the bundled binary from CI.)
