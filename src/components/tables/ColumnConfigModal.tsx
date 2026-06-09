import React from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  Button,
  Checkbox,
} from "@/components/ui";
import { RotateCcw, Eye, EyeOff } from "lucide-react";
import type { UseColumnConfigReturn } from "@/hooks/useColumnConfig";

interface ColumnConfigModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  resourceType: string;
  columnConfig: UseColumnConfigReturn;
  columnLabels: Record<string, string>; // key -> display label
}

export function ColumnConfigModal({
  open,
  onOpenChange,
  resourceType,
  columnConfig,
  columnLabels,
}: ColumnConfigModalProps) {
  const { isColumnVisible, toggleColumn, resetToDefaults, showAllColumns, hideAllColumns } =
    columnConfig;

  const columnKeys = Object.keys(columnLabels);
  const visibleCount = columnKeys.filter((key) => isColumnVisible(key)).length;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Configure {resourceType} Columns</DialogTitle>
          <DialogDescription>
            Choose which columns to display in the table. Changes are saved automatically.
          </DialogDescription>
        </DialogHeader>

        <div className="py-4">
          <div className="flex items-center justify-between mb-4 pb-3 border-b">
            <div className="text-sm text-muted-foreground">
              {visibleCount} of {columnKeys.length} columns visible
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={showAllColumns}
                className="flex items-center gap-1"
              >
                <Eye className="h-3 w-3" />
                Show All
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={hideAllColumns}
                className="flex items-center gap-1"
              >
                <EyeOff className="h-3 w-3" />
                Hide All
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={resetToDefaults}
                className="flex items-center gap-1"
              >
                <RotateCcw className="h-3 w-3" />
                Reset
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            {columnKeys.map((key) => (
              <label
                key={key}
                className="flex items-center gap-3 px-3 py-2 rounded hover:bg-accent cursor-pointer transition-colors"
              >
                <Checkbox
                  checked={isColumnVisible(key)}
                  onCheckedChange={() => toggleColumn(key)}
                />
                <span className="flex-1 text-sm">{columnLabels[key]}</span>
                {key === "name" && (
                  <span className="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">
                    Required
                  </span>
                )}
              </label>
            ))}
          </div>
        </div>

        <DialogFooter>
          <Button onClick={() => onOpenChange(false)}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
