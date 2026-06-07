import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { PersistentVolumeInfo } from "@/lib/tauriCommands";

interface PVListProps {
  pvs: PersistentVolumeInfo[];
  _clusterId: string;
}

export function PVList({ pvs, _clusterId }: PVListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Capacity</TableHead>
            <TableHead>Access Modes</TableHead>
            <TableHead>Reclaim Policy</TableHead>
            <TableHead>Storage Class</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {pvs.length === 0 ? (
            <TableRow>
              <TableCell colSpan={7} className="text-center text-muted-foreground">
                No PVs found
              </TableCell>
            </TableRow>
          ) : (
            pvs.map((pv) => (
              <TableRow key={pv.name}>
                <TableCell className="font-medium">{pv.name}</TableCell>
                <TableCell>{pv.status}</TableCell>
                <TableCell>{pv.capacity}</TableCell>
                <TableCell>{pv.access_modes.join(", ")}</TableCell>
                <TableCell>{pv.reclaim_policy}</TableCell>
                <TableCell>{pv.storage_class}</TableCell>
                <TableCell className="text-muted-foreground">{pv.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
