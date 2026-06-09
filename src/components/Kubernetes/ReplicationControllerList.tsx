import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { ReplicationControllerInfo } from "@/lib/tauriCommands";

interface ReplicationControllerListProps {
  items: ReplicationControllerInfo[];
  clusterId: string;
  namespace?: string;
}

export function ReplicationControllerList({ items }: ReplicationControllerListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Desired</TableHead>
            <TableHead>Ready</TableHead>
            <TableHead>Current</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No replication controllers found
              </TableCell>
            </TableRow>
          ) : (
            items.map((rc) => (
              <TableRow key={`${rc.name}-${rc.namespace}`}>
                <TableCell className="font-medium">{rc.name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{rc.namespace}</TableCell>
                <TableCell className="text-sm">{rc.desired}</TableCell>
                <TableCell className="text-sm">{rc.ready}</TableCell>
                <TableCell className="text-sm">{rc.current}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{rc.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
