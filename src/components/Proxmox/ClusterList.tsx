import { useState, useEffect } from "react";
import { ClusterInfo } from "@/lib/domain";
import { listProxmoxClusters, removeProxmoxCluster } from "@/lib/proxmoxClient";
import { Button, Card, CardContent, CardHeader, CardTitle, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { toast } from "sonner";

export function ClusterList() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [loading, setLoading] = useState(true);

  const loadClusters = async () => {
    try {
      const loadedClusters = await listProxmoxClusters();
      setClusters(loadedClusters);
    } catch (error) {
      toast.error("Failed to load Proxmox clusters");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadClusters();
  }, []);

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

  if (loading) {
    return <div className="text-center py-8">Loading Proxmox clusters...</div>;
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Proxmox Clusters</CardTitle>
          <Button onClick={loadClusters}>Refresh</Button>
        </div>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Type</TableHead>
              <TableHead>URL</TableHead>
              <TableHead>Port</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {clusters.map((cluster) => (
              <TableRow key={cluster.id}>
                <TableCell className="font-medium">{cluster.name}</TableCell>
                <TableCell>
                  <span className="inline-flex items-center rounded-full bg-secondary px-2 py-1 text-xs font-medium">
                    {cluster.clusterType.toUpperCase()}
                  </span>
                </TableCell>
                <TableCell>{cluster.url}</TableCell>
                <TableCell>{cluster.port}</TableCell>
                <TableCell className="text-right">
                  <Button variant="ghost" size="sm" onClick={() => handleRemoveCluster(cluster.id)}>
                    Remove
                  </Button>
                </TableCell>
              </TableRow>
            ))}
            {clusters.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                  No Proxmox clusters configured
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}
