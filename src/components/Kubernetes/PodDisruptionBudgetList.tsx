import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { PodDisruptionBudgetInfo } from "@/lib/tauriCommands";

interface PodDisruptionBudgetListProps {
  items: PodDisruptionBudgetInfo[];
  clusterId: string;
  namespace?: string;
}

export function PodDisruptionBudgetList({ items }: PodDisruptionBudgetListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Min Available</TableHead>
            <TableHead>Max Unavailable</TableHead>
            <TableHead>Disruptions Allowed</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No pod disruption budgets found
              </TableCell>
            </TableRow>
          ) : (
            items.map((pdb) => (
              <TableRow key={`${pdb.name}-${pdb.namespace}`}>
                <TableCell className="font-medium">{pdb.name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{pdb.namespace}</TableCell>
                <TableCell className="text-sm">{pdb.min_available}</TableCell>
                <TableCell className="text-sm">{pdb.max_unavailable}</TableCell>
                <TableCell className="text-sm">{pdb.disruptions_allowed}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{pdb.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
