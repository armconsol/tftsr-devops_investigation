import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { EndpointInfo } from "@/lib/tauriCommands";

interface EndpointListProps {
  items: EndpointInfo[];
  clusterId: string;
  namespace?: string;
}

export function EndpointList({ items }: EndpointListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Addresses</TableHead>
            <TableHead>Ports</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={5} className="text-center text-muted-foreground">
                No endpoints found
              </TableCell>
            </TableRow>
          ) : (
            items.map((ep) => (
              <TableRow key={`${ep.name}-${ep.namespace}`}>
                <TableCell className="font-medium">{ep.name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{ep.namespace}</TableCell>
                <TableCell className="text-sm font-mono">
                  {ep.addresses.length > 0 ? ep.addresses.join(", ") : "—"}
                </TableCell>
                <TableCell className="text-sm">
                  {ep.ports.length > 0 ? ep.ports.join(", ") : "—"}
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">{ep.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
