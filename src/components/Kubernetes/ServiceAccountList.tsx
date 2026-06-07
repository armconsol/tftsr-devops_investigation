import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { ServiceAccountInfo } from "@/lib/tauriCommands";

interface ServiceAccountListProps {
  serviceAccounts: ServiceAccountInfo[];
  _clusterId: string;
  _namespace: string;
}

export function ServiceAccountList({ serviceAccounts, _clusterId, _namespace }: ServiceAccountListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Secrets</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {serviceAccounts.length === 0 ? (
            <TableRow>
              <TableCell colSpan={4} className="text-center text-muted-foreground">
                No service accounts found
              </TableCell>
            </TableRow>
          ) : (
            serviceAccounts.map((sa) => (
              <TableRow key={`${sa.name}-${sa.namespace}`}>
                <TableCell className="font-medium">{sa.name}</TableCell>
                <TableCell>{sa.namespace}</TableCell>
                <TableCell>{sa.secrets}</TableCell>
                <TableCell className="text-muted-foreground">{sa.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
