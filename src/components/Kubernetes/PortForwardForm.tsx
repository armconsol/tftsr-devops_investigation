import React, { useState } from "react";
import { X, Loader2 } from "lucide-react";
import { Button } from "@/components/ui";
import type { PortForwardResponse } from "@/lib/tauriCommands";
import { startPortForwardCmd } from "@/lib/tauriCommands";
import { listClustersCmd } from "@/lib/tauriCommands";

interface PortForwardFormProps {
  isOpen: boolean;
  onClose: () => void;
  onStart: (portForward: PortForwardResponse) => void;
}

export function PortForwardForm({ isOpen, onClose, onStart }: PortForwardFormProps) {
  const [clusterId, setClusterId] = useState("");
  const [namespace, setNamespace] = useState("default");
  const [pod, setPod] = useState("");
  const [containerPort, setContainerPort] = useState<string>("80");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const [clusters, setClusters] = useState<{ id: string; name: string }[]>([]);

  if (!isOpen) return null;

  React.useEffect(() => {
    if (isOpen) {
      loadClusters();
    }
  }, [isOpen]);

  const loadClusters = async () => {
    try {
      const clusters = await listClustersCmd();
      setClusters(clusters.map((c) => ({ id: c.id, name: c.name })));
    } catch (err) {
      console.error("Failed to load clusters:", err);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!clusterId || !namespace || !pod || !containerPort) {
      setError("All fields are required");
      return;
    }

    const port = parseInt(containerPort, 10);
    if (isNaN(port) || port < 1 || port > 65535) {
      setError("Container port must be a valid port number (1-65535)");
      return;
    }

    setIsLoading(true);

    try {
      const portForward = await startPortForwardCmd({
        cluster_id: clusterId,
        namespace,
        pod,
        container_port: port,
      });
      onStart(portForward);
      onClose();
      setClusterId("");
      setNamespace("default");
      setPod("");
      setContainerPort("80");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to start port forward");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-lg rounded-lg border bg-background shadow-lg">
        <div className="flex items-center justify-between border-b px-6 py-4">
          <h3 className="text-lg font-semibold">Start Port Forward</h3>
          <button
            onClick={onClose}
            className="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {error && (
            <div className="rounded-md bg-destructive/15 px-4 py-3 text-sm text-destructive">
              {error}
            </div>
          )}

          <div className="space-y-2">
            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
              Cluster
            </label>
            <select
              value={clusterId}
              onChange={(e) => setClusterId(e.target.value)}
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={isLoading}
            >
              <option value="" disabled>
                Select a cluster
              </option>
              {clusters.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name} ({c.id})
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
              Namespace
            </label>
            <input
              type="text"
              value={namespace}
              onChange={(e) => setNamespace(e.target.value)}
              placeholder="default"
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={isLoading}
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
              Pod Name
            </label>
            <input
              type="text"
              value={pod}
              onChange={(e) => setPod(e.target.value)}
              placeholder="e.g., nginx-abc123"
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={isLoading}
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
              Container Port
            </label>
            <input
              type="number"
              value={containerPort}
              onChange={(e) => setContainerPort(e.target.value)}
              placeholder="80"
              min="1"
              max="65535"
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={isLoading}
            />
          </div>

          <div className="flex justify-end gap-2 pt-4">
            <Button
              type="button"
              variant="outline"
              onClick={onClose}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
              Start Port Forward
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
