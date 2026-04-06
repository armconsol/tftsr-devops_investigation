import React, { useEffect, useState } from "react";
import { Download, Trash2, RefreshCw } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  Button,
  Input,
  Badge,
  Progress,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui";
import { HardwareReport } from "@/components/HardwareReport";
import {
  checkOllamaInstalledCmd,
  detectHardwareCmd,
  recommendModelsCmd,
  pullOllamaModelCmd,
  deleteOllamaModelCmd,
  listOllamaModelsCmd,
  getOllamaInstallGuideCmd,
  type OllamaStatus,
  type HardwareInfo,
  type ModelRecommendation,
  type OllamaModel,
  type InstallGuide,
} from "@/lib/tauriCommands";
import { listen } from "@tauri-apps/api/event";

export default function Ollama() {
  const [status, setStatus] = useState<OllamaStatus | null>(null);
  const [installGuide, setInstallGuide] = useState<InstallGuide | null>(null);
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [recommendations, setRecommendations] = useState<ModelRecommendation[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [pullModel, setPullModel] = useState("");
  const [customModel, setCustomModel] = useState("");
  const [isPulling, setIsPulling] = useState(false);
  const [pullProgress, setPullProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const platform = navigator.platform.toLowerCase().includes("mac") ? "macos"
        : navigator.platform.toLowerCase().includes("win") ? "windows" : "linux";

      const [ollamaStatus, hw, recs, modelList, guide] = await Promise.all([
        checkOllamaInstalledCmd(),
        detectHardwareCmd(),
        recommendModelsCmd(),
        listOllamaModelsCmd().catch(() => [] as OllamaModel[]),
        getOllamaInstallGuideCmd(platform),
      ]);
      setStatus(ollamaStatus);
      setInstallGuide(guide);
      setHardware(hw);
      setRecommendations(recs);
      setModels(modelList);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const setup = async () => {
      unlisten = await listen<{ progress: number }>("model://progress", (event) => {
        setPullProgress(event.payload.progress);
        if (event.payload.progress >= 100) {
          setIsPulling(false);
          loadData();
        }
      });
    };
    setup();
    return () => {
      unlisten?.();
    };
  }, []);

  const handlePull = async () => {
    const modelName = pullModel === "__custom__" ? customModel : pullModel;
    if (!modelName.trim()) return;
    setIsPulling(true);
    setPullProgress(0);
    setError(null);
    try {
      await pullOllamaModelCmd(modelName.trim());
    } catch (err) {
      setError(String(err));
      setIsPulling(false);
    }
  };

  const handleDelete = async (modelName: string) => {
    try {
      await deleteOllamaModelCmd(modelName);
      await loadData();
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Ollama (Local AI)</h1>
          <p className="text-muted-foreground mt-1">
            Manage local AI models via Ollama for privacy-first inference.
          </p>
        </div>
        <Button variant="outline" onClick={loadData} disabled={isLoading} className="border-border text-foreground bg-card hover:bg-accent">
          <RefreshCw className={`w-4 h-4 mr-2 ${isLoading ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </div>

      {/* Hardware Report */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Hardware</CardTitle>
        </CardHeader>
        <CardContent>
          <HardwareReport hardware={hardware} recommendations={recommendations} />
        </CardContent>
      </Card>

      {/* Ollama Status */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Ollama Status</CardTitle>
        </CardHeader>
        <CardContent>
          {status ? (
            <div className="flex items-center gap-3">
              <Badge variant={status.installed ? "default" : "destructive"}>
                {status.installed ? "Installed" : "Not Installed"}
              </Badge>
              <Badge variant={status.running ? "default" : "secondary"}>
                {status.running ? "Running" : "Stopped"}
              </Badge>
              {status.version && (
                <span className="text-xs text-muted-foreground">v{status.version}</span>
              )}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              {isLoading ? "Checking status..." : "Unable to determine status."}
            </p>
          )}
        </CardContent>
      </Card>

      {/* Install Instructions — shown when Ollama is not detected */}
      {status && !status.installed && installGuide && (
        <Card className="border-yellow-500/50">
          <CardHeader>
            <CardTitle className="text-lg">
              Ollama Not Detected — Installation Required
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ol className="space-y-2 list-decimal list-inside">
              {installGuide.steps.map((step, i) => (
                <li key={i} className="text-sm text-muted-foreground">{step}</li>
              ))}
            </ol>
          </CardContent>
        </Card>
      )}

      {/* Model List */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Installed Models</CardTitle>
        </CardHeader>
        <CardContent>
          {models.length > 0 ? (
            <div className="space-y-2">
              {models.map((model: OllamaModel) => (
                <div
                  key={model.name}
                  className="flex items-center justify-between rounded-md border p-3"
                >
                  <div>
                    <p className="text-sm font-medium">{model.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {model.size} | Modified: {new Date(model.modified).toLocaleDateString()}
                    </p>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDelete(model.name)}
                  >
                    <Trash2 className="w-3 h-3 text-destructive" />
                  </Button>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No models installed.</p>
          )}
        </CardContent>
      </Card>

      {/* Pull Model */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Pull Model</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-3">
            <div className="flex-1">
              <Select value={pullModel} onValueChange={setPullModel}>
                <SelectTrigger>
                  <SelectValue placeholder="Select a model..." />
                </SelectTrigger>
                <SelectContent>
                  {recommendations.map((rec) => (
                    <SelectItem key={rec.name} value={rec.name}>
                      {rec.name} ({rec.size} GB)
                      {rec.recommended ? " - Recommended" : ""}
                    </SelectItem>
                  ))}
                  <SelectItem value="__custom__">Custom model name...</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {pullModel === "__custom__" && (
              <Input
                value={customModel}
                onChange={(e) => setCustomModel(e.target.value)}
                placeholder="e.g., llama3:8b"
                className="w-48"
              />
            )}
            <Button onClick={handlePull} disabled={isPulling}>
              <Download className="w-4 h-4 mr-2" />
              {isPulling ? "Pulling..." : "Pull"}
            </Button>
          </div>

          {isPulling && (
            <div className="space-y-1">
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span>Downloading...</span>
                <span>{pullProgress.toFixed(0)}%</span>
              </div>
              <Progress value={pullProgress} />
            </div>
          )}
        </CardContent>
      </Card>

      {error && (
        <div className="text-sm text-destructive bg-destructive/10 rounded-md p-3">
          {error}
        </div>
      )}
    </div>
  );
}
