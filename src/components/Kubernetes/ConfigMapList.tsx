import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { ConfigMapInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface ConfigMapListProps {
  configmaps: ConfigMapInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; cm: ConfigMapInfo; yaml: string }
  | { type: "delete"; cm: ConfigMapInfo }
  | null;

export function ConfigMapList({ configmaps, clusterId, namespace: _namespace, onRefresh }: ConfigMapListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (cm: ConfigMapInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "configmaps", cm.namespace, cm.name);
      setActiveModal({ type: "edit", cm, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(clusterId, "configmaps", activeModal.cm.namespace, activeModal.cm.name);
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
              configmaps.map((cm) => (
                <TableRow key={cm.name}>
                  <TableCell className="font-medium">{cm.name}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{cm.namespace}</TableCell>
                  <TableCell className="text-sm">{cm.data_keys}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{cm.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(cm),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", cm }),
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
          namespace={activeModal.cm.namespace}
          resourceType="configmaps"
          resourceName={activeModal.cm.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="ConfigMap"
          resourceName={activeModal.cm.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
