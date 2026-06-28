// Copyright (c) 2025 Shaun Arman
// MIT License - see LICENSE file for details

import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AppSettings, ProviderConfig } from "@/lib/tauriCommands";

interface SettingsState extends AppSettings {
  addProvider: (provider: ProviderConfig) => void;
  updateProvider: (index: number, provider: ProviderConfig) => void;
  removeProvider: (index: number) => void;
  setProviders: (providers: ProviderConfig[]) => void;
  setActiveProvider: (name: string) => void;
  setTheme: (theme: "light" | "dark") => void;
  getActiveProvider: () => ProviderConfig | undefined;
  pii_enabled_patterns: Record<string, boolean>;
  setPiiPattern: (id: string, enabled: boolean) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      theme: "dark" as const,
      ai_providers: [] as ProviderConfig[],
      active_provider: undefined,
      default_provider: "ollama",
      default_model: "llama3.2:3b",
      ollama_url: "http://localhost:11434",

      addProvider: (provider) =>
        set((state) => ({ ai_providers: [...state.ai_providers, provider] })),
      updateProvider: (index, provider) =>
        set((state) => {
          const providers = [...state.ai_providers];
          providers[index] = provider;
          return { ai_providers: providers };
        }),
      removeProvider: (index) =>
        set((state) => ({
          ai_providers: state.ai_providers.filter((_, i) => i !== index),
        })),
      setProviders: (providers) => set({ ai_providers: providers }),
      setActiveProvider: (name) => set({ active_provider: name }),
      setTheme: (theme) => set({ theme }),
      pii_enabled_patterns: Object.fromEntries(
        ["email", "ip_address", "phone", "ssn", "credit_card", "hostname", "password", "api_key"]
          .map((id) => [id, true])
      ) as Record<string, boolean>,
      setPiiPattern: (id: string, enabled: boolean) =>
        set((state) => ({
          pii_enabled_patterns: { ...state.pii_enabled_patterns, [id]: enabled },
        })),
      getActiveProvider: () => {
        const state = get();
        return state.ai_providers.find((p) => p.name === state.active_provider)
          ?? state.ai_providers[0];
      },
    }),
    {
      name: "trcaa-settings",
      // Don't persist ai_providers to localStorage - they're stored in encrypted database
      partialize: (state) => ({
        theme: state.theme,
        active_provider: state.active_provider,
        default_provider: state.default_provider,
        default_model: state.default_model,
        ollama_url: state.ollama_url,
        pii_enabled_patterns: state.pii_enabled_patterns,
      }),
    }
  )
);
