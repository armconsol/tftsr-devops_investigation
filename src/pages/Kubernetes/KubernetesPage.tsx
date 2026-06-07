import React, { useState, useEffect } from "react";
import { useKubernetesStore } from "@/stores/kubernetesStore";
import { ClusterList } from "@/components/Kubernetes/ClusterList";
import { PortForwardList } from "@/components/Kubernetes/PortForwardList";
import { AddClusterModal } from "@/components/Kubernetes/AddClusterModal";
import { PortForwardForm } from "@/components/Kubernetes/PortForwardForm";
import { ResourceBrowser } from "@/components/Kubernetes/ResourceBrowser";
import type { ClusterInfo, PortForwardResponse } from "@/lib/tauriCommands";
import {
  listClustersCmd,
  removeClusterCmd,
  listPortForwardsCmd,
  stopPortForwardCmd,
  deletePortForwardCmd,
} from "@/lib/tauriCommands";

export function KubernetesPage() {
  const { clusters, addCluster, removeCluster, selectedClusterId } = useKubernetesStore();
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
      
      clustersData.forEach(addCluster);
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
      removeCluster(clusterId);
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
      await deletePortForwardCmd(id);
      setPortForwards((prev) => prev.filter((pf) => pf.id !== id));
    } catch (err) {
      console.error("Failed to delete port forward:", err);
      alert("Failed to delete port forward");
    }
  };

  const handleAddCluster = (cluster: ClusterInfo) => {
    addCluster(cluster);
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
          Manage your Kubernetes clusters and resources
        </p>
      </div>

      {/* Cluster Management Section */}
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold">Clusters</h2>
          <button
            onClick={() => setIsAddClusterOpen(true)}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
          >
            Add Cluster
          </button>
        </div>
        
        <ClusterList
          clusters={clusters}
          onAdd={() => setIsAddClusterOpen(true)}
          onRemove={handleRemoveCluster}
        />
      </div>

      {/* Port Forwarding Section */}
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold">Port Forwarding</h2>
          <button
            onClick={() => setIsStartPortForwardOpen(true)}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
          >
            Start Port Forward
          </button>
        </div>
        
        <PortForwardList
          portForwards={portForwards}
          onStart={() => setIsStartPortForwardOpen(true)}
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

      {/* Add Cluster Modal */}
      <AddClusterModal
        isOpen={isAddClusterOpen}
        onClose={() => setIsAddClusterOpen(false)}
        onAdd={handleAddCluster}
      />

      {/* Port Forward Form */}
      <PortForwardForm
        isOpen={isStartPortForwardOpen}
        onClose={() => setIsStartPortForwardOpen(false)}
        onStart={handleStartPortForward}
      />
    </div>
  );
}
