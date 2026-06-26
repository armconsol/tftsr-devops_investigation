import React from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui";
import { Button } from "@/components/ui";
import { Input } from "@/components/ui";
import { Label } from "@/components/ui";
import { Loader2 } from "lucide-react";

interface ScaleModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  resourceType: string;
  resourceName: string;
  currentReplicas: number;
  onScale: (replicas: number) => Promise<void>;
}

export function ScaleModal({
  open,
  onOpenChange,
  resourceType,
  resourceName,
  currentReplicas,
  onScale,
}: ScaleModalProps) {
  const [value, setValue] = React.useState(String(currentReplicas));
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    if (open) {
      setValue(String(currentReplicas));
      setError(null);
    }
  }, [open, currentReplicas]);

  const handleSubmit = async () => {
    const replicas = parseInt(value, 10);
    if (isNaN(replicas) || replicas < 0) {
      setError("Enter a valid non-negative integer.");
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      await onScale(replicas);
      onOpenChange(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-sm">
        <DialogHeader>
          <DialogTitle>
            Scale {resourceType}
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-3 py-2">
          <p className="text-sm text-muted-foreground">
            Scaling <span className="font-mono text-foreground">{resourceName}</span>
          </p>
          <div className="space-y-1">
            <Label htmlFor="scale-replicas">Replica Count</Label>
            <Input
              id="scale-replicas"
              type="number"
              min={0}
              value={value}
              onChange={(e) => { setValue(e.target.value); setError(null); }}
            />
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isLoading}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={isLoading}>
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Scaling...
              </>
            ) : (
              "Scale"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
