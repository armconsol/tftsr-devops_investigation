import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { SecretInfo } from "@/lib/tauriCommands";

interface SecretListProps {
  secrets: SecretInfo[];
  _clusterId: string;
  _namespace: string;
}

export function SecretList({ secrets, _clusterId, _namespace }: SecretListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Data Keys</TableHead>
            <TableHead>Age</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {secrets.length === 0 ? (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No secrets found
              </TableCell>
            </TableRow>
          ) : (
            secrets.map((secret) => (
              <TableRow key={`${secret.name}-${secret.namespace}`}>
                <TableCell className="font-medium">{secret.name}</TableCell>
                <TableCell>{secret.namespace}</TableCell>
                <TableCell>{secret.type}</TableCell>
                <TableCell>{secret.data_keys}</TableCell>
                <TableCell className="text-muted-foreground">{secret.age}</TableCell>
                <TableCell className="text-right">
                  <span className="text-sm">View/Edit</span>
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
