import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Button } from "@/components/ui";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui";
import { AlertCircle, Terminal } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui";
import type { NodeInfo } from "@/lib/tauriCommands";

interface NodeListProps {
  nodes: NodeInfo[];
  clusterId: string;
}

export function NodeList({ nodes, clusterId }: NodeListProps) {
  const [selectedNode, setSelectedNode] = useState<NodeInfo | null>(null);
  const [isCordoning, setIsCordoning] = useState(false);
  const [isUncordoning, setIsUncordoning] = useState(false);
  const [isDraining, setIsDraining] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getNodeStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case "ready":
        return "bg-green-500";
      case "notready":
        return "bg-red-500";
      case "schedulingdisabled":
        return "bg-yellow-500";
      default:
        return "bg-gray-500";
    }
  };

  const handleCordon = async () => {
    if (!selectedNode) return;
    
    setIsCordoning(true);
    setError(null);
    try {
      await invoke<void>("cordon_node", { clusterId, nodeName: selectedNode.name });
      setSelectedNode(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to cordon node");
    } finally {
      setIsCordoning(false);
    }
  };

  const handleUncordon = async () => {
    if (!selectedNode) return;
    
    setIsUncordoning(true);
    setError(null);
    try {
      await invoke<void>("uncordon_node", { clusterId, nodeName: selectedNode.name });
      setSelectedNode(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to uncordon node");
    } finally {
      setIsUncordoning(false);
    }
  };

  const handleDrain = async () => {
    if (!selectedNode) return;
    
    setIsDraining(true);
    setError(null);
    try {
      await invoke<void>("drain_node", { clusterId, nodeName: selectedNode.name });
      setSelectedNode(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to drain node");
    } finally {
      setIsDraining(false);
    }
  };

  return (
    <>
      <div className="overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Roles</TableHead>
              <TableHead>Version</TableHead>
              <TableHead>Internal IP</TableHead>
              <TableHead>OS Image</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {nodes.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground">
                  No nodes found
                </TableCell>
              </TableRow>
            ) : (
              nodes.map((node) => (
                <TableRow key={node.name}>
                  <TableCell className="font-medium">{node.name}</TableCell>
                  <TableCell>
                    <Badge className={`${getNodeStatusColor(node.status)} text-white`}>
                      {node.status}
                    </Badge>
                  </TableCell>
                  <TableCell>{node.roles}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{node.version}</TableCell>
                  <TableCell className="text-sm font-mono">{node.internal_ip}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{node.os_image}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{node.age}</TableCell>
                  <TableCell className="text-right">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setSelectedNode(node)}
                      className="text-primary hover:text-primary hover:bg-primary/10"
                    >
                      Manage
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {/* Node Management Dialog */}
      {selectedNode && (
        <Dialog open={true} onOpenChange={(open) => {
          if (!open) {
            setSelectedNode(null);
            setError(null);
          }
        }}>
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Terminal className="w-5 h-5" />
                Manage Node: {selectedNode.name}
              </DialogTitle>
            </DialogHeader>

            <div className="space-y-4 py-4">
              {/* Node Details */}
              <div className="grid grid-cols-2 gap-4 p-4 bg-muted rounded-lg">
                <div>
                  <p className="text-xs font-medium text-muted-foreground">Status</p>
                  <p className="font-semibold">{selectedNode.status}</p>
                </div>
                <div>
                  <p className="text-xs font-medium text-muted-foreground">Roles</p>
                  <p className="font-semibold">{selectedNode.roles}</p>
                </div>
                <div>
                  <p className="text-xs font-medium text-muted-foreground">Version</p>
                  <p className="font-semibold">{selectedNode.version}</p>
                </div>
                <div>
                  <p className="text-xs font-medium text-muted-foreground">OS Image</p>
                  <p className="font-semibold">{selectedNode.os_image}</p>
                </div>
                <div>
                  <p className="text-xs font-medium text-muted-foreground">Kernel</p>
                  <p className="font-semibold">{selectedNode.kernel_version}</p>
                </div>
                <div>
                  <p className="text-xs font-medium text-muted-foreground">Kubelet</p>
                  <p className="font-semibold">{selectedNode.kubelet_version}</p>
                </div>
                <div>
                  <p className="text-xs font-medium text-muted-foreground">Internal IP</p>
                  <p className="font-semibold font-mono">{selectedNode.internal_ip}</p>
                </div>
                {selectedNode.external_ip && (
                  <div>
                    <p className="text-xs font-medium text-muted-foreground">External IP</p>
                    <p className="font-semibold font-mono">{selectedNode.external_ip}</p>
                  </div>
                )}
              </div>

              {/* Action Buttons */}
              <div className="space-y-3">
                {selectedNode.roles.toLowerCase().includes("schedulingdisabled") ? (
                  <Button
                    onClick={handleUncordon}
                    disabled={isUncordoning}
                    className="w-full"
                  >
                    {isUncordoning ? "Uncordoning..." : "Uncordon Node"}
                  </Button>
                ) : (
                  <Button
                    onClick={handleCordon}
                    variant="outline"
                    disabled={isCordoning}
                    className="w-full"
                  >
                    {isCordoning ? "Cordoning..." : "Cordon Node"}
                  </Button>
                )}

                <Button
                  onClick={handleDrain}
                  variant="destructive"
                  disabled={isDraining}
                  className="w-full"
                >
                  {isDraining ? "Draining..." : "Drain Node"}
                </Button>
              </div>

              {error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}
            </div>
          </DialogContent>
        </Dialog>
      )}
    </>
  );
}
