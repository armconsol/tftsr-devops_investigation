import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { ReplicaSetInfo } from "@/lib/tauriCommands";

interface ReplicaSetListProps {
  replicaSets: ReplicaSetInfo[];
  _clusterId: string;
  _namespace: string;
}

export function ReplicaSetList({ replicaSets, _clusterId, _namespace }: ReplicaSetListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Replicas</TableHead>
            <TableHead>Ready</TableHead>
            <TableHead>Age</TableHead>
            <TableHead>Labels</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {replicaSets.length === 0 ? (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No replica sets found
              </TableCell>
            </TableRow>
          ) : (
            replicaSets.map((replicaSet) => (
              <TableRow key={`${replicaSet.name}-${replicaSet.namespace}`}>
                <TableCell className="font-medium">{replicaSet.name}</TableCell>
                <TableCell>{replicaSet.namespace}</TableCell>
                <TableCell>{replicaSet.replicas}</TableCell>
                <TableCell>{replicaSet.ready}</TableCell>
                <TableCell className="text-muted-foreground">{replicaSet.age}</TableCell>
                <TableCell>
                  {Object.entries(replicaSet.labels)
                    .map(([k, v]) => `${k}=${v}`)
                    .join(", ")}
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
