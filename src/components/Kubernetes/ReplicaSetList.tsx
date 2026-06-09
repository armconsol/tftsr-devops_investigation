import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Scale, Pencil, Trash2, FileText } from "lucide-react";
import type { ReplicaSetInfo } from "@/lib/tauriCommands";
import {
  scaleReplicasetCmd,
  deleteResourceCmd,
  getResourceYamlCmd,
} from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { ScaleModal } from "./ScaleModal";
import { EditResourceModal } from "./EditResourceModal";
import { WorkloadLogsModal } from "./WorkloadLogsModal";

interface ReplicaSetListProps {
  replicaSets: ReplicaSetInfo[];
  clusterId?: string;
  _clusterId?: string;
  namespace?: string;
  _namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "scale"; rs: ReplicaSetInfo }
  | { type: "logs"; rs: ReplicaSetInfo }
  | { type: "edit"; rs: ReplicaSetInfo; yaml: string }
  | { type: "delete"; rs: ReplicaSetInfo }
  | null;

export function ReplicaSetList({
  replicaSets,
  clusterId,
  _clusterId,
  onRefresh,
}: ReplicaSetListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isActing, setIsActing] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (rs: ReplicaSetInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "replicasets", rs.namespace, rs.name);
      setActiveModal({ type: "edit", rs, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsActing(true);
    try {
      await deleteResourceCmd(cid, "replicasets", activeModal.rs.namespace, activeModal.rs.name);
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
              <TableHead>Namespace</TableHead>
              <TableHead>Replicas</TableHead>
              <TableHead>Ready</TableHead>
              <TableHead>Age</TableHead>
              <TableHead>Labels</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {replicaSets.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No replica sets found
                </TableCell>
              </TableRow>
            ) : (
              replicaSets.map((rs) => (
                <TableRow key={`${rs.name}-${rs.namespace}`}>
                  <TableCell className="font-medium">{rs.name}</TableCell>
                  <TableCell>{rs.namespace}</TableCell>
                  <TableCell>{rs.replicas}</TableCell>
                  <TableCell>{rs.ready}</TableCell>
                  <TableCell className="text-muted-foreground">{rs.age}</TableCell>
                  <TableCell>
                    {Object.entries(rs.labels)
                      .map(([k, v]) => `${k}=${v}`)
                      .join(", ")}
                  </TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Scale",
                          icon: Scale,
                          onClick: () => setActiveModal({ type: "scale", rs }),
                        },
                        {
                          label: "Logs",
                          icon: FileText,
                          onClick: () => setActiveModal({ type: "logs", rs }),
                        },
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(rs),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", rs }),
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
          clusterId={cid}
          namespace={activeModal.rs.namespace}
          workloadType="replicaset"
          workloadName={activeModal.rs.name}
          labels={activeModal.rs.labels}
        />
      )}

      {activeModal?.type === "scale" && (
        <ScaleModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="ReplicaSet"
          resourceName={activeModal.rs.name}
          currentReplicas={activeModal.rs.replicas}
          onScale={(replicas) =>
            scaleReplicasetCmd(cid, activeModal.rs.namespace, activeModal.rs.name, replicas).then(() => {
              setActiveModal(null);
              onRefresh?.();
            })
          }
        />
      )}

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={cid}
          namespace={activeModal.rs.namespace}
          resourceType="replicasets"
          resourceName={activeModal.rs.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="ReplicaSet"
          resourceName={activeModal.rs.name}
          isLoading={isActing}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
