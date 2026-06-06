import React, { useState } from "react";
import { X, Loader2 } from "lucide-react";
import { Button } from "@/components/ui";
import type { ClusterInfo } from "@/lib/tauriCommands";
import { addClusterCmd } from "@/lib/tauriCommands";

interface AddClusterModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAdd: (cluster: ClusterInfo) => void;
}

export function AddClusterModal({ isOpen, onClose, onAdd }: AddClusterModalProps) {
  const [id, setId] = useState("");
  const [name, setName] = useState("");
  const [kubeconfig, setKubeconfig] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!id.trim() || !name.trim() || !kubeconfig.trim()) {
      setError("All fields are required");
      return;
    }

    setIsLoading(true);

    try {
      const cluster = await addClusterCmd(id, name, kubeconfig);
      onAdd(cluster);
      onClose();
      setId("");
      setName("");
      setKubeconfig("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add cluster");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-2xl rounded-lg border bg-background shadow-lg">
        <div className="flex items-center justify-between border-b px-6 py-4">
          <h3 className="text-lg font-semibold">Add Kubernetes Cluster</h3>
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
              Cluster ID
            </label>
            <input
              type="text"
              value={id}
              onChange={(e) => setId(e.target.value)}
              placeholder="e.g., prod-cluster-01"
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={isLoading}
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
              Cluster Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Production Cluster"
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={isLoading}
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
              Kubeconfig Content
            </label>
            <textarea
              value={kubeconfig}
              onChange={(e) => setKubeconfig(e.target.value)}
              placeholder="Paste your kubeconfig YAML here..."
              rows={10}
              className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 font-mono"
              disabled={isLoading}
            />
            <p className="text-xs text-muted-foreground">
              Paste the contents of your kubeconfig file (YAML format)
            </p>
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
              Add Cluster
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
