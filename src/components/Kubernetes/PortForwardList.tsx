import React from "react";
import { Trash2, Plus, Activity } from "lucide-react";
import { Button } from "@/components/ui";
import type { PortForwardResponse } from "@/lib/tauriCommands";

interface PortForwardListProps {
  portForwards: PortForwardResponse[];
  onStart: () => void;
  onStop: (id: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

export function PortForwardList({ portForwards, onStart, onStop, onDelete }: PortForwardListProps) {
  const handleStop = async (id: string) => {
    if (window.confirm("Are you sure you want to stop this port forward?")) {
      await onStop(id);
    }
  };

  const handleDelete = async (id: string) => {
    if (window.confirm("Are you sure you want to delete this port forward? This cannot be undone.")) {
      await onDelete(id);
    }
  };

  const getStatusColor = (status: string) => {
    const statusLower = status.toLowerCase().trim();
    if (statusLower === "") {
      return "bg-muted text-muted-foreground";
    }
    switch (statusLower) {
      case "active":
        return "bg-green-500/15 text-green-600 dark:text-green-400 border-green-500/20";
      case "stopped":
        return "bg-muted text-muted-foreground border-border";
      case "error":
        return "bg-red-500/15 text-red-600 dark:text-red-400 border-red-500/20";
      default:
        return "bg-muted text-muted-foreground";
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Activity className="w-5 h-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">Port Forwards</h2>
        </div>
        <Button onClick={onStart}>
          <Plus className="w-4 h-4 mr-2" />
          Start Port Forward
        </Button>
      </div>

      {portForwards.length === 0 ? (
        <div className="rounded-lg border border-dashed px-6 py-12 text-center">
          <Activity className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
          <h3 className="text-lg font-medium mb-2">No active port forwards</h3>
          <p className="text-sm text-muted-foreground mb-4">
            Start a port forward to expose a pod locally
          </p>
          <Button variant="outline" onClick={onStart}>
            <Plus className="w-4 h-4 mr-2" />
            Start Your First Port Forward
          </Button>
        </div>
      ) : (
        <div className="grid gap-4">
          {portForwards.map((pf) => (
            <div
              key={pf.id}
              className="rounded-lg border bg-card p-4 hover:border-primary/50 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium text-lg">Port Forward</h3>
                    <span
                      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium border ${getStatusColor(
                        pf.status
                      )}`}
                    >
                      {pf.status}
                    </span>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    Cluster: {pf.cluster_id}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    Namespace: {pf.namespace}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    Pod: {pf.pod}
                  </p>
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <span>Container Ports: {pf.container_ports.join(", ")}</span>
                    <span className="text-muted-foreground/50">|</span>
                    <span>Local Ports: {pf.local_ports.some(p => p > 0) ? pf.local_ports.join(", ") : "pending"}</span>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {pf.status.toLowerCase() === "active" && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleStop(pf.id)}
                    >
                      Stop
                    </Button>
                  )}
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => handleDelete(pf.id)}
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
