import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { ClusterRoleBindingInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface ClusterRoleBindingListProps {
  clusterRoleBindings: ClusterRoleBindingInfo[];
  clusterId?: string;
  _clusterId?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; crb: ClusterRoleBindingInfo; yaml: string }
  | { type: "delete"; crb: ClusterRoleBindingInfo }
  | null;

export function ClusterRoleBindingList({
  clusterRoleBindings,
  clusterId,
  _clusterId,
  onRefresh,
}: ClusterRoleBindingListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (crb: ClusterRoleBindingInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "clusterrolebindings", "", crb.name);
      setActiveModal({ type: "edit", crb, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(cid, "clusterrolebindings", "", activeModal.crb.name);
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
              <TableHead>Cluster Role</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {clusterRoleBindings.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} className="text-center text-muted-foreground">
                  No cluster role bindings found
                </TableCell>
              </TableRow>
            ) : (
              clusterRoleBindings.map((crb) => (
                <TableRow key={crb.name}>
                  <TableCell className="font-medium">{crb.name}</TableCell>
                  <TableCell>{crb.cluster_role}</TableCell>
                  <TableCell className="text-muted-foreground">{crb.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(crb),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", crb }),
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
          resourceType="clusterrolebindings"
          resourceName={activeModal.crb.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="ClusterRoleBinding"
          resourceName={activeModal.crb.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
