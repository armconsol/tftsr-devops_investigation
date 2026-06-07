import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { RoleBindingInfo } from "@/lib/tauriCommands";

interface RoleBindingListProps {
  roleBindings: RoleBindingInfo[];
  _clusterId: string;
  _namespace: string;
}

export function RoleBindingList({ roleBindings, _clusterId, _namespace }: RoleBindingListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Role</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {roleBindings.length === 0 ? (
            <TableRow>
              <TableCell colSpan={4} className="text-center text-muted-foreground">
                No role bindings found
              </TableCell>
            </TableRow>
          ) : (
            roleBindings.map((rb) => (
              <TableRow key={`${rb.name}-${rb.namespace}`}>
                <TableCell className="font-medium">{rb.name}</TableCell>
                <TableCell>{rb.namespace}</TableCell>
                <TableCell>{rb.role}</TableCell>
                <TableCell className="text-muted-foreground">{rb.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
