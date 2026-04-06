import { describe, it, expect, beforeEach } from "vitest";
import { useSettingsStore } from "@/stores/settingsStore";
import type { ProviderConfig } from "@/lib/tauriCommands";

const mockProvider: ProviderConfig = {
  name: "openai",
  api_url: "https://api.openai.com/v1",
  api_key: "sk-test-key",
  model: "gpt-4o",
};

const DEFAULT_PII_PATTERNS = ["email", "ip_address", "phone", "ssn", "credit_card", "hostname", "password", "api_key"];

describe("Settings Store", () => {
  beforeEach(() => {
    localStorage.clear();
    useSettingsStore.setState({
      theme: "dark",
      ai_providers: [],
      active_provider: undefined,
      default_provider: "ollama",
      default_model: "llama3.2:3b",
      ollama_url: "http://localhost:11434",
      pii_enabled_patterns: Object.fromEntries(DEFAULT_PII_PATTERNS.map((id) => [id, true])),
    });
  });

  it("adds a provider", () => {
    useSettingsStore.getState().addProvider(mockProvider);
    expect(useSettingsStore.getState().ai_providers).toHaveLength(1);
    expect(useSettingsStore.getState().ai_providers[0].name).toBe("openai");
  });

  it("removes a provider", () => {
    useSettingsStore.getState().addProvider(mockProvider);
    useSettingsStore.getState().removeProvider(0);
    expect(useSettingsStore.getState().ai_providers).toHaveLength(0);
  });

  it("updates a provider", () => {
    useSettingsStore.getState().addProvider(mockProvider);
    useSettingsStore.getState().updateProvider(0, { ...mockProvider, model: "gpt-4o-mini" });
    expect(useSettingsStore.getState().ai_providers[0].model).toBe("gpt-4o-mini");
  });

  it("toggles theme", () => {
    useSettingsStore.getState().setTheme("light");
    expect(useSettingsStore.getState().theme).toBe("light");
  });

  it("does not persist API keys to localStorage", () => {
    useSettingsStore.getState().addProvider(mockProvider);
    const raw = localStorage.getItem("tftsr-settings");
    expect(raw).toBeTruthy();
    expect(raw).not.toContain("sk-test-key");
  });
});

describe("Settings Store — PII patterns", () => {
  beforeEach(() => {
    localStorage.clear();
    useSettingsStore.setState({
      theme: "dark",
      ai_providers: [],
      active_provider: undefined,
      default_provider: "ollama",
      default_model: "llama3.2:3b",
      ollama_url: "http://localhost:11434",
      pii_enabled_patterns: Object.fromEntries(DEFAULT_PII_PATTERNS.map((id) => [id, true])),
    });
  });

  it("initializes all 8 PII patterns as enabled by default", () => {
    const patterns = useSettingsStore.getState().pii_enabled_patterns;
    for (const id of DEFAULT_PII_PATTERNS) {
      expect(patterns[id]).toBe(true);
    }
  });

  it("setPiiPattern disables a single pattern", () => {
    useSettingsStore.getState().setPiiPattern("email", false);
    expect(useSettingsStore.getState().pii_enabled_patterns["email"]).toBe(false);
  });

  it("setPiiPattern does not affect other patterns", () => {
    useSettingsStore.getState().setPiiPattern("email", false);
    for (const id of DEFAULT_PII_PATTERNS.filter((id) => id !== "email")) {
      expect(useSettingsStore.getState().pii_enabled_patterns[id]).toBe(true);
    }
  });

  it("setPiiPattern re-enables a disabled pattern", () => {
    useSettingsStore.getState().setPiiPattern("ssn", false);
    useSettingsStore.getState().setPiiPattern("ssn", true);
    expect(useSettingsStore.getState().pii_enabled_patterns["ssn"]).toBe(true);
  });

  it("pii_enabled_patterns is persisted to localStorage", () => {
    useSettingsStore.getState().setPiiPattern("api_key", false);
    const raw = localStorage.getItem("tftsr-settings");
    expect(raw).toBeTruthy();
    // Zustand persist wraps state in { state: {...}, version: ... }
    const parsed = JSON.parse(raw!);
    const stored = parsed.state ?? parsed;
    expect(stored.pii_enabled_patterns.api_key).toBe(false);
    expect(stored.pii_enabled_patterns.email).toBe(true);
  });
});
