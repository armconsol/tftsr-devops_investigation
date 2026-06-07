import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { ClusterRoleBindingInfo } from "@/lib/tauriCommands";

interface ClusterRoleBindingListProps {
  clusterRoleBindings: ClusterRoleBindingInfo[];
  _clusterId: string;
}

export function ClusterRoleBindingList({ clusterRoleBindings, _clusterId }: ClusterRoleBindingListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Cluster Role</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {clusterRoleBindings.length === 0 ? (
            <TableRow>
              <TableCell colSpan={3} className="text-center text-muted-foreground">
                No cluster role bindings found
              </TableCell>
            </TableRow>
          ) : (
            clusterRoleBindings.map((crb) => (
              <TableRow key={crb.name}>
                <TableCell className="font-medium">{crb.name}</TableCell>
                <TableCell>{crb.cluster_role}</TableCell>
                <TableCell className="text-muted-foreground">{crb.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
