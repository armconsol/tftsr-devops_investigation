import React from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui";
import { Button } from "@/components/ui";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";
import { Alert, AlertDescription } from "@/components/ui";
import { FileText, Loader2 } from "lucide-react";
import { getPodLogsCmd } from "@/lib/tauriCommands";

interface LogsModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  clusterId: string;
  namespace: string;
  podName: string;
  containers: string[];
}

export function LogsModal({
  open,
  onOpenChange,
  clusterId,
  namespace,
  podName,
  containers,
}: LogsModalProps) {
  const [selectedContainer, setSelectedContainer] = React.useState("");
  const [logs, setLogs] = React.useState("");
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    if (open) {
      setSelectedContainer(containers[0] ?? "");
      setLogs("");
      setError(null);
    }
  }, [open, containers]);

  const fetchLogs = async () => {
    if (!selectedContainer) return;
    setIsLoading(true);
    setError(null);
    try {
      const response = await getPodLogsCmd(clusterId, namespace, podName, selectedContainer);
      setLogs(response.logs);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl">
        <DialogHeader>
          <DialogTitle>
            Logs — <span className="font-mono">{podName}</span>
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          <div className="flex items-center gap-2">
            <Select value={selectedContainer} onValueChange={setSelectedContainer}>
              <SelectTrigger className="w-48">
                <SelectValue placeholder="Select container" />
              </SelectTrigger>
              <SelectContent>
                {containers.map((c) => (
                  <SelectItem key={c} value={c}>
                    {c}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              size="sm"
              onClick={fetchLogs}
              disabled={!selectedContainer || isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Loading...
                </>
              ) : (
                <>
                  <FileText className="mr-2 h-4 w-4" />
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
          <pre className="max-h-[50vh] overflow-auto rounded-md border bg-muted p-3 font-mono text-xs whitespace-pre-wrap break-all">
            {logs || "No logs. Select a container and click Fetch Logs."}
          </pre>
        </div>
      </DialogContent>
    </Dialog>
  );
}
