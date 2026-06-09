import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { PodDisruptionBudgetInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface PodDisruptionBudgetListProps {
  items: PodDisruptionBudgetInfo[];
  clusterId: string;
  namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; pdb: PodDisruptionBudgetInfo; yaml: string }
  | { type: "delete"; pdb: PodDisruptionBudgetInfo }
  | null;

export function PodDisruptionBudgetList({ items, clusterId, onRefresh }: PodDisruptionBudgetListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (pdb: PodDisruptionBudgetInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "poddisruptionbudgets", pdb.namespace, pdb.name);
      setActiveModal({ type: "edit", pdb, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(clusterId, "poddisruptionbudgets", activeModal.pdb.namespace, activeModal.pdb.name);
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
              <TableHead>Namespace</TableHead>
              <TableHead>Min Available</TableHead>
              <TableHead>Max Unavailable</TableHead>
              <TableHead>Disruptions Allowed</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No pod disruption budgets found
                </TableCell>
              </TableRow>
            ) : (
              items.map((pdb) => (
                <TableRow key={`${pdb.name}-${pdb.namespace}`}>
                  <TableCell className="font-medium">{pdb.name}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{pdb.namespace}</TableCell>
                  <TableCell className="text-sm">{pdb.min_available}</TableCell>
                  <TableCell className="text-sm">{pdb.max_unavailable}</TableCell>
                  <TableCell className="text-sm">{pdb.disruptions_allowed}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{pdb.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(pdb),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", pdb }),
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
          namespace={activeModal.pdb.namespace}
          resourceType="poddisruptionbudgets"
          resourceName={activeModal.pdb.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="PodDisruptionBudget"
          resourceName={activeModal.pdb.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
