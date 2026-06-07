import React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { DaemonSetInfo } from "@/lib/tauriCommands";

interface DaemonSetListProps {
  daemonsets: DaemonSetInfo[];
  clusterId: string;
  namespace: string;
}

export function DaemonSetList({ daemonsets, clusterId, namespace }: DaemonSetListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Desired</TableHead>
            <TableHead>Current</TableHead>
            <TableHead>Ready</TableHead>
            <TableHead>Up-to-date</TableHead>
            <TableHead>Available</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {daemonsets.length === 0 ? (
            <TableRow>
              <TableCell colSpan={7} className="text-center text-muted-foreground">
                No daemonsets found
              </TableCell>
            </TableRow>
          ) : (
            daemonsets.map((ds) => (
              <TableRow key={ds.name}>
                <TableCell className="font-medium">{ds.name}</TableCell>
                <TableCell>{ds.desired}</TableCell>
                <TableCell>{ds.current}</TableCell>
                <TableCell>{ds.ready}</TableCell>
                <TableCell>{ds.up_to_date}</TableCell>
                <TableCell>{ds.available}</TableCell>
                <TableCell className="text-muted-foreground">{ds.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
