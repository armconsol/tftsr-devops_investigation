import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Badge, Button } from "@/components/ui";
import { Pencil } from "lucide-react";
import type { NamespaceResourceInfo } from "@/lib/tauriCommands";
import { getResourceYamlCmd } from "@/lib/tauriCommands";
import { EditResourceModal } from "./EditResourceModal";

interface NamespaceListProps {
  items: NamespaceResourceInfo[];
  clusterId: string;
  namespace?: string;
}

function statusVariant(status: string): "success" | "destructive" | "secondary" {
  if (status === "Active") return "success";
  if (status === "Terminating") return "destructive";
  return "secondary";
}

export function NamespaceList({ items, clusterId }: NamespaceListProps) {
  const [editState, setEditState] = useState<{
    open: boolean;
    name: string;
    yaml: string;
  } | null>(null);
  const [editError, setEditError] = useState<string | null>(null);

  const openEdit = async (ns: NamespaceResourceInfo) => {
    setEditError(null);
    try {
      // Namespaces are cluster-scoped — pass empty string for namespace param
      const yaml = await getResourceYamlCmd(clusterId, "namespaces", "", ns.name);
      setEditState({ open: true, name: ns.name, yaml });
    } catch (err) {
      setEditError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <div className="overflow-x-auto">
      {editError && (
        <div className="mb-2 rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {editError}
        </div>
      )}

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Age</TableHead>
            <TableHead className="w-16 text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.length === 0 ? (
            <TableRow>
              <TableCell colSpan={4} className="text-center text-muted-foreground">
                No namespaces found
              </TableCell>
            </TableRow>
          ) : (
            items.map((ns) => (
              <TableRow key={ns.name}>
                <TableCell className="font-medium">{ns.name}</TableCell>
                <TableCell className="text-sm">
                  <Badge variant={statusVariant(ns.status)}>{ns.status}</Badge>
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">{ns.age}</TableCell>
                <TableCell className="text-right">
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-7 w-7 p-0"
                    title="Edit YAML"
                    onClick={() => void openEdit(ns)}
                  >
                    <Pencil className="h-3.5 w-3.5" />
                    <span className="sr-only">Edit</span>
                  </Button>
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>

      {editState && (
        <EditResourceModal
          isOpen={editState.open}
          clusterId={clusterId}
          namespace=""
          resourceType="namespaces"
          resourceName={editState.name}
          initialYaml={editState.yaml}
          onClose={() => setEditState(null)}
        />
      )}
    </div>
  );
}
