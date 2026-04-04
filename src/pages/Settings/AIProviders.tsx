import React, { useState } from "react";
import { Plus, Pencil, Trash2, CheckCircle, XCircle, Zap } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  Button,
  Input,
  Label,
  Badge,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Separator,
} from "@/components/ui";
import { useSettingsStore } from "@/stores/settingsStore";
import { testProviderConnectionCmd, type ProviderConfig } from "@/lib/tauriCommands";

export const CUSTOM_REST_MODELS = [
  "ChatGPT4o",
  "ChatGPT4o-mini",
  "ChatGPT-o3-mini",
  "Gemini-2_0-Flash-001",
  "Gemini-2_5-Flash",
  "Claude-Sonnet-3_7",
  "Openai-gpt-4_1-mini",
  "Openai-o4-mini",
  "Claude-Sonnet-4",
  "ChatGPT-o3-pro",
  "OpenAI-ChatGPT-4_1",
  "OpenAI-GPT-4_1-Nano",
  "ChatGPT-5",
  "VertexGemini",
  "ChatGPT-5_1",
  "ChatGPT-5_1-chat",
  "ChatGPT-5_2-Chat",
  "Gemini-3_Pro-Preview",
  "Gemini-3_1-flash-lite-preview",
] as const;

export const CUSTOM_MODEL_OPTION = "__custom_model__";
export const LEGACY_API_FORMAT = "msi_genai";
export const CUSTOM_REST_FORMAT = "custom_rest";

export const normalizeApiFormat = (format?: string): string | undefined =>
  format === LEGACY_API_FORMAT ? CUSTOM_REST_FORMAT : format;

const emptyProvider: ProviderConfig = {
  name: "",
  provider_type: "openai",
  api_url: "",
  api_key: "",
  model: "",
  max_tokens: 4096,
  temperature: 0.7,
  custom_endpoint_path: undefined,
  custom_auth_header: undefined,
  custom_auth_prefix: undefined,
  api_format: undefined,
  session_id: undefined,
  user_id: undefined,
};

export default function AIProviders() {
  const {
    ai_providers,
    active_provider,
    addProvider,
    updateProvider,
    removeProvider,
    setActiveProvider,
  } = useSettingsStore();

  const [editIndex, setEditIndex] = useState<number | null>(null);
  const [isAdding, setIsAdding] = useState(false);
  const [form, setForm] = useState<ProviderConfig>({ ...emptyProvider });
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [isTesting, setIsTesting] = useState(false);
  const [isCustomModel, setIsCustomModel] = useState(false);
  const [customModelInput, setCustomModelInput] = useState("");

  const startAdd = () => {
    setForm({ ...emptyProvider });
    setEditIndex(null);
    setIsAdding(true);
    setTestResult(null);
    setIsCustomModel(false);
    setCustomModelInput("");
  };

  const startEdit = (index: number) => {
    const provider = ai_providers[index];
    const apiFormat = normalizeApiFormat(provider.api_format);
    const nextForm = { ...provider, api_format: apiFormat };

    setForm(nextForm);
    setEditIndex(index);
    setIsAdding(true);
    setTestResult(null);

    const isCustomRestProvider =
      nextForm.provider_type === "custom" && apiFormat === CUSTOM_REST_FORMAT;
    const knownModel = CUSTOM_REST_MODELS.includes(nextForm.model as (typeof CUSTOM_REST_MODELS)[number]);

    if (isCustomRestProvider && !knownModel) {
      setIsCustomModel(true);
      setCustomModelInput(nextForm.model);
    } else {
      setIsCustomModel(false);
      setCustomModelInput("");
    }
  };

  const handleSave = () => {
    if (!form.name || !form.api_url || !form.model) return;
    if (editIndex != null) {
      updateProvider(editIndex, form);
    } else {
      addProvider(form);
    }
    setIsAdding(false);
    setEditIndex(null);
    setForm({ ...emptyProvider });
  };

  const handleCancel = () => {
    setIsAdding(false);
    setEditIndex(null);
    setForm({ ...emptyProvider });
    setTestResult(null);
  };

  const handleTest = async () => {
    setIsTesting(true);
    setTestResult(null);
    try {
      const response = await testProviderConnectionCmd(form);
      setTestResult({ success: true, message: `OK: ${response.content.slice(0, 100)}` });
    } catch (err) {
      setTestResult({ success: false, message: String(err) });
    } finally {
      setIsTesting(false);
    }
  };

  const maskApiKey = (key?: string) => {
    if (!key) return "Not set";
    if (key.length <= 8) return "****";
    return key.slice(0, 4) + "..." + key.slice(-4);
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">AI Providers</h1>
          <p className="text-muted-foreground mt-1">
            Configure AI model providers for triage and document generation.
          </p>
        </div>
        {!isAdding && (
          <Button onClick={startAdd}>
            <Plus className="w-4 h-4 mr-2" />
            Add Provider
          </Button>
        )}
      </div>

      {/* Provider list */}
      {ai_providers.length === 0 && !isAdding && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground">No providers configured yet.</p>
            <Button className="mt-3" onClick={startAdd}>
              <Plus className="w-4 h-4 mr-2" />
              Add your first provider
            </Button>
          </CardContent>
        </Card>
      )}

      {ai_providers.map((provider, idx) => (
        <Card key={`${provider.name}-${idx}`}>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">{provider.name}</span>
                  <Badge variant="secondary">{provider.provider_type}</Badge>
                  {active_provider === provider.name && (
                    <Badge variant="success">Active</Badge>
                  )}
                </div>
                <p className="text-xs text-muted-foreground">
                  {provider.api_url} | Model: {provider.model} | Key: {maskApiKey(provider.api_key)}
                </p>
              </div>
              <div className="flex items-center gap-1">
                {active_provider !== provider.name && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setActiveProvider(provider.name)}
                  >
                    <Zap className="w-3 h-3 mr-1" />
                    Set Active
                  </Button>
                )}
                <Button variant="ghost" size="sm" onClick={() => startEdit(idx)}>
                  <Pencil className="w-3 h-3" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => removeProvider(idx)}
                >
                  <Trash2 className="w-3 h-3 text-destructive" />
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      ))}

      {/* Add/Edit form */}
      {isAdding && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">
              {editIndex != null ? "Edit Provider" : "Add Provider"}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>Name</Label>
                <Input
                  value={form.name}
                  onChange={(e) => setForm({ ...form, name: e.target.value })}
                  placeholder="My OpenAI"
                />
              </div>
              <div className="space-y-2">
                <Label>Type</Label>
                <Select
                  value={form.provider_type}
                  onValueChange={(v) => {
                    const type = v as ProviderConfig["provider_type"];
                    const defaults: Partial<ProviderConfig> =
                      type === "ollama"
                        ? { api_url: "http://localhost:11434", api_key: "", model: "llama3.2:3b" }
                        : type === "openai"
                        ? { api_url: "https://api.openai.com/v1" }
                        : type === "anthropic"
                        ? { api_url: "https://api.anthropic.com" }
                        : {};
                    setForm({ ...form, provider_type: type, ...defaults });
                  }}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="openai">OpenAI</SelectItem>
                    <SelectItem value="anthropic">Anthropic</SelectItem>
                    <SelectItem value="ollama">Ollama</SelectItem>
                    <SelectItem value="azure">Azure OpenAI</SelectItem>
                    <SelectItem value="custom">Custom</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-2">
              <Label>API URL</Label>
              <Input
                value={form.api_url}
                onChange={(e) => setForm({ ...form, api_url: e.target.value })}
                placeholder="https://api.openai.com/v1"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>API Key</Label>
                <Input
                  type="password"
                  value={form.api_key ?? ""}
                  onChange={(e) => setForm({ ...form, api_key: e.target.value })}
                  placeholder="sk-..."
                />
              </div>
              <div className="space-y-2">
                <Label>Model</Label>
                {form.provider_type === "custom"
                && normalizeApiFormat(form.api_format) === CUSTOM_REST_FORMAT ? (
                  <Input
                    value={form.model}
                    onChange={(e) => setForm({ ...form, model: e.target.value })}
                    placeholder="Select API Format below to choose model"
                    disabled
                  />
                ) : (
                  <Input
                    value={form.model}
                    onChange={(e) => setForm({ ...form, model: e.target.value })}
                    placeholder="gpt-4o"
                  />
                )}
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>Max Tokens</Label>
                <Input
                  type="number"
                  value={form.max_tokens ?? 4096}
                  onChange={(e) => setForm({ ...form, max_tokens: Number(e.target.value) })}
                />
              </div>
              <div className="space-y-2">
                <Label>Temperature</Label>
                <Input
                  type="number"
                  step="0.1"
                  min="0"
                  max="2"
                  value={form.temperature ?? 0.7}
                  onChange={(e) => setForm({ ...form, temperature: Number(e.target.value) })}
                />
              </div>
            </div>

            {/* Custom provider format options */}
            {form.provider_type === "custom" && (
              <>
                <Separator />
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>API Format</Label>
                    <Select
                      value={form.api_format ?? "openai"}
                      onValueChange={(v) => {
                        const format = v;
                        const defaults =
                          format === CUSTOM_REST_FORMAT
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
                        if (format !== CUSTOM_REST_FORMAT) {
                          setIsCustomModel(false);
                          setCustomModelInput("");
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="openai">OpenAI Compatible</SelectItem>
                        <SelectItem value={CUSTOM_REST_FORMAT}>Custom REST</SelectItem>
                      </SelectContent>
                    </Select>
                    <p className="text-xs text-muted-foreground">
                      Select the API format. Custom REST uses a non-OpenAI request/response structure.
                    </p>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label>Endpoint Path</Label>
                      <Input
                        value={form.custom_endpoint_path ?? ""}
                        onChange={(e) =>
                          setForm({ ...form, custom_endpoint_path: e.target.value })
                        }
                        placeholder="/chat/completions"
                      />
                      <p className="text-xs text-muted-foreground">
                        Path appended to API URL. Leave empty if URL includes full path.
                      </p>
                    </div>
                    <div className="space-y-2">
                      <Label>Auth Header Name</Label>
                      <Input
                        value={form.custom_auth_header ?? ""}
                        onChange={(e) =>
                          setForm({ ...form, custom_auth_header: e.target.value })
                        }
                        placeholder="Authorization"
                      />
                      <p className="text-xs text-muted-foreground">
                        Header name for authentication (e.g., "Authorization" or "x-msi-genai-api-key")
                      </p>
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
                      Prefix added before API key (e.g., "Bearer " for OpenAI, empty for Custom REST)
                    </p>
                  </div>

                  {/* Custom REST specific: User ID field */}
                  {normalizeApiFormat(form.api_format) === CUSTOM_REST_FORMAT && (
                    <div className="space-y-2">
                      <Label>User ID (CORE ID)</Label>
                      <Input
                        value={form.user_id ?? ""}
                        onChange={(e) => setForm({ ...form, user_id: e.target.value })}
                        placeholder="your.name@motorolasolutions.com"
                      />
                      <p className="text-xs text-muted-foreground">
                        Optional: Your Motorola CORE ID email. If omitted, costs are tracked to API key owner.
                      </p>
                    </div>
                  )}

                  {/* Custom REST specific: model dropdown with custom option */}
                  {normalizeApiFormat(form.api_format) === CUSTOM_REST_FORMAT && (
                    <div className="space-y-2">
                      <Label>Model</Label>
                      <Select
                        value={isCustomModel ? CUSTOM_MODEL_OPTION : form.model}
                        onValueChange={(value) => {
                          if (value === CUSTOM_MODEL_OPTION) {
                            setIsCustomModel(true);
                            if (CUSTOM_REST_MODELS.includes(form.model as (typeof CUSTOM_REST_MODELS)[number])) {
                              setForm({ ...form, model: "" });
                              setCustomModelInput("");
                            }
                          } else {
                            setIsCustomModel(false);
                            setCustomModelInput("");
                            setForm({ ...form, model: value });
                          }
                        }}
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select a model..." />
                        </SelectTrigger>
                        <SelectContent>
                          {CUSTOM_REST_MODELS.map((model) => (
                            <SelectItem key={model} value={model}>
                              {model}
                            </SelectItem>
                          ))}
                          <SelectItem value={CUSTOM_MODEL_OPTION}>Custom model...</SelectItem>
                        </SelectContent>
                      </Select>
                      {isCustomModel && (
                        <Input
                          value={customModelInput}
                          onChange={(e) => {
                            const value = e.target.value;
                            setCustomModelInput(value);
                            setForm({ ...form, model: value });
                          }}
                          placeholder="Enter custom model ID"
                        />
                      )}
                    </div>
                  )}
                </div>
              </>
            )}

            {/* Test result */}
            {testResult && (
              <div
                className={`flex items-center gap-2 rounded-md p-3 text-sm ${
                  testResult.success
                    ? "bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300"
                    : "bg-destructive/10 text-destructive"
                }`}
              >
                {testResult.success ? (
                  <CheckCircle className="w-4 h-4" />
                ) : (
                  <XCircle className="w-4 h-4" />
                )}
                {testResult.message}
              </div>
            )}

            <Separator />

            <div className="flex items-center gap-2">
              <Button onClick={handleSave}>Save</Button>
              <Button variant="outline" onClick={handleTest} disabled={isTesting}>
                {isTesting ? "Testing..." : "Test Connection"}
              </Button>
              <Button variant="ghost" onClick={handleCancel}>
                Cancel
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
