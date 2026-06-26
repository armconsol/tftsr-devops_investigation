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
import { Terminal, Loader2 } from "lucide-react";
import { execPodCmd } from "@/lib/tauriCommands";

interface ShellExecModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  clusterId: string;
  namespace: string;
  podName: string;
  containers: string[];
}

const SHELLS = [
  { label: "bash", value: "/bin/bash" },
  { label: "sh", value: "/bin/sh" },
  { label: "ash", value: "/bin/ash" },
];

export function ShellExecModal({
  open,
  onOpenChange,
  clusterId,
  namespace,
  podName,
  containers,
}: ShellExecModalProps) {
  const [selectedContainer, setSelectedContainer] = React.useState("");
  const [selectedShell, setSelectedShell] = React.useState("/bin/bash");
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

  const handleExec = async () => {
    if (!selectedContainer) return;
    setIsLoading(true);
    setError(null);
    try {
      const result = await execPodCmd(
        clusterId,
        namespace,
        podName,
        selectedContainer,
        selectedShell,
        selectedShell
      );
      const combined = [result.stdout, result.stderr].filter(Boolean).join("\n");
      setOutput(combined || `Exited with code ${result.exit_code ?? "unknown"}`);
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
            Exec — <span className="font-mono">{podName}</span>
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          <div className="flex items-center gap-2 flex-wrap">
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
            <Select value={selectedShell} onValueChange={setSelectedShell}>
              <SelectTrigger className="w-32">
                <SelectValue placeholder="Shell" />
              </SelectTrigger>
              <SelectContent>
                {SHELLS.map((s) => (
                  <SelectItem key={s.value} value={s.value}>
                    {s.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              size="sm"
              onClick={handleExec}
              disabled={!selectedContainer || isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Running...
                </>
              ) : (
                <>
                  <Terminal className="mr-2 h-4 w-4" />
                  Exec
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
            {output || "Select a container and shell, then click Exec."}
          </pre>
        </div>
      </DialogContent>
    </Dialog>
  );
}
