import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { IngressInfo } from "@/lib/tauriCommands";

interface IngressListProps {
  ingresses: IngressInfo[];
  _clusterId: string;
  _namespace: string;
}

export function IngressList({ ingresses, _clusterId, _namespace }: IngressListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Class</TableHead>
            <TableHead>Host</TableHead>
            <TableHead>Addresses</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {ingresses.length === 0 ? (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No ingresses found
              </TableCell>
            </TableRow>
          ) : (
            ingresses.map((ingress) => (
              <TableRow key={`${ingress.name}-${ingress.namespace}`}>
                <TableCell className="font-medium">{ingress.name}</TableCell>
                <TableCell>{ingress.namespace}</TableCell>
                <TableCell>{ingress.class || "-"}</TableCell>
                <TableCell>{ingress.host}</TableCell>
                <TableCell>{ingress.addresses.join(", ")}</TableCell>
                <TableCell className="text-muted-foreground">{ingress.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
