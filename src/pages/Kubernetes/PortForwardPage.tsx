import React, { useState, useEffect, useCallback } from "react";
import { Play, Square, Trash2, Plus, RefreshCw } from "lucide-react";
import { useKubernetesStore } from "@/stores/kubernetesStore";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Badge,
  Button,
} from "@/components/ui";
import type { PortForwardResponse } from "@/lib/tauriCommands";
import {
  listPortForwardsCmd,
  startPortForwardCmd,
  stopPortForwardCmd,
  deletePortForwardCmd,
} from "@/lib/tauriCommands";
import { PortForwardForm } from "@/components/Kubernetes";

export function PortForwardPage() {
  const { selectedClusterId } = useKubernetesStore();
  const [portForwards, setPortForwards] = useState<PortForwardResponse[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadPortForwards = useCallback(async () => {
    if (!selectedClusterId) return;
    setIsLoading(true);
    setError(null);
    try {
      const data = await listPortForwardsCmd();
      setPortForwards(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  }, [selectedClusterId]);

  useEffect(() => {
    loadPortForwards();
    const interval = setInterval(loadPortForwards, 5000);
    return () => clearInterval(interval);
  }, [loadPortForwards]);

  const handleStop = async (id: string) => {
    try {
      await stopPortForwardCmd(id);
      setPortForwards((prev) => prev.filter((pf) => pf.id !== id));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deletePortForwardCmd(id);
      setPortForwards((prev) => prev.filter((pf) => pf.id !== id));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleStart = async (pf: PortForwardResponse) => {
    try {
      if (!selectedClusterId) return;
      const result = await startPortForwardCmd({
        cluster_id: selectedClusterId,
        namespace: pf.namespace,
        pod: pf.pod,
        container_port: pf.container_ports[0] ?? 80,
        local_port: pf.local_ports[0] ?? 0,
      });
      setPortForwards((prev) => [...prev, result]);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case "active":
        return "bg-green-500";
      case "stopped":
        return "bg-gray-500";
      default:
        return "bg-red-500";
    }
  };

  if (!selectedClusterId) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4 text-center px-8">
        <Play className="w-16 h-16 text-muted-foreground" />
        <h2 className="text-2xl font-semibold">No cluster selected</h2>
        <p className="text-muted-foreground max-w-sm">
          Select a cluster from the dropdown to manage port forwards.
        </p>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">Port Forwarding</h1>
          <p className="text-sm text-muted-foreground mt-1">
            Manage port forwards to access pods locally
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={loadPortForwards}
            disabled={isLoading}
          >
            <RefreshCw className={`w-4 h-4 mr-2 ${isLoading ? "animate-spin" : ""}`} />
            Refresh
          </Button>
          <Button size="sm" onClick={() => setIsFormOpen(true)}>
            <Plus className="w-4 h-4 mr-2" />
            New Port Forward
          </Button>
        </div>
      </div>

      {error && (
        <div className="p-4 bg-destructive/10 border border-destructive/20 rounded-md text-destructive text-sm">
          {error}
        </div>
      )}

      <div className="border rounded-lg bg-card">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Namespace</TableHead>
              <TableHead>Kind</TableHead>
              <TableHead>Pod Port</TableHead>
              <TableHead>Local Port</TableHead>
              <TableHead>Protocol</TableHead>
              <TableHead>Address</TableHead>
              <TableHead>Status</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {portForwards.length === 0 ? (
              <TableRow>
                <TableCell colSpan={9} className="text-center text-muted-foreground py-8">
                  {isLoading ? "Loading port forwards..." : "No active port forwards"}
                </TableCell>
              </TableRow>
            ) : (
              portForwards.map((pf) => (
                <TableRow key={pf.id}>
                  <TableCell className="font-medium">{pf.pod}</TableCell>
                  <TableCell>{pf.namespace}</TableCell>
                  <TableCell>
                    <Badge variant="outline">Pod</Badge>
                  </TableCell>
                  <TableCell className="font-mono text-sm">
                    {pf.container_ports.join(", ")}
                  </TableCell>
                  <TableCell className="font-mono text-sm">
                    {pf.local_ports.join(", ")}
                  </TableCell>
                  <TableCell>TCP</TableCell>
                  <TableCell className="font-mono text-sm">
                    localhost:{pf.local_ports[0]}
                  </TableCell>
                  <TableCell>
                    <Badge className={`${getStatusColor(pf.status)} text-white`}>
                      {pf.status}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex gap-1 justify-end">
                      {pf.status.toLowerCase() === "active" ? (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleStop(pf.id)}
                          title="Stop"
                        >
                          <Square className="w-4 h-4" />
                        </Button>
                      ) : (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleStart(pf)}
                          title="Start"
                        >
                          <Play className="w-4 h-4" />
                        </Button>
                      )}
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleDelete(pf.id)}
                        title="Delete"
                      >
                        <Trash2 className="w-4 h-4 text-destructive" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <PortForwardForm
        isOpen={isFormOpen}
        onClose={() => setIsFormOpen(false)}
        onStart={(pf) => {
          setPortForwards((prev) => [...prev, pf]);
          setIsFormOpen(false);
        }}
      />
    </div>
  );
}
