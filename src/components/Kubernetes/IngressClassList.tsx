import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Badge } from "@/components/ui";
import type { IngressClassInfo } from "@/lib/tauriCommands";

interface IngressClassListProps {
  items: IngressClassInfo[];
  clusterId: string;
  namespace?: string;
}

export function IngressClassList({ items }: IngressClassListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Controller</TableHead>
            <TableHead>Default</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={4} className="text-center text-muted-foreground">
                No ingress classes found
              </TableCell>
            </TableRow>
          ) : (
            items.map((ic) => (
              <TableRow key={ic.name}>
                <TableCell className="font-medium">{ic.name}</TableCell>
                <TableCell className="text-sm font-mono">{ic.controller}</TableCell>
                <TableCell className="text-sm">
                  {ic.is_default ? (
                    <Badge variant="success">Yes</Badge>
                  ) : (
                    <span className="text-muted-foreground">No</span>
                  )}
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">{ic.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
