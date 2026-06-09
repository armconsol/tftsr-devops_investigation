import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { EndpointSliceInfo } from "@/lib/tauriCommands";

interface EndpointSliceListProps {
  items: EndpointSliceInfo[];
  clusterId: string;
  namespace?: string;
}

export function EndpointSliceList({ items }: EndpointSliceListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Address Type</TableHead>
            <TableHead>Endpoints</TableHead>
            <TableHead>Ports</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No endpoint slices found
              </TableCell>
            </TableRow>
          ) : (
            items.map((eps) => (
              <TableRow key={`${eps.name}-${eps.namespace}`}>
                <TableCell className="font-medium">{eps.name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{eps.namespace}</TableCell>
                <TableCell className="text-sm font-mono">{eps.address_type}</TableCell>
                <TableCell className="text-sm">{eps.endpoints}</TableCell>
                <TableCell className="text-sm">
                  {eps.ports.length > 0 ? eps.ports.join(", ") : "—"}
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">{eps.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
