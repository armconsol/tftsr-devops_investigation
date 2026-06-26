import { useState } from "react";
import { ClusterInfo, ClusterType } from "@/lib/domain";
import { listProxmoxClusters, removeProxmoxCluster } from "@/lib/proxmoxClient";
import { Button, Dialog, DialogContent, DialogHeader, DialogTitle, Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Switch } from "@/components/ui";
import { toast } from "sonner";

interface ClusterSelectorProps {
  selectedClusterIds: string[];
  onClusterSelect: (clusterIds: string[]) => void;
  mode: "single" | "multiple" | "all";
}

export function ClusterSelector({ selectedClusterIds, onClusterSelect, mode }: ClusterSelectorProps) {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  const [isAdding, setIsAdding] = useState(false);
  const [newCluster, setNewCluster] = useState({
    id: "",
    name: "",
    clusterType: "ve" as ClusterType,
    url: "",
    port: 8006,
    username: "root@pam",
    password: "",
  });

  const handleAddCluster = async () => {
    try {
      await listProxmoxClusters();
      setClusters(await listProxmoxClusters());
      setIsAdding(false);
      toast.success("Cluster added successfully");
    } catch (error) {
      toast.error("Failed to add cluster");
      console.error(error);
    }
  };

  const handleRemoveCluster = async (id: string) => {
    try {
      await removeProxmoxCluster(id);
      setClusters(clusters.filter((c) => c.id !== id));
      toast.success("Cluster removed successfully");
    } catch (error) {
      toast.error("Failed to remove cluster");
      console.error(error);
    }
  };

  const handleClusterToggle = (id: string) => {
    if (mode === "single") {
      onClusterSelect([id]);
    } else {
      const isSelected = selectedClusterIds.includes(id);
      if (isSelected) {
        onClusterSelect(selectedClusterIds.filter((cid) => cid !== id));
      } else {
        onClusterSelect([...selectedClusterIds, id]);
      }
    }
  };

  return (
    <>
      <Button variant="outline" onClick={() => setIsOpen(true)}>
        {mode === "all" ? "All Clusters" : `Cluster: ${clusters.find((c) => c.id === selectedClusterIds[0])?.name || "None"}`}
      </Button>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Proxmox Cluster Selector</DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <Label>Cluster Selection Mode</Label>
                <p className="text-sm text-muted-foreground">
                  {mode === "single" && "Select one cluster"}
                  {mode === "multiple" && "Select multiple clusters"}
                  {mode === "all" && "All clusters selected"}
                </p>
              </div>
              {mode !== "all" && (
                <Button size="sm" onClick={() => setIsAdding(true)}>
                  + Add Cluster
                </Button>
              )}
            </div>

            <div className="space-y-2">
              {clusters.map((cluster) => (
                <div
                  key={cluster.id}
                  className="flex items-center justify-between p-3 border rounded-lg hover:bg-accent"
                >
                  <div className="flex items-center space-x-3">
                    <Switch
                      checked={selectedClusterIds.includes(cluster.id)}
                      onCheckedChange={() => handleClusterToggle(cluster.id)}
                      disabled={mode === "single" && selectedClusterIds.includes(cluster.id) && selectedClusterIds.length === 1}
                    />
                    <div className="space-y-1">
                      <div className="font-medium">{cluster.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {cluster.url}:{cluster.port} • {cluster.clusterType.toUpperCase()}
                      </div>
                    </div>
                  </div>
                  <Button variant="ghost" size="sm" onClick={() => handleRemoveCluster(cluster.id)}>
                    Remove
                  </Button>
                </div>
              ))}
              {clusters.length === 0 && (
                <div className="text-center text-muted-foreground py-8">
                  No Proxmox clusters configured. Click "+ Add Cluster" to add one.
                </div>
              )}
            </div>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={isAdding} onOpenChange={setIsAdding}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Proxmox Cluster</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="cluster-name">Cluster Name</Label>
              <Input
                id="cluster-name"
                value={newCluster.name}
                onChange={(e) => setNewCluster({ ...newCluster, name: e.target.value })}
                placeholder="e.g., Production Cluster"
              />
            </div>
            <div className="space-y-2">
              <Label>Cluster Type</Label>
              <Select
                value={newCluster.clusterType}
                onValueChange={(value: string) => setNewCluster({ ...newCluster, clusterType: value as ClusterType })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="ve">Proxmox VE (port 8006)</SelectItem>
                  <SelectItem value="pbs">Proxmox Backup Server (port 8007)</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="cluster-url">URL</Label>
              <Input
                id="cluster-url"
                value={newCluster.url}
                onChange={(e) => setNewCluster({ ...newCluster, url: e.target.value })}
                placeholder="https://pve.example.com"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="cluster-port">Port</Label>
              <Input
                id="cluster-port"
                type="number"
                value={newCluster.port}
                onChange={(e) => setNewCluster({ ...newCluster, port: parseInt(e.target.value) })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="cluster-username">Username</Label>
              <Input
                id="cluster-username"
                value={newCluster.username}
                onChange={(e) => setNewCluster({ ...newCluster, username: e.target.value })}
                placeholder="root@pam"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="cluster-password">Password</Label>
              <Input
                id="cluster-password"
                type="password"
                value={newCluster.password}
                onChange={(e) => setNewCluster({ ...newCluster, password: e.target.value })}
                placeholder="••••••••"
              />
            </div>
            <Button onClick={handleAddCluster} className="w-full">
              Add Cluster
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
