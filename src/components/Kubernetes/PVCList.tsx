import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { PersistentVolumeClaimInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface PVCListProps {
  pvcs: PersistentVolumeClaimInfo[];
  clusterId?: string;
  _clusterId?: string;
  namespace?: string;
  _namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; pvc: PersistentVolumeClaimInfo; yaml: string }
  | { type: "delete"; pvc: PersistentVolumeClaimInfo }
  | null;

export function PVCList({
  pvcs,
  clusterId,
  _clusterId,
  namespace,
  _namespace,
  onRefresh,
}: PVCListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const ns = namespace ?? _namespace ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (pvc: PersistentVolumeClaimInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "persistentvolumeclaims", ns, pvc.name);
      setActiveModal({ type: "edit", pvc, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(cid, "persistentvolumeclaims", ns, activeModal.pvc.name);
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
              <TableHead>Status</TableHead>
              <TableHead>Volume</TableHead>
              <TableHead>Capacity</TableHead>
              <TableHead>Access Modes</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {pvcs.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground">
                  No PVCs found
                </TableCell>
              </TableRow>
            ) : (
              pvcs.map((pvc) => (
                <TableRow key={`${pvc.name}-${pvc.namespace}`}>
                  <TableCell className="font-medium">{pvc.name}</TableCell>
                  <TableCell>{pvc.namespace}</TableCell>
                  <TableCell>{pvc.status}</TableCell>
                  <TableCell>{pvc.volume}</TableCell>
                  <TableCell>{pvc.capacity}</TableCell>
                  <TableCell>{pvc.access_modes.join(", ")}</TableCell>
                  <TableCell className="text-muted-foreground">{pvc.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(pvc),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", pvc }),
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
          clusterId={cid}
          namespace={ns}
          resourceType="persistentvolumeclaims"
          resourceName={activeModal.pvc.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="PVC"
          resourceName={activeModal.pvc.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
