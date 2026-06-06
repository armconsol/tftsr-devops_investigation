import React, { useState, useEffect } from "react";
import { Server, Activity } from "lucide-react";
import { ClusterList } from "@/components/Kubernetes/ClusterList";
import { PortForwardList } from "@/components/Kubernetes/PortForwardList";
import { AddClusterModal } from "@/components/Kubernetes/AddClusterModal";
import { PortForwardForm } from "@/components/Kubernetes/PortForwardForm";
import type { ClusterInfo, PortForwardResponse } from "@/lib/tauriCommands";
import {
  listClustersCmd,
  removeClusterCmd,
  listPortForwardsCmd,
  stopPortForwardCmd,
} from "@/lib/tauriCommands";

const deletePortForwardCmd = async (id: string): Promise<void> => {
  await stopPortForwardCmd(id);
};

export function KubernetesPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [portForwards, setPortForwards] = useState<PortForwardResponse[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isAddClusterOpen, setIsAddClusterOpen] = useState(false);
  const [isStartPortForwardOpen, setIsStartPortForwardOpen] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const [clustersData, portForwardsData] = await Promise.all([
        listClustersCmd(),
        listPortForwardsCmd(),
      ]);
      setClusters(clustersData);
      setPortForwards(portForwardsData);
    } catch (err) {
      console.error("Failed to load data:", err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRemoveCluster = async (clusterId: string) => {
    try {
      await removeClusterCmd(clusterId);
      setClusters((prev) => prev.filter((c) => c.id !== clusterId));
    } catch (err) {
      console.error("Failed to remove cluster:", err);
      alert("Failed to remove cluster");
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
      await stopPortForwardCmd(id);
      setPortForwards((prev) => prev.filter((pf) => pf.id !== id));
    } catch (err) {
      console.error("Failed to delete port forward:", err);
      alert("Failed to delete port forward");
    }
  };

  const handleAddCluster = (cluster: ClusterInfo) => {
    setClusters((prev) => [...prev, cluster]);
  };

  const handleStartPortForward = (portForward: PortForwardResponse) => {
    setPortForwards((prev) => [...prev, portForward]);
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
          Manage your Kubernetes clusters and port forwarding sessions
        </p>
      </div>

      <div className="grid gap-8">
        <ClusterList
          clusters={clusters}
          onAdd={() => setIsAddClusterOpen(true)}
          onRemove={handleRemoveCluster}
        />

        <PortForwardList
          portForwards={portForwards}
          onStart={() => setIsStartPortForwardOpen(true)}
          onStop={handleStopPortForward}
          onDelete={handleDeletePortForward}
        />
      </div>

      <AddClusterModal
        isOpen={isAddClusterOpen}
        onClose={() => setIsAddClusterOpen(false)}
        onAdd={handleAddCluster}
      />

      <PortForwardForm
        isOpen={isStartPortForwardOpen}
        onClose={() => setIsStartPortForwardOpen(false)}
        onStart={handleStartPortForward}
      />
    </div>
  );
}
