import React from "react";
import { Loader2 } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  Button,
  Input,
  Label,
} from "@/components/ui";
import { startPortForwardCmd } from "@/lib/tauriCommands";

interface PortForwardDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  clusterId: string;
  namespace: string;
  podName?: string;
}

export function PortForwardDialog({
  open,
  onOpenChange,
  clusterId,
  namespace,
  podName,
}: PortForwardDialogProps) {
  const [pod, setPod] = React.useState(podName ?? "");
  const [containerPort, setContainerPort] = React.useState("");
  const [localPort, setLocalPort] = React.useState("");
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [success, setSuccess] = React.useState(false);

  React.useEffect(() => {
    if (open) {
      setPod(podName ?? "");
      setContainerPort("");
      setLocalPort("");
      setError(null);
      setSuccess(false);
    }
  }, [open, podName]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(false);

    const podValue = pod.trim();
    if (!podValue) {
      setError("Pod name is required.");
      return;
    }

    const portNum = parseInt(containerPort, 10);
    if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
      setError("Container port must be a valid number between 1 and 65535.");
      return;
    }

    let localPortNum: number | undefined;
    if (localPort.trim() !== "") {
      localPortNum = parseInt(localPort, 10);
      if (isNaN(localPortNum) || localPortNum < 1 || localPortNum > 65535) {
        setError("Local port must be a valid number between 1 and 65535.");
        return;
      }
    }

    setLoading(true);
    try {
      await startPortForwardCmd({
        cluster_id: clusterId,
        namespace,
        pod: podValue,
        container_port: portNum,
        local_port: localPortNum,
      });
      setSuccess(true);
      onOpenChange(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const isPodReadonly = podName !== undefined;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Start Port Forward</DialogTitle>
        </DialogHeader>

        <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4 py-2">
          <div className="space-y-1.5">
            <Label htmlFor="pfd-namespace">Namespace</Label>
            <Input
              id="pfd-namespace"
              value={namespace}
              readOnly
              disabled
            />
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="pfd-pod">Pod Name</Label>
            <Input
              id="pfd-pod"
              value={pod}
              onChange={(e) => setPod(e.target.value)}
              placeholder="e.g. nginx-abc123"
              readOnly={isPodReadonly}
              disabled={isPodReadonly || loading}
            />
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="pfd-container-port">Container Port</Label>
            <Input
              id="pfd-container-port"
              type="number"
              min={1}
              max={65535}
              value={containerPort}
              onChange={(e) => setContainerPort(e.target.value)}
              placeholder="80"
              disabled={loading}
            />
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="pfd-local-port">Local Port (optional)</Label>
            <Input
              id="pfd-local-port"
              type="number"
              min={1}
              max={65535}
              value={localPort}
              onChange={(e) => setLocalPort(e.target.value)}
              placeholder="auto"
              disabled={loading}
            />
          </div>

          {error && (
            <div className="rounded-md bg-destructive/15 px-4 py-3 text-sm text-destructive">
              {error}
            </div>
          )}

          {success && (
            <div className="rounded-md bg-green-500/15 px-4 py-3 text-sm text-green-600">
              Port forward started successfully.
            </div>
          )}

          <DialogFooter className="pt-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={loading}>
              {loading && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
              Start
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
