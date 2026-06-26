import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { RuntimeClassInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface RuntimeClassListProps {
  items: RuntimeClassInfo[];
  clusterId: string;
  namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; rc: RuntimeClassInfo; yaml: string }
  | { type: "delete"; rc: RuntimeClassInfo }
  | null;

export function RuntimeClassList({ items, clusterId, onRefresh }: RuntimeClassListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (rc: RuntimeClassInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "runtimeclasses", "", rc.name);
      setActiveModal({ type: "edit", rc, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(clusterId, "runtimeclasses", "", activeModal.rc.name);
      setActiveModal(null);
      onRefresh?.();
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <>
      {actionError && (
        <p className="mb-2 text-sm text-destructive">{actionError}</p>
      )}
      <div className="overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Handler</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} className="text-center text-muted-foreground">
                  No runtime classes found
                </TableCell>
              </TableRow>
            ) : (
              items.map((rc) => (
                <TableRow key={rc.name}>
                  <TableCell className="font-medium">{rc.name}</TableCell>
                  <TableCell className="text-sm font-mono">{rc.handler}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{rc.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(rc),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", rc }),
                        },
                      ]}
                    />
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={clusterId}
          namespace=""
          resourceType="runtimeclasses"
          resourceName={activeModal.rc.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="RuntimeClass"
          resourceName={activeModal.rc.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
