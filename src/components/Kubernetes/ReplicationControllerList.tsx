import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Scale, Pencil, Trash2, FileText } from "lucide-react";
import type { ReplicationControllerInfo } from "@/lib/tauriCommands";
import {
  scaleReplicationcontrollerCmd,
  deleteResourceCmd,
  getResourceYamlCmd,
} from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { ScaleModal } from "./ScaleModal";
import { EditResourceModal } from "./EditResourceModal";
import { WorkloadLogsModal } from "./WorkloadLogsModal";

interface ReplicationControllerListProps {
  items: ReplicationControllerInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "scale"; rc: ReplicationControllerInfo }
  | { type: "logs"; rc: ReplicationControllerInfo }
  | { type: "edit"; rc: ReplicationControllerInfo; yaml: string }
  | { type: "delete"; rc: ReplicationControllerInfo }
  | null;

export function ReplicationControllerList({
  items,
  clusterId,
  namespace: _namespace,
  onRefresh,
}: ReplicationControllerListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isActing, setIsActing] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (rc: ReplicationControllerInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "replicationcontrollers", rc.namespace, rc.name);
      setActiveModal({ type: "edit", rc, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsActing(true);
    try {
      await deleteResourceCmd(clusterId, "replicationcontrollers", activeModal.rc.namespace, activeModal.rc.name);
      setActiveModal(null);
      onRefresh?.();
    } finally {
      setIsActing(false);
    }
  };

  // Convert "X/Y" string to number (for current replicas)
  const getDesiredReplicas = (rc: ReplicationControllerInfo): number => {
    return rc.desired;
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
              <TableHead>Desired</TableHead>
              <TableHead>Current</TableHead>
              <TableHead>Ready</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No replication controllers found
                </TableCell>
              </TableRow>
            ) : (
              items.map((rc) => (
                <TableRow key={`${rc.name}-${rc.namespace}`}>
                  <TableCell className="font-medium">{rc.name}</TableCell>
                  <TableCell className="text-muted-foreground">{rc.namespace}</TableCell>
                  <TableCell>{rc.desired}</TableCell>
                  <TableCell>{rc.current}</TableCell>
                  <TableCell>{rc.ready}</TableCell>
                  <TableCell className="text-muted-foreground">{rc.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Scale",
                          icon: Scale,
                          onClick: () => setActiveModal({ type: "scale", rc }),
                        },
                        {
                          label: "Logs",
                          icon: FileText,
                          onClick: () => setActiveModal({ type: "logs", rc }),
                        },
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

      {activeModal?.type === "logs" && (
        <WorkloadLogsModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          clusterId={clusterId}
          namespace={activeModal.rc.namespace}
          workloadType="replicationcontroller"
          workloadName={activeModal.rc.name}
          labels={{}}
        />
      )}

      {activeModal?.type === "scale" && (
        <ScaleModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="ReplicationController"
          resourceName={activeModal.rc.name}
          currentReplicas={getDesiredReplicas(activeModal.rc)}
          onScale={(replicas) =>
            scaleReplicationcontrollerCmd(clusterId, activeModal.rc.namespace, activeModal.rc.name, replicas).then(() => {
              setActiveModal(null);
              onRefresh?.();
            })
          }
        />
      )}

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={clusterId}
          namespace={activeModal.rc.namespace}
          resourceType="replicationcontrollers"
          resourceName={activeModal.rc.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="ReplicationController"
          resourceName={activeModal.rc.name}
          isLoading={isActing}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
