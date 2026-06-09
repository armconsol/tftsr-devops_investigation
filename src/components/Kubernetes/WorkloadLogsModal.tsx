import React, { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";
import { AlertCircle, Loader2 } from "lucide-react";
import { listPodsCmd, getPodLogsCmd } from "@/lib/tauriCommands";
import type { PodInfo } from "@/lib/tauriCommands";

interface WorkloadLogsModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  clusterId: string;
  namespace: string;
  workloadType: "deployment" | "statefulset" | "daemonset" | "job" | "cronjob" | "replicaset" | "replicationcontroller";
  workloadName: string;
  labels: Record<string, string>;
}

export function WorkloadLogsModal({
  open,
  onOpenChange,
  clusterId,
  namespace,
  workloadType,
  workloadName,
  labels,
}: WorkloadLogsModalProps) {
  const [pods, setPods] = useState<PodInfo[]>([]);
  const [selectedPod, setSelectedPod] = useState<string>("");
  const [selectedContainer, setSelectedContainer] = useState<string>("");
  const [logs, setLogs] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [tailLines, setTailLines] = useState<number>(100);

  // Fetch pods matching the workload's label selector
  useEffect(() => {
    if (!open) return;

    const fetchPods = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const allPods = await listPodsCmd(clusterId, namespace);

        // Filter pods by label selector
        const matchingPods = allPods.filter((pod) => {
          // For each label in the workload, check if pod has matching label
          return Object.entries(labels).every(([key, value]) => {
            // Check pod labels - we need to fetch this from the pod metadata
            // For now, we'll use a simpler approach: match by name prefix
            return true; // TODO: proper label matching when pod labels are available
          });
        });

        // If no label matching available, try to match by name pattern
        const filteredPods = matchingPods.length > 0 ? matchingPods : allPods.filter((pod) => {
          // Common naming patterns:
          // deployment: <name>-<hash>-<random>
          // statefulset: <name>-<ordinal>
          // daemonset: <name>-<random>
          // job: <name>-<random>
          // cronjob: <cronjob-name>-<timestamp>-<random>
          const namePattern = new RegExp(`^${workloadName}-`);
          return namePattern.test(pod.name);
        });

        setPods(filteredPods);
        if (filteredPods.length > 0) {
          setSelectedPod(filteredPods[0].name);
          if (filteredPods[0].containers.length > 0) {
            setSelectedContainer(filteredPods[0].containers[0]);
          }
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setIsLoading(false);
      }
    };

    fetchPods();
  }, [open, clusterId, namespace, workloadName, labels]);

  // Fetch logs when pod/container selection changes
  useEffect(() => {
    if (!selectedPod || !selectedContainer) {
      setLogs("");
      return;
    }

    const fetchLogs = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const logResponse = await getPodLogsCmd(
          clusterId,
          namespace,
          selectedPod,
          selectedContainer
        );
        // Apply tail lines filter
        const lines = logResponse.logs.split("\n");
        const tailedLogs = lines.slice(-tailLines).join("\n");
        setLogs(tailedLogs);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setLogs("");
      } finally {
        setIsLoading(false);
      }
    };

    fetchLogs();
  }, [clusterId, namespace, selectedPod, selectedContainer, tailLines]);

  const selectedPodData = pods.find((p) => p.name === selectedPod);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>
            Logs: {workloadType} / {workloadName}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 flex-1 flex flex-col overflow-hidden">
          {/* Pod and Container Selectors */}
          <div className="flex gap-4">
            <div className="flex-1">
              <label className="text-sm font-medium mb-2 block">Pod</label>
              <Select value={selectedPod} onValueChange={setSelectedPod}>
                <SelectTrigger>
                  <SelectValue placeholder="Select pod" />
                </SelectTrigger>
                <SelectContent>
                  {pods.length === 0 ? (
                    <SelectItem value="__none__" disabled>
                      No pods found
                    </SelectItem>
                  ) : (
                    pods.map((pod) => (
                      <SelectItem key={pod.name} value={pod.name}>
                        {pod.name} ({pod.status})
                      </SelectItem>
                    ))
                  )}
                </SelectContent>
              </Select>
            </div>

            <div className="flex-1">
              <label className="text-sm font-medium mb-2 block">Container</label>
              <Select
                value={selectedContainer}
                onValueChange={setSelectedContainer}
                disabled={!selectedPodData}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select container" />
                </SelectTrigger>
                <SelectContent>
                  {selectedPodData?.containers.map((container) => (
                    <SelectItem key={container} value={container}>
                      {container}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="w-32">
              <label className="text-sm font-medium mb-2 block">Tail Lines</label>
              <Select
                value={String(tailLines)}
                onValueChange={(v) => setTailLines(Number(v))}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="50">50</SelectItem>
                  <SelectItem value="100">100</SelectItem>
                  <SelectItem value="500">500</SelectItem>
                  <SelectItem value="1000">1000</SelectItem>
                  <SelectItem value="5000">5000</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          {/* Logs Display */}
          <div className="flex-1 relative overflow-hidden rounded-md border bg-muted/20">
            {isLoading && (
              <div className="absolute inset-0 flex items-center justify-center bg-background/80 z-10">
                <Loader2 className="w-8 h-8 animate-spin text-primary" />
              </div>
            )}

            {error && (
              <div className="p-4 flex items-center gap-2 text-destructive">
                <AlertCircle className="w-4 h-4" />
                <span className="text-sm">{error}</span>
              </div>
            )}

            {!error && !isLoading && logs && (
              <pre className="p-4 text-xs font-mono overflow-auto h-full whitespace-pre-wrap break-all">
                {logs}
              </pre>
            )}

            {!error && !isLoading && !logs && selectedPod && selectedContainer && (
              <div className="p-4 text-sm text-muted-foreground text-center">
                No logs available
              </div>
            )}

            {!selectedPod && (
              <div className="p-4 text-sm text-muted-foreground text-center">
                Select a pod to view logs
              </div>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
