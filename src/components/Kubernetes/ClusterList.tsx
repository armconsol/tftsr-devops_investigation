import React from "react";
import { Trash2, Plus, Server, Activity } from "lucide-react";
import { Button } from "@/components/ui";
import type { ClusterInfo } from "@/lib/tauriCommands";
import { removeClusterCmd } from "@/lib/tauriCommands";

interface ClusterListProps {
  clusters: ClusterInfo[];
  onAdd: () => void;
  onRemove: (clusterId: string) => Promise<void>;
}

export function ClusterList({ clusters, onAdd, onRemove }: ClusterListProps) {
  const handleRemove = async (clusterId: string) => {
    if (window.confirm("Are you sure you want to remove this cluster?")) {
      await onRemove(clusterId);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Server className="w-5 h-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">Clusters</h2>
        </div>
        <Button onClick={onAdd}>
          <Plus className="w-4 h-4 mr-2" />
          Add Cluster
        </Button>
      </div>

      {clusters.length === 0 ? (
        <div className="rounded-lg border border-dashed px-6 py-12 text-center">
          <Server className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
          <h3 className="text-lg font-medium mb-2">No clusters configured</h3>
          <p className="text-sm text-muted-foreground mb-4">
            Add a Kubernetes cluster to start managing it
          </p>
          <Button variant="outline" onClick={onAdd}>
            <Plus className="w-4 h-4 mr-2" />
            Add Your First Cluster
          </Button>
        </div>
      ) : (
        <div className="grid gap-4">
          {clusters.map((cluster) => (
            <div
              key={cluster.id}
              className="rounded-lg border bg-card p-4 hover:border-primary/50 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="space-y-1">
                  <h3 className="font-medium text-lg">{cluster.name}</h3>
                  <p className="text-sm text-muted-foreground font-mono">
                    ID: {cluster.id}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    Context: {cluster.context}
                  </p>
                  <p className="text-sm text-muted-foreground font-mono break-all">
                    URL: {cluster.cluster_url}
                  </p>
                </div>
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => handleRemove(cluster.id)}
                >
                  <Trash2 className="w-4 h-4" />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
