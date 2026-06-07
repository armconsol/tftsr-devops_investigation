import React, { useEffect, useState, useCallback } from "react";
import { Server, Box, Globe, Layers, AlertCircle, RefreshCw } from "lucide-react";
import {
  listNodesCmd,
  listPodsCmd,
  listDeploymentsCmd,
  listNamespacesCmd,
} from "@/lib/tauriCommands";
import type { NodeInfo, PodInfo, DeploymentInfo, NamespaceInfo } from "@/lib/tauriCommands";

interface ClusterOverviewProps {
  clusterId: string;
  clusterName?: string;
}

interface SummaryCardProps {
  title: string;
  value: number;
  subtitle?: string;
  icon: React.ReactNode;
  testId: string;
  subtitleTestId?: string;
}

function SummaryCard({ title, value, subtitle, icon, testId, subtitleTestId }: SummaryCardProps) {
  return (
    <div className="bg-card rounded-lg p-4 border">
      <div className="flex items-center justify-between pb-2">
        <h3 className="text-sm font-medium">{title}</h3>
        {icon}
      </div>
      <div className="text-2xl font-bold" data-testid={testId}>{value}</div>
      {subtitle && (
        <p className="text-xs text-muted-foreground mt-1" data-testid={subtitleTestId}>
          {subtitle}
        </p>
      )}
    </div>
  );
}

function nodeIsReady(node: NodeInfo): boolean {
  return node.status === "Ready";
}

export function ClusterOverview({ clusterId, clusterName }: ClusterOverviewProps) {
  const [nodes, setNodes] = useState<NodeInfo[]>([]);
  const [pods, setPods] = useState<PodInfo[]>([]);
  const [deployments, setDeployments] = useState<DeploymentInfo[]>([]);
  const [namespaces, setNamespaces] = useState<NamespaceInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [nodesData, podsData, deploymentsData, namespacesData] = await Promise.all([
        listNodesCmd(clusterId),
        listPodsCmd(clusterId, ""),
        listDeploymentsCmd(clusterId, ""),
        listNamespacesCmd(clusterId),
      ]);
      setNodes(nodesData);
      setPods(podsData);
      setDeployments(deploymentsData);
      setNamespaces(namespacesData);
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
      <div className="h-full flex items-center justify-center" data-testid="overview-loading">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <div className="animate-spin rounded-full h-8 w-8 border-2 border-primary border-t-transparent" />
          <span className="text-sm">Loading cluster overview…</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className="h-full flex items-center justify-center"
        data-testid="overview-error"
      >
        <div className="flex flex-col items-center gap-3 text-center max-w-sm">
          <AlertCircle className="h-10 w-10 text-destructive" />
          <p className="font-medium">Failed to load cluster data</p>
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

  const readyNodeCount = nodes.filter(nodeIsReady).length;
  const runningPodCount = pods.filter((p) => p.status === "Running").length;

  return (
    <div className="h-full overflow-y-auto space-y-6 p-1">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">Cluster Overview</h2>
          <p className="text-muted-foreground text-sm mt-0.5" data-testid="cluster-name-header">
            {clusterName ?? clusterId}
          </p>
        </div>
        <button
          onClick={() => void loadData()}
          className="flex items-center gap-2 px-3 py-1.5 rounded-md border text-sm hover:bg-accent transition-colors"
        >
          <RefreshCw className="h-4 w-4" />
          Refresh
        </button>
      </div>

      {/* Summary cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <SummaryCard
          title="Nodes"
          value={nodes.length}
          subtitle={`Ready: ${readyNodeCount}/${nodes.length}`}
          icon={<Server className="h-4 w-4 text-muted-foreground" />}
          testId="node-count"
          subtitleTestId="node-ready-status"
        />
        <SummaryCard
          title="Pods"
          value={pods.length}
          subtitle={`Running: ${runningPodCount}`}
          icon={<Box className="h-4 w-4 text-muted-foreground" />}
          testId="pod-count"
        />
        <SummaryCard
          title="Deployments"
          value={deployments.length}
          icon={<Layers className="h-4 w-4 text-muted-foreground" />}
          testId="deployment-count"
        />
        <SummaryCard
          title="Namespaces"
          value={namespaces.length}
          icon={<Globe className="h-4 w-4 text-muted-foreground" />}
          testId="namespace-count"
        />
      </div>

      {/* Node table */}
      <div className="bg-card rounded-lg border">
        <div className="border-b px-6 py-4">
          <h3 className="font-semibold">Nodes</h3>
        </div>
        {nodes.length === 0 ? (
          <div className="p-6 text-center text-muted-foreground text-sm">No nodes found</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b text-muted-foreground">
                  <th className="text-left px-4 py-3 font-medium">Name</th>
                  <th className="text-left px-4 py-3 font-medium">Status</th>
                  <th className="text-left px-4 py-3 font-medium">Roles</th>
                  <th className="text-left px-4 py-3 font-medium">Version</th>
                  <th className="text-left px-4 py-3 font-medium">Age</th>
                </tr>
              </thead>
              <tbody>
                {nodes.map((node) => (
                  <tr key={node.name} className="border-b last:border-0 hover:bg-muted/30 transition-colors">
                    <td className="px-4 py-3 font-mono">{node.name}</td>
                    <td className="px-4 py-3">
                      <span
                        className={[
                          "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium",
                          nodeIsReady(node)
                            ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                            : "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400",
                        ].join(" ")}
                      >
                        {node.status}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-muted-foreground">{node.roles || "—"}</td>
                    <td className="px-4 py-3 font-mono text-xs">{node.version}</td>
                    <td className="px-4 py-3 text-muted-foreground">{node.age}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Info note */}
      <p className="text-xs text-muted-foreground">
        Events are available in the Cluster → Events section.
      </p>
    </div>
  );
}
