import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { StorageClassInfo } from "@/lib/tauriCommands";

interface StorageClassListProps {
  storageclasses: StorageClassInfo[];
  clusterId: string;
  namespace: string;
}

export function StorageClassList({ storageclasses }: StorageClassListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Provisioner</TableHead>
            <TableHead>Reclaim Policy</TableHead>
            <TableHead>Volume Binding Mode</TableHead>
            <TableHead>Expand</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {storageclasses.length === 0 ? (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No storage classes found
              </TableCell>
            </TableRow>
          ) : (
            storageclasses.map((sc) => (
              <TableRow key={sc.name}>
                <TableCell className="font-medium">{sc.name}</TableCell>
                <TableCell className="text-sm font-mono">{sc.provisioner}</TableCell>
                <TableCell className="text-sm">{sc.reclaim_policy}</TableCell>
                <TableCell className="text-sm">{sc.volume_binding_mode}</TableCell>
                <TableCell className="text-sm">{sc.allow_volume_expansion ? "Yes" : "No"}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{sc.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
