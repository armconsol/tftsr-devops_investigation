import React from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui";
import { Button } from "@/components/ui";
import { AlertTriangle, Loader2 } from "lucide-react";

interface ConfirmDeleteDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  resourceType: string;
  resourceName: string;
  onConfirm: () => Promise<void> | void;
  isLoading?: boolean;
  variant?: "delete" | "force-delete";
}

export function ConfirmDeleteDialog({
  open,
  onOpenChange,
  resourceType,
  resourceName,
  onConfirm,
  isLoading = false,
  variant = "delete",
}: ConfirmDeleteDialogProps) {
  const isForce = variant === "force-delete";

  const handleConfirm = async () => {
    await onConfirm();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-destructive" />
            {isForce ? `Force Delete ${resourceType}` : `Delete ${resourceType}`}
          </DialogTitle>
          <DialogDescription>
            {isForce ? (
              <>
                Are you sure you want to <strong>force delete</strong>{" "}
                <span className="font-mono text-foreground">{resourceName}</span>?
                <br />
                <span className="mt-1 block text-destructive">
                  This will immediately terminate the resource with no grace period.
                </span>
              </>
            ) : (
              <>
                Are you sure you want to delete{" "}
                <span className="font-mono text-foreground">{resourceName}</span>? This
                action cannot be undone.
              </>
            )}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isLoading}>
            Cancel
          </Button>
          <Button variant="destructive" onClick={handleConfirm} disabled={isLoading}>
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Deleting...
              </>
            ) : isForce ? (
              "Force Delete"
            ) : (
              "Delete"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
