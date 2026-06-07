import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { ResourceQuotaInfo } from "@/lib/tauriCommands";

interface ResourceQuotaListProps {
  resourcequotas: ResourceQuotaInfo[];
  clusterId: string;
  namespace: string;
}

export function ResourceQuotaList({ resourcequotas }: ResourceQuotaListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>CPU Req</TableHead>
            <TableHead>Mem Req</TableHead>
            <TableHead>CPU Limit</TableHead>
            <TableHead>Mem Limit</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {resourcequotas.length === 0 ? (
            <TableRow>
              <TableCell colSpan={7} className="text-center text-muted-foreground">
                No resource quotas found
              </TableCell>
            </TableRow>
          ) : (
            resourcequotas.map((rq) => (
              <TableRow key={`${rq.name}-${rq.namespace}`}>
                <TableCell className="font-medium">{rq.name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{rq.namespace}</TableCell>
                <TableCell className="text-sm font-mono">{rq.request_cpu || "—"}</TableCell>
                <TableCell className="text-sm font-mono">{rq.request_memory || "—"}</TableCell>
                <TableCell className="text-sm font-mono">{rq.limit_cpu || "—"}</TableCell>
                <TableCell className="text-sm font-mono">{rq.limit_memory || "—"}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{rq.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
