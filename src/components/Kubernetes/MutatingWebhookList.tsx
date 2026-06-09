import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { WebhookConfigInfo } from "@/lib/tauriCommands";

interface MutatingWebhookListProps {
  items: WebhookConfigInfo[];
  clusterId: string;
  namespace?: string;
}

export function MutatingWebhookList({ items }: MutatingWebhookListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Webhooks</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={3} className="text-center text-muted-foreground">
                No mutating webhook configurations found
              </TableCell>
            </TableRow>
          ) : (
            items.map((wh) => (
              <TableRow key={wh.name}>
                <TableCell className="font-medium">{wh.name}</TableCell>
                <TableCell className="text-sm">{wh.webhooks}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{wh.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
