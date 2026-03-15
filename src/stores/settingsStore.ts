import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AppSettings, ProviderConfig } from "@/lib/tauriCommands";

interface SettingsState extends AppSettings {
  addProvider: (provider: ProviderConfig) => void;
  updateProvider: (index: number, provider: ProviderConfig) => void;
  removeProvider: (index: number) => void;
  setActiveProvider: (name: string) => void;
  setTheme: (theme: "light" | "dark") => void;
  getActiveProvider: () => ProviderConfig | undefined;
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
      setActiveProvider: (name) => set({ active_provider: name }),
      setTheme: (theme) => set({ theme }),
      getActiveProvider: () => {
        const state = get();
        return state.ai_providers.find((p) => p.name === state.active_provider)
          ?? state.ai_providers[0];
      },
    }),
    { name: "tftsr-settings" }
  )
);
