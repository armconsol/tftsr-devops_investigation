import React from "react";
import type { ProviderConfig } from "@/lib/tauriCommands";
import { useSettingsStore } from "@/stores/settingsStore";

interface ModelSelectorProps {
  onSelect: (provider: ProviderConfig) => void;
  selectedProvider?: ProviderConfig;
}

export function ModelSelector({ onSelect, selectedProvider }: ModelSelectorProps) {
  const providers = useSettingsStore((s) => s.ai_providers);

  const grouped = providers.reduce<Record<string, ProviderConfig[]>>((acc, p) => {
    const key = p.name;
    if (!acc[key]) acc[key] = [];
    acc[key].push(p);
    return acc;
  }, {});

  const providerTypeLabels: Record<string, string> = {
    openai: "OpenAI",
    anthropic: "Anthropic",
    ollama: "Ollama (Local)",
    azure: "Azure OpenAI",
    custom: "Custom",
  };

  return (
    <div className="relative">
      <select
        value={selectedProvider?.name ?? ""}
        onChange={(e) => {
          const provider = providers.find((p) => p.name === e.target.value);
          if (provider) onSelect(provider);
        }}
        className="flex h-10 w-full items-center rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
      >
        <option value="" disabled>
          Select a model...
        </option>
        {Object.entries(grouped).map(([type, configs]) => (
          <optgroup key={type} label={providerTypeLabels[type] ?? type}>
            {configs.map((config) => (
              <option key={config.name} value={config.name}>
                {config.name} ({config.model})
              </option>
            ))}
          </optgroup>
        ))}
      </select>
      {providers.length === 0 && (
        <p className="mt-1 text-xs text-muted-foreground">
          No providers configured. Add one in Settings &gt; AI Providers.
        </p>
      )}
    </div>
  );
}
