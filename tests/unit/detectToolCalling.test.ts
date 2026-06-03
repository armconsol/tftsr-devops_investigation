import { describe, it, expect, vi, beforeEach } from "vitest";
import * as tauriCommands from "@/lib/tauriCommands";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("detectToolCallingSupportCmd", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should be defined and callable", () => {
    expect(tauriCommands.detectToolCallingSupportCmd).toBeDefined();
    expect(typeof tauriCommands.detectToolCallingSupportCmd).toBe("function");
  });

  it("should accept a ProviderConfig parameter", () => {
    const mockConfig: tauriCommands.ProviderConfig = {
      name: "test-provider",
      provider_type: "openai",
      api_url: "https://api.example.com",
      api_key: "test-key",
      model: "gpt-4",
      max_tokens: 4096,
      temperature: 0.7,
      supports_tool_calling: false,
    };

    // Should not throw when called with valid config
    expect(() => tauriCommands.detectToolCallingSupportCmd(mockConfig)).not.toThrow();
  });

  it("should return a Promise<boolean>", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue(true);

    const mockConfig: tauriCommands.ProviderConfig = {
      name: "test-provider",
      provider_type: "openai",
      api_url: "https://api.example.com",
      api_key: "test-key",
      model: "gpt-4",
      max_tokens: 4096,
      temperature: 0.7,
      supports_tool_calling: false,
    };

    const result = tauriCommands.detectToolCallingSupportCmd(mockConfig);
    expect(result).toBeInstanceOf(Promise);

    const value = await result;
    expect(typeof value).toBe("boolean");
  });

  it("should call invoke with correct command name", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue(true);

    const mockConfig: tauriCommands.ProviderConfig = {
      name: "test-provider",
      provider_type: "openai",
      api_url: "https://api.example.com",
      api_key: "test-key",
      model: "gpt-4",
      max_tokens: 4096,
      temperature: 0.7,
      supports_tool_calling: false,
    };

    await tauriCommands.detectToolCallingSupportCmd(mockConfig);

    expect(invoke).toHaveBeenCalledWith("detect_tool_calling_support", {
      providerConfig: mockConfig,
    });
  });

  it("should handle true response correctly", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue(true);

    const mockConfig: tauriCommands.ProviderConfig = {
      name: "test-provider",
      provider_type: "openai",
      api_url: "https://api.example.com",
      api_key: "test-key",
      model: "gpt-4",
      max_tokens: 4096,
      temperature: 0.7,
      supports_tool_calling: false,
    };

    const result = await tauriCommands.detectToolCallingSupportCmd(mockConfig);
    expect(result).toBe(true);
  });

  it("should handle false response correctly", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue(false);

    const mockConfig: tauriCommands.ProviderConfig = {
      name: "test-provider",
      provider_type: "openai",
      api_url: "https://api.example.com",
      api_key: "test-key",
      model: "gpt-4",
      max_tokens: 4096,
      temperature: 0.7,
      supports_tool_calling: false,
    };

    const result = await tauriCommands.detectToolCallingSupportCmd(mockConfig);
    expect(result).toBe(false);
  });

  it("should propagate errors from backend", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockRejectedValue(new Error("Connection failed"));

    const mockConfig: tauriCommands.ProviderConfig = {
      name: "test-provider",
      provider_type: "openai",
      api_url: "https://api.example.com",
      api_key: "test-key",
      model: "gpt-4",
      max_tokens: 4096,
      temperature: 0.7,
      supports_tool_calling: false,
    };

    await expect(tauriCommands.detectToolCallingSupportCmd(mockConfig)).rejects.toThrow(
      "Connection failed"
    );
  });
});
