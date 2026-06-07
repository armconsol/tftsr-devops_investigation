import React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { StatefulSetInfo } from "@/lib/tauriCommands";

interface StatefulSetListProps {
  statefulsets: StatefulSetInfo[];
  clusterId: string;
  namespace: string;
}

export function StatefulSetList({ statefulsets, clusterId, namespace }: StatefulSetListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Ready</TableHead>
            <TableHead>Replicas</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {statefulsets.length === 0 ? (
            <TableRow>
              <TableCell colSpan={4} className="text-center text-muted-foreground">
                No statefulsets found
              </TableCell>
            </TableRow>
          ) : (
            statefulsets.map((ss) => (
              <TableRow key={ss.name}>
                <TableCell className="font-medium">{ss.name}</TableCell>
                <TableCell>{ss.ready}</TableCell>
                <TableCell>{ss.replicas}</TableCell>
                <TableCell className="text-muted-foreground">{ss.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
