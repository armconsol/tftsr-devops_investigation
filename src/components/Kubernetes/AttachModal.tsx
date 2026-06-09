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
import { Link, Loader2 } from "lucide-react";
import { attachPodCmd } from "@/lib/tauriCommands";

interface AttachModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  clusterId: string;
  namespace: string;
  podName: string;
  containers: string[];
}

export function AttachModal({
  open,
  onOpenChange,
  clusterId,
  namespace,
  podName,
  containers,
}: AttachModalProps) {
  const [selectedContainer, setSelectedContainer] = React.useState("");
  const [output, setOutput] = React.useState("");
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    if (open) {
      setSelectedContainer(containers[0] ?? "");
      setOutput("");
      setError(null);
    }
  }, [open, containers]);

  const handleAttach = async () => {
    if (!selectedContainer) return;
    setIsLoading(true);
    setError(null);
    try {
      const result = await attachPodCmd(clusterId, namespace, podName, selectedContainer);
      setOutput(
        `Session ${result.session_id} — status: ${result.status}`
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>
            Attach — <span className="font-mono">{podName}</span>
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
              onClick={handleAttach}
              disabled={!selectedContainer || isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Attaching...
                </>
              ) : (
                <>
                  <Link className="mr-2 h-4 w-4" />
                  Attach
                </>
              )}
            </Button>
          </div>
          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          <pre className="max-h-[50vh] overflow-auto rounded-md bg-black p-3 font-mono text-xs text-green-400 whitespace-pre-wrap break-all">
            {output || "Select a container and click Attach."}
          </pre>
        </div>
      </DialogContent>
    </Dialog>
  );
}
