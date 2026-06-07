import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { HorizontalPodAutoscalerInfo } from "@/lib/tauriCommands";

interface HPAListProps {
  hpas: HorizontalPodAutoscalerInfo[];
  _clusterId: string;
  _namespace: string;
}

export function HPAList({ hpas, _clusterId, _namespace }: HPAListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Min Replicas</TableHead>
            <TableHead>Max Replicas</TableHead>
            <TableHead>Current Replicas</TableHead>
            <TableHead>Desired Replicas</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {hpas.length === 0 ? (
            <TableRow>
              <TableCell colSpan={7} className="text-center text-muted-foreground">
                No HPAs found
              </TableCell>
            </TableRow>
          ) : (
            hpas.map((hpa) => (
              <TableRow key={`${hpa.name}-${hpa.namespace}`}>
                <TableCell className="font-medium">{hpa.name}</TableCell>
                <TableCell>{hpa.namespace}</TableCell>
                <TableCell>{hpa.min_replicas}</TableCell>
                <TableCell>{hpa.max_replicas}</TableCell>
                <TableCell>{hpa.current_replicas}</TableCell>
                <TableCell>{hpa.desired_replicas}</TableCell>
                <TableCell className="text-muted-foreground">{hpa.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
