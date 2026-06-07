import React, { useEffect, useState, useCallback } from "react";
import { AlertCircle, RefreshCw, CheckCircle2, XCircle } from "lucide-react";
import { listKubeconfigsCmd, listNodesCmd } from "@/lib/tauriCommands";
import type { KubeconfigInfo, NodeInfo } from "@/lib/tauriCommands";

interface ClusterDetailsProps {
  clusterId: string;
}

interface InfoRowProps {
  label: string;
  value: React.ReactNode;
  mono?: boolean;
  testId?: string;
}

function InfoRow({ label, value, mono, testId }: InfoRowProps) {
  return (
    <div>
      <span className="text-sm text-muted-foreground">{label}</span>
      <p
        className={["font-medium mt-0.5 truncate", mono ? "font-mono text-xs" : ""].join(" ")}
        data-testid={testId}
      >
        {value}
      </p>
    </div>
  );
}

export function ClusterDetails({ clusterId }: ClusterDetailsProps) {
  const [kubeconfig, setKubeconfig] = useState<KubeconfigInfo | null>(null);
  const [nodes, setNodes] = useState<NodeInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notFound, setNotFound] = useState(false);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    setNotFound(false);
    try {
      const [kubeconfigs, nodesData] = await Promise.all([
        listKubeconfigsCmd(),
        listNodesCmd(clusterId),
      ]);

      const found = kubeconfigs.find((k) => k.id === clusterId) ?? null;
      if (!found) {
        setNotFound(true);
      } else {
        setKubeconfig(found);
        setNodes(nodesData);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [clusterId]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center" data-testid="details-loading">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <div className="animate-spin rounded-full h-8 w-8 border-2 border-primary border-t-transparent" />
          <span className="text-sm">Loading cluster details…</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full flex items-center justify-center" data-testid="details-error">
        <div className="flex flex-col items-center gap-3 text-center max-w-sm">
          <AlertCircle className="h-10 w-10 text-destructive" />
          <p className="font-medium">Failed to load cluster details</p>
          <p className="text-sm text-muted-foreground">{error}</p>
          <button
            onClick={() => void loadData()}
            className="flex items-center gap-2 px-4 py-2 rounded-md border text-sm hover:bg-accent transition-colors"
          >
            <RefreshCw className="h-4 w-4" />
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (notFound || !kubeconfig) {
    return (
      <div className="h-full flex items-center justify-center" data-testid="cluster-no-data">
        <div className="flex flex-col items-center gap-3 text-center max-w-sm">
          <AlertCircle className="h-10 w-10 text-muted-foreground" />
          <p className="font-medium">Cluster not found</p>
          <p className="text-sm text-muted-foreground">
            No kubeconfig found for cluster ID: {clusterId}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto space-y-6 p-1">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">Cluster Details</h2>
          <p className="text-muted-foreground text-sm mt-0.5">Cluster ID: {clusterId}</p>
        </div>
        <button
          onClick={() => void loadData()}
          className="flex items-center gap-2 px-3 py-1.5 rounded-md border text-sm hover:bg-accent transition-colors"
        >
          <RefreshCw className="h-4 w-4" />
          Refresh
        </button>
      </div>

      {/* Basic Information */}
      <div className="bg-card rounded-lg border">
        <div className="border-b px-6 py-4">
          <h3 className="font-semibold">Basic Information</h3>
        </div>
        <div className="p-6 grid grid-cols-2 gap-4">
          <InfoRow
            label="Name"
            value={kubeconfig.name}
            testId="cluster-name"
          />
          <InfoRow
            label="Context"
            value={kubeconfig.context}
            testId="cluster-context"
          />
          <InfoRow
            label="API Server"
            value={kubeconfig.cluster_url ?? "—"}
            mono
            testId="cluster-api-server"
          />
          <InfoRow
            label="Status"
            value={
              kubeconfig.is_active ? (
                <span className="flex items-center gap-1.5 text-green-600 dark:text-green-400">
                  <CheckCircle2 className="h-4 w-4" />
                  Active
                </span>
              ) : (
                <span className="flex items-center gap-1.5 text-muted-foreground">
                  <XCircle className="h-4 w-4" />
                  Inactive
                </span>
              )
            }
          />
        </div>
      </div>

      {/* Node table */}
      <div className="bg-card rounded-lg border">
        <div className="border-b px-6 py-4">
          <h3 className="font-semibold">Nodes ({nodes.length})</h3>
        </div>
        {nodes.length === 0 ? (
          <div className="p-6 text-center text-muted-foreground text-sm">
            No nodes found for this cluster
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b text-muted-foreground">
                  <th className="text-left px-4 py-3 font-medium">Name</th>
                  <th className="text-left px-4 py-3 font-medium">Status</th>
                  <th className="text-left px-4 py-3 font-medium">Roles</th>
                  <th className="text-left px-4 py-3 font-medium">Kubelet Version</th>
                  <th className="text-left px-4 py-3 font-medium">Age</th>
                </tr>
              </thead>
              <tbody>
                {nodes.map((node) => (
                  <tr
                    key={node.name}
                    className="border-b last:border-0 hover:bg-muted/30 transition-colors"
                  >
                    <td className="px-4 py-3 font-mono">{node.name}</td>
                    <td className="px-4 py-3">
                      <span
                        className={[
                          "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium",
                          node.status === "Ready"
                            ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                            : "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400",
                        ].join(" ")}
                      >
                        {node.status}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-muted-foreground">{node.roles || "—"}</td>
                    <td className="px-4 py-3 font-mono text-xs">{node.kubelet_version}</td>
                    <td className="px-4 py-3 text-muted-foreground">{node.age}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
