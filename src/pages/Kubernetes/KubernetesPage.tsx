import React, { useState, useEffect } from "react";
import { useKubernetesStore } from "@/stores/kubernetesStore";

import { PortForwardList } from "@/components/Kubernetes/PortForwardList";
import { PortForwardForm } from "@/components/Kubernetes/PortForwardForm";
import { ResourceBrowser } from "@/components/Kubernetes/ResourceBrowser";
import type { PortForwardResponse, KubeconfigInfo, PortForwardRequest } from "@/lib/tauriCommands";
import {
  listPortForwardsCmd,
  stopPortForwardCmd,
  deletePortForwardCmd,
  listKubeconfigsCmd,
  activateKubeconfigCmd,
  startPortForwardCmd,
} from "@/lib/tauriCommands";

export function KubernetesPage() {
  const { selectedClusterId } = useKubernetesStore();
  const [kubeconfigs, setKubeconfigs] = useState<KubeconfigInfo[]>([]);
  const [portForwards, setPortForwards] = useState<PortForwardResponse[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const [kubeconfigsData, portForwardsData] = await Promise.all([
        listKubeconfigsCmd(),
        listPortForwardsCmd(),
      ]);
      
      setKubeconfigs(kubeconfigsData);
      setPortForwards(portForwardsData);
    } catch (err) {
      console.error("Failed to load data:", err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleActivateKubeconfig = async (id: string) => {
    try {
      await activateKubeconfigCmd(id);
      await loadData();
    } catch (err) {
      console.error("Failed to activate kubeconfig:", err);
      alert("Failed to activate kubeconfig");
    }
  };

  const handleStopPortForward = async (id: string) => {
    try {
      await stopPortForwardCmd(id);
      setPortForwards((prev) => prev.filter((pf) => pf.id !== id));
    } catch (err) {
      console.error("Failed to stop port forward:", err);
      alert("Failed to stop port forward");
    }
  };

  const handleDeletePortForward = async (id: string) => {
    try {
      await deletePortForwardCmd(id);
      setPortForwards((prev) => prev.filter((pf) => pf.id !== id));
    } catch (err) {
      console.error("Failed to delete port forward:", err);
      alert("Failed to delete port forward");
    }
  };

  const handleStartPortForward = async (portForward: PortForwardRequest) => {
    try {
      const result = await startPortForwardCmd(portForward);
      setPortForwards((prev) => [...prev, result]);
    } catch (err) {
      console.error("Failed to start port forward:", err);
      alert("Failed to start port forward");
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="flex flex-col items-center gap-4">
          <div className="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin" />
          <p className="text-muted-foreground">Loading Kubernetes resources...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-6 space-y-8">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-bold tracking-tight">Kubernetes Management</h1>
        <p className="text-muted-foreground">
          Manage your Kubernetes clusters and resources
        </p>
      </div>

      {/* Cluster Management Section - Uses kubeconfig files from Settings */}
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold">Clusters (from kubeconfig files)</h2>
          <div className="flex gap-2">
            <button
              onClick={() => window.location.href = "/settings/kubeconfig"}
              className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/90"
            >
              Manage kubeconfigs
            </button>
          </div>
        </div>
        
        {kubeconfigs.length === 0 ? (
          <div className="rounded-lg border border-dashed px-6 py-12 text-center bg-card">
            <div className="mx-auto w-12 h-12 text-muted-foreground mb-4">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
              </svg>
            </div>
            <h3 className="text-lg font-medium mb-2">No kubeconfig files uploaded</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Upload kubeconfig files in Settings → Kubeconfig to manage Kubernetes clusters
            </p>
            <button
              onClick={() => window.location.href = "/settings/kubeconfig"}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
            >
              Go to Kubeconfig Manager
            </button>
          </div>
        ) : (
          <div className="grid gap-4">
            {kubeconfigs.map((config) => (
              <div
                key={config.id}
                className={`rounded-lg border bg-card p-4 hover:border-primary/50 transition-colors ${
                  config.is_active ? "border-primary ring-1 ring-primary/20" : ""
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="space-y-1 flex-1">
                    <div className="flex items-center gap-2">
                      <h3 className="font-medium text-lg">{config.name}</h3>
                      {config.is_active && (
                        <span className="px-2 py-1 text-xs font-semibold bg-green-100 text-green-800 rounded">
                          Active
                        </span>
                      )}
                    </div>
                    <div className="text-sm text-muted-foreground space-y-1">
                      <div>
                        <span className="font-medium">Context:</span> {config.context}
                      </div>
                      {config.cluster_url && (
                        <div>
                          <span className="font-medium">Cluster:</span> {config.cluster_url}
                        </div>
                      )}
                    </div>
                  </div>
                  <div className="flex gap-2">
                    {!config.is_active && (
                      <button
                        onClick={() => handleActivateKubeconfig(config.id)}
                        className="px-3 py-1 text-sm bg-secondary text-secondary-foreground rounded hover:bg-secondary/90"
                      >
                        Activate
                      </button>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Port Forwarding Section */}
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold">Port Forwarding</h2>
        </div>
        
        <PortForwardList
          portForwards={portForwards}
          onStart={() => {}}
          onStop={handleStopPortForward}
          onDelete={handleDeletePortForward}
        />
      </div>

      {/* Resource Browser Section */}
      {selectedClusterId && (
        <div className="space-y-6">
          <h2 className="text-xl font-semibold">Resource Browser</h2>
          <ResourceBrowser clusterId={selectedClusterId} />
        </div>
      )}
    </div>
  );
}


