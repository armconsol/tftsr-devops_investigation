import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { ClusterRoleInfo } from "@/lib/tauriCommands";

interface ClusterRoleListProps {
  clusterRoles: ClusterRoleInfo[];
  _clusterId: string;
}

export function ClusterRoleList({ clusterRoles, _clusterId }: ClusterRoleListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {clusterRoles.length === 0 ? (
            <TableRow>
              <TableCell colSpan={2} className="text-center text-muted-foreground">
                No cluster roles found
              </TableCell>
            </TableRow>
          ) : (
            clusterRoles.map((clusterRole) => (
              <TableRow key={clusterRole.name}>
                <TableCell className="font-medium">{clusterRole.name}</TableCell>
                <TableCell className="text-muted-foreground">{clusterRole.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
