import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import type { ConfigMapInfo } from "@/lib/tauriCommands";

interface ConfigMapListProps {
  configmaps: ConfigMapInfo[];
  clusterId: string;
  namespace: string;
}

export function ConfigMapList({ configmaps }: ConfigMapListProps) {

  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Data Keys</TableHead>
            <TableHead>Age</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {configmaps.length === 0 ? (
            <TableRow>
              <TableCell colSpan={5} className="text-center text-muted-foreground">
                No configmaps found
              </TableCell>
            </TableRow>
          ) : (
            configmaps.map((configmap) => (
              <TableRow key={configmap.name}>
                <TableCell className="font-medium">{configmap.name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{configmap.namespace}</TableCell>
                <TableCell className="text-sm">{configmap.data_keys}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{configmap.age}</TableCell>
                <TableCell className="text-right">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {}}
                    className="text-primary hover:text-primary hover:bg-primary/10"
                  >
                    View/Edit
                  </Button>
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
