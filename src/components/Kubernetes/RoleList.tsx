import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { RoleInfo } from "@/lib/tauriCommands";

interface RoleListProps {
  roles: RoleInfo[];
  _clusterId: string;
  _namespace: string;
}

export function RoleList({ roles, _clusterId, _namespace }: RoleListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {roles.length === 0 ? (
            <TableRow>
              <TableCell colSpan={3} className="text-center text-muted-foreground">
                No roles found
              </TableCell>
            </TableRow>
          ) : (
            roles.map((role) => (
              <TableRow key={`${role.name}-${role.namespace}`}>
                <TableCell className="font-medium">{role.name}</TableCell>
                <TableCell>{role.namespace}</TableCell>
                <TableCell className="text-muted-foreground">{role.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
