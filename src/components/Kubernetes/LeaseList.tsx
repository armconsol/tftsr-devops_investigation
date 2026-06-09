import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { LeaseInfo } from "@/lib/tauriCommands";

interface LeaseListProps {
  items: LeaseInfo[];
  clusterId: string;
  namespace?: string;
}

export function LeaseList({ items }: LeaseListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Holder</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={4} className="text-center text-muted-foreground">
                No leases found
              </TableCell>
            </TableRow>
          ) : (
            items.map((lease) => (
              <TableRow key={`${lease.name}-${lease.namespace}`}>
                <TableCell className="font-medium">{lease.name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{lease.namespace}</TableCell>
                <TableCell className="text-sm font-mono">{lease.holder || "—"}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{lease.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
