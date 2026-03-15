import { describe, it, expect, beforeEach } from "vitest";
import { useSettingsStore } from "@/stores/settingsStore";
import type { ProviderConfig } from "@/lib/tauriCommands";

const mockProvider: ProviderConfig = {
  name: "openai",
  api_url: "https://api.openai.com/v1",
  api_key: "sk-test-key",
  model: "gpt-4o",
};

describe("Settings Store", () => {
  beforeEach(() => {
    useSettingsStore.setState({
      theme: "dark",
      ai_providers: [],
      active_provider: undefined,
      default_provider: "ollama",
      default_model: "llama3.2:3b",
      ollama_url: "http://localhost:11434",
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
});
