import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Button } from "@/components/ui";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui";
import { Textarea } from "@/components/ui";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui";
import { Terminal, FileText, RotateCcw } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui";
import type { PodInfo, LogResponse } from "@/lib/tauriCommands";

interface PodListProps {
  pods: PodInfo[];
  clusterId: string;
  namespace: string;
}

export function PodList({ pods, clusterId, namespace }: PodListProps) {
  const [selectedPod, setSelectedPod] = useState<PodInfo | null>(null);
  const [selectedContainer, setSelectedContainer] = useState<string>("");
  const [logs, setLogs] = useState<string>("");
  const [isFetchingLogs, setIsFetchingLogs] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  const getPodStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case "running":
        return "bg-green-500";
      case "pending":
        return "bg-yellow-500";
      case "succeeded":
      case "completed":
        return "bg-blue-500";
      case "failed":
      case "error":
        return "bg-red-500";
      default:
        return "bg-gray-500";
    }
  };

  const fetchLogs = async () => {
    if (!selectedPod || !selectedContainer) return;

    setIsFetchingLogs(true);
    setError(null);
    try {
      const response = await invoke<LogResponse>("get_pod_logs", {
        clusterId,
        namespace,
        podName: selectedPod.name,
        containerName: selectedContainer,
      });
      setLogs(response.logs);
    } catch (err) {
      console.error("Failed to fetch logs:", err);
      setError(err instanceof Error ? err.message : "Failed to fetch logs");
    } finally {
      setIsFetchingLogs(false);
    }
  };

  const handleContainerChange = (container: string) => {
    setSelectedContainer(container);
    setLogs("");
    setError(null);
  };

  const containers = selectedPod ? [selectedPod.name] : [];

  return (
    <>
      <div className="overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Ready</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {pods.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  No pods found
                </TableCell>
              </TableRow>
            ) : (
              pods.map((pod) => (
                <TableRow key={pod.name}>
                  <TableCell className="font-medium">{pod.name}</TableCell>
                  <TableCell>
                    <Badge className={`${getPodStatusColor(pod.status)} text-white`}>
                      {pod.status}
                    </Badge>
                  </TableCell>
                  <TableCell>{pod.ready}</TableCell>
                  <TableCell className="text-muted-foreground">{pod.age}</TableCell>
                  <TableCell className="text-right">
                    <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
                      <Button variant="ghost" size="sm" onClick={() => { setSelectedPod(pod); setIsDialogOpen(true); }}>
                        <Terminal className="w-4 h-4" />
                      </Button>
                      <DialogContent className="max-w-4xl max-h-[80vh] flex flex-col">
                        <DialogHeader>
                          <DialogTitle>{pod.name} - {namespace} namespace</DialogTitle>
                        </DialogHeader>
                        <div className="flex-1 overflow-hidden flex flex-col">
                          {selectedPod && (
                            <div className="space-y-4">
                              <div className="flex items-center gap-2">
                                <span className="text-sm font-medium">Container:</span>
                                <select
                                  value={selectedContainer}
                                  onChange={(e) => handleContainerChange(e.target.value)}
                                  className="flex h-9 w-32 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                                >
                                  <option value="">Select container...</option>
                                  {containers.map((container) => (
                                    <option key={container} value={container}>
                                      {container}
                                    </option>
                                  ))}
                                </select>
                                <Button
                                  onClick={fetchLogs}
                                  disabled={!selectedContainer || isFetchingLogs}
                                  size="sm"
                                >
                                  {isFetchingLogs ? (
                                    <>
                                      <RotateCcw className="w-4 h-4 animate-spin" />
                                      Loading...
                                    </>
                                  ) : (
                                    <>
                                      <FileText className="w-4 h-4" />
                                      Fetch Logs
                                    </>
                                  )}
                                </Button>
                              </div>

                              {error && (
                                <Alert variant="destructive">
                                  <AlertDescription>{error}</AlertDescription>
                                </Alert>
                              )}

                              <Tabs value="logs" onValueChange={() => {}}>
                                <TabsList className="grid grid-cols-2">
                                  <TabsTrigger value="logs">Logs</TabsTrigger>
                                  <TabsTrigger value="details">Details</TabsTrigger>
                                </TabsList>
                                <div className="flex-1 overflow-auto">
                                  <TabsContent value="logs" className="h-full">
                                    <Textarea
                                      value={logs}
                                      readOnly
                                      className="font-mono text-xs h-64"
                                      placeholder="No logs available. Click 'Fetch Logs' to retrieve."
                                    />
                                  </TabsContent>
                                  <TabsContent value="details" className="h-full">
                                    <div className="space-y-2 text-sm">
                                      <div className="grid grid-cols-2 gap-2">
                                        <div className="text-muted-foreground">Name:</div>
                                        <div>{selectedPod.name}</div>
                                        <div className="text-muted-foreground">Status:</div>
                                        <div>{selectedPod.status}</div>
                                        <div className="text-muted-foreground">Ready:</div>
                                        <div>{selectedPod.ready}</div>
                                        <div className="text-muted-foreground">Age:</div>
                                        <div>{selectedPod.age}</div>
                                      </div>
                                    </div>
                                  </TabsContent>
                                </div>
                              </Tabs>
                            </div>
                          )}
                        </div>
                      </DialogContent>
                    </Dialog>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>
    </>
  );
}
