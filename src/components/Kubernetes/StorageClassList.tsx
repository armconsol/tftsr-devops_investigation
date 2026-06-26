import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { StorageClassInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface StorageClassListProps {
  storageclasses: StorageClassInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; sc: StorageClassInfo; yaml: string }
  | { type: "delete"; sc: StorageClassInfo }
  | null;

export function StorageClassList({ storageclasses, clusterId, onRefresh }: StorageClassListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (sc: StorageClassInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "storageclasses", "", sc.name);
      setActiveModal({ type: "edit", sc, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(clusterId, "storageclasses", "", activeModal.sc.name);
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
              <TableHead>Provisioner</TableHead>
              <TableHead>Reclaim Policy</TableHead>
              <TableHead>Volume Binding Mode</TableHead>
              <TableHead>Expand</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {storageclasses.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No storage classes found
                </TableCell>
              </TableRow>
            ) : (
              storageclasses.map((sc) => (
                <TableRow key={sc.name}>
                  <TableCell className="font-medium">{sc.name}</TableCell>
                  <TableCell className="text-sm font-mono">{sc.provisioner}</TableCell>
                  <TableCell className="text-sm">{sc.reclaim_policy}</TableCell>
                  <TableCell className="text-sm">{sc.volume_binding_mode}</TableCell>
                  <TableCell className="text-sm">{sc.allow_volume_expansion ? "Yes" : "No"}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{sc.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(sc),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", sc }),
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
          resourceType="storageclasses"
          resourceName={activeModal.sc.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="StorageClass"
          resourceName={activeModal.sc.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
