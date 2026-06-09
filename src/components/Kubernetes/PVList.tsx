import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { PersistentVolumeInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface PVListProps {
  pvs: PersistentVolumeInfo[];
  clusterId?: string;
  _clusterId?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; pv: PersistentVolumeInfo; yaml: string }
  | { type: "delete"; pv: PersistentVolumeInfo }
  | null;

export function PVList({ pvs, clusterId, _clusterId, onRefresh }: PVListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (pv: PersistentVolumeInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "persistentvolumes", "", pv.name);
      setActiveModal({ type: "edit", pv, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(cid, "persistentvolumes", "", activeModal.pv.name);
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
              <TableHead>Status</TableHead>
              <TableHead>Capacity</TableHead>
              <TableHead>Access Modes</TableHead>
              <TableHead>Reclaim Policy</TableHead>
              <TableHead>Storage Class</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {pvs.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground">
                  No PVs found
                </TableCell>
              </TableRow>
            ) : (
              pvs.map((pv) => (
                <TableRow key={pv.name}>
                  <TableCell className="font-medium">{pv.name}</TableCell>
                  <TableCell>{pv.status}</TableCell>
                  <TableCell>{pv.capacity}</TableCell>
                  <TableCell>{pv.access_modes.join(", ")}</TableCell>
                  <TableCell>{pv.reclaim_policy}</TableCell>
                  <TableCell>{pv.storage_class}</TableCell>
                  <TableCell className="text-muted-foreground">{pv.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(pv),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", pv }),
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
          namespace=""
          resourceType="persistentvolumes"
          resourceName={activeModal.pv.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="PersistentVolume"
          resourceName={activeModal.pv.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
