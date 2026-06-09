import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Badge } from "@/components/ui";
import type { NamespaceResourceInfo } from "@/lib/tauriCommands";

interface NamespaceListProps {
  items: NamespaceResourceInfo[];
  clusterId: string;
  namespace?: string;
}

function statusVariant(status: string): "success" | "destructive" | "secondary" {
  if (status === "Active") return "success";
  if (status === "Terminating") return "destructive";
  return "secondary";
}

export function NamespaceList({ items }: NamespaceListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={3} className="text-center text-muted-foreground">
                No namespaces found
              </TableCell>
            </TableRow>
          ) : (
            items.map((ns) => (
              <TableRow key={ns.name}>
                <TableCell className="font-medium">{ns.name}</TableCell>
                <TableCell className="text-sm">
                  <Badge variant={statusVariant(ns.status)}>{ns.status}</Badge>
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">{ns.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
