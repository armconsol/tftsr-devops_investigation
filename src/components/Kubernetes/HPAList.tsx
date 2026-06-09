import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { HorizontalPodAutoscalerInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface HPAListProps {
  hpas: HorizontalPodAutoscalerInfo[];
  clusterId?: string;
  _clusterId?: string;
  namespace?: string;
  _namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; hpa: HorizontalPodAutoscalerInfo; yaml: string }
  | { type: "delete"; hpa: HorizontalPodAutoscalerInfo }
  | null;

export function HPAList({
  hpas,
  clusterId,
  _clusterId,
  namespace,
  _namespace,
  onRefresh,
}: HPAListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const ns = namespace ?? _namespace ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (hpa: HorizontalPodAutoscalerInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "horizontalpodautoscalers", ns, hpa.name);
      setActiveModal({ type: "edit", hpa, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(cid, "horizontalpodautoscalers", ns, activeModal.hpa.name);
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
              <TableHead>Min Replicas</TableHead>
              <TableHead>Max Replicas</TableHead>
              <TableHead>Current Replicas</TableHead>
              <TableHead>Desired Replicas</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {hpas.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground">
                  No HPAs found
                </TableCell>
              </TableRow>
            ) : (
              hpas.map((hpa) => (
                <TableRow key={`${hpa.name}-${hpa.namespace}`}>
                  <TableCell className="font-medium">{hpa.name}</TableCell>
                  <TableCell>{hpa.namespace}</TableCell>
                  <TableCell>{hpa.min_replicas}</TableCell>
                  <TableCell>{hpa.max_replicas}</TableCell>
                  <TableCell>{hpa.current_replicas}</TableCell>
                  <TableCell>{hpa.desired_replicas}</TableCell>
                  <TableCell className="text-muted-foreground">{hpa.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(hpa),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", hpa }),
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
          resourceType="horizontalpodautoscalers"
          resourceName={activeModal.hpa.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="HPA"
          resourceName={activeModal.hpa.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
