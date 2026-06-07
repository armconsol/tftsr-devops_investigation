import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { PersistentVolumeClaimInfo } from "@/lib/tauriCommands";

interface PVCListProps {
  pvcs: PersistentVolumeClaimInfo[];
  _clusterId: string;
  _namespace: string;
}

export function PVCList({ pvcs, _clusterId, _namespace }: PVCListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Volume</TableHead>
            <TableHead>Capacity</TableHead>
            <TableHead>Access Modes</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {pvcs.length === 0 ? (
            <TableRow>
              <TableCell colSpan={7} className="text-center text-muted-foreground">
                No PVCs found
              </TableCell>
            </TableRow>
          ) : (
            pvcs.map((pvc) => (
              <TableRow key={`${pvc.name}-${pvc.namespace}`}>
                <TableCell className="font-medium">{pvc.name}</TableCell>
                <TableCell>{pvc.namespace}</TableCell>
                <TableCell>{pvc.status}</TableCell>
                <TableCell>{pvc.volume}</TableCell>
                <TableCell>{pvc.capacity}</TableCell>
                <TableCell>{pvc.access_modes.join(", ")}</TableCell>
                <TableCell className="text-muted-foreground">{pvc.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
