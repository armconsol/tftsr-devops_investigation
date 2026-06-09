import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Scale, RotateCcw, Pencil, Trash2 } from "lucide-react";
import type { StatefulSetInfo } from "@/lib/tauriCommands";
import {
  scaleStatefulsetCmd,
  restartStatefulsetCmd,
  deleteResourceCmd,
  getResourceYamlCmd,
} from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { ScaleModal } from "./ScaleModal";
import { EditResourceModal } from "./EditResourceModal";

interface StatefulSetListProps {
  statefulsets: StatefulSetInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "scale"; ss: StatefulSetInfo }
  | { type: "restart"; ss: StatefulSetInfo }
  | { type: "edit"; ss: StatefulSetInfo; yaml: string }
  | { type: "delete"; ss: StatefulSetInfo }
  | null;

export function StatefulSetList({ statefulsets, clusterId, namespace, onRefresh }: StatefulSetListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isActing, setIsActing] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (ss: StatefulSetInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "statefulsets", namespace, ss.name);
      setActiveModal({ type: "edit", ss, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRestart = async () => {
    if (activeModal?.type !== "restart") return;
    setIsActing(true);
    try {
      await restartStatefulsetCmd(clusterId, namespace, activeModal.ss.name);
      setActiveModal(null);
      onRefresh?.();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsActing(false);
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsActing(true);
    try {
      await deleteResourceCmd(clusterId, "statefulsets", namespace, activeModal.ss.name);
      setActiveModal(null);
      onRefresh?.();
    } finally {
      setIsActing(false);
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
              <TableHead>Ready</TableHead>
              <TableHead>Replicas</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {statefulsets.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  No statefulsets found
                </TableCell>
              </TableRow>
            ) : (
              statefulsets.map((ss) => (
                <TableRow key={ss.name}>
                  <TableCell className="font-medium">{ss.name}</TableCell>
                  <TableCell>{ss.ready}</TableCell>
                  <TableCell>{ss.replicas}</TableCell>
                  <TableCell className="text-muted-foreground">{ss.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Scale",
                          icon: Scale,
                          onClick: () => setActiveModal({ type: "scale", ss }),
                        },
                        {
                          label: "Restart",
                          icon: RotateCcw,
                          onClick: () => setActiveModal({ type: "restart", ss }),
                        },
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(ss),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", ss }),
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

      {activeModal?.type === "scale" && (
        <ScaleModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="StatefulSet"
          resourceName={activeModal.ss.name}
          currentReplicas={activeModal.ss.replicas}
          onScale={(replicas) =>
            scaleStatefulsetCmd(clusterId, namespace, activeModal.ss.name, replicas).then(() => {
              setActiveModal(null);
              onRefresh?.();
            })
          }
        />
      )}

      {activeModal?.type === "restart" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="StatefulSet"
          resourceName={activeModal.ss.name}
          isLoading={isActing}
          onConfirm={handleRestart}
          variant="delete"
        />
      )}

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={clusterId}
          namespace={namespace}
          resourceType="statefulsets"
          resourceName={activeModal.ss.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="StatefulSet"
          resourceName={activeModal.ss.name}
          isLoading={isActing}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
