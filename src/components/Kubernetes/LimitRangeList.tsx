import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { LimitRangeInfo } from "@/lib/tauriCommands";

interface LimitRangeListProps {
  limitranges: LimitRangeInfo[];
  clusterId: string;
  namespace: string;
}

export function LimitRangeList({ limitranges }: LimitRangeListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Limits</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {limitranges.length === 0 ? (
            <TableRow>
              <TableCell colSpan={4} className="text-center text-muted-foreground">
                No limit ranges found
              </TableCell>
            </TableRow>
          ) : (
            limitranges.map((lr) => (
              <TableRow key={`${lr.name}-${lr.namespace}`}>
                <TableCell className="font-medium">{lr.name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{lr.namespace}</TableCell>
                <TableCell className="text-sm">{lr.limit_count}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{lr.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
