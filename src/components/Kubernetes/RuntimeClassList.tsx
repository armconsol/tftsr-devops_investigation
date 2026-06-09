import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { RuntimeClassInfo } from "@/lib/tauriCommands";

interface RuntimeClassListProps {
  items: RuntimeClassInfo[];
  clusterId: string;
  namespace?: string;
}

export function RuntimeClassList({ items }: RuntimeClassListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Handler</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={3} className="text-center text-muted-foreground">
                No runtime classes found
              </TableCell>
            </TableRow>
          ) : (
            items.map((rc) => (
              <TableRow key={rc.name}>
                <TableCell className="font-medium">{rc.name}</TableCell>
                <TableCell className="text-sm font-mono">{rc.handler}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{rc.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
