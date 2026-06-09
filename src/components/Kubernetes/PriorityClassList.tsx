import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Badge } from "@/components/ui";
import type { PriorityClassInfo } from "@/lib/tauriCommands";

interface PriorityClassListProps {
  items: PriorityClassInfo[];
  clusterId: string;
  namespace?: string;
}

export function PriorityClassList({ items }: PriorityClassListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Value</TableHead>
            <TableHead>Global Default</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={4} className="text-center text-muted-foreground">
                No priority classes found
              </TableCell>
            </TableRow>
          ) : (
            items.map((pc) => (
              <TableRow key={pc.name}>
                <TableCell className="font-medium">{pc.name}</TableCell>
                <TableCell className="text-sm font-mono">{pc.value}</TableCell>
                <TableCell className="text-sm">
                  {pc.global_default ? (
                    <Badge variant="success">Yes</Badge>
                  ) : (
                    <span className="text-muted-foreground">No</span>
                  )}
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">{pc.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
