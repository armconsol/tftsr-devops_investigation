import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { RotateCcw, Pencil, Trash2, FileText } from "lucide-react";
import type { DaemonSetInfo } from "@/lib/tauriCommands";
import {
  restartDaemonsetCmd,
  deleteResourceCmd,
  getResourceYamlCmd,
} from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";
import { WorkloadLogsModal } from "./WorkloadLogsModal";

interface DaemonSetListProps {
  daemonsets: DaemonSetInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "restart"; ds: DaemonSetInfo }
  | { type: "logs"; ds: DaemonSetInfo }
  | { type: "edit"; ds: DaemonSetInfo; yaml: string }
  | { type: "delete"; ds: DaemonSetInfo }
  | null;

export function DaemonSetList({ daemonsets, clusterId, namespace: _namespace, onRefresh }: DaemonSetListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isActing, setIsActing] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (ds: DaemonSetInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "daemonsets", ds.namespace, ds.name);
      setActiveModal({ type: "edit", ds, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRestart = async () => {
    if (activeModal?.type !== "restart") return;
    setIsActing(true);
    try {
      await restartDaemonsetCmd(clusterId, activeModal.ds.namespace, activeModal.ds.name);
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
      await deleteResourceCmd(clusterId, "daemonsets", activeModal.ds.namespace, activeModal.ds.name);
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
              <TableHead>Desired</TableHead>
              <TableHead>Current</TableHead>
              <TableHead>Ready</TableHead>
              <TableHead>Up-to-date</TableHead>
              <TableHead>Available</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {daemonsets.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground">
                  No daemonsets found
                </TableCell>
              </TableRow>
            ) : (
              daemonsets.map((ds) => (
                <TableRow key={ds.name}>
                  <TableCell className="font-medium">{ds.name}</TableCell>
                  <TableCell>{ds.desired}</TableCell>
                  <TableCell>{ds.current}</TableCell>
                  <TableCell>{ds.ready}</TableCell>
                  <TableCell>{ds.up_to_date}</TableCell>
                  <TableCell>{ds.available}</TableCell>
                  <TableCell className="text-muted-foreground">{ds.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Restart",
                          icon: RotateCcw,
                          onClick: () => setActiveModal({ type: "restart", ds }),
                        },
                        {
                          label: "Logs",
                          icon: FileText,
                          onClick: () => setActiveModal({ type: "logs", ds }),
                        },
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(ds),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", ds }),
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
          namespace={activeModal.ds.namespace}
          workloadType="daemonset"
          workloadName={activeModal.ds.name}
          labels={activeModal.ds.labels}
        />
      )}

      {activeModal?.type === "restart" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="DaemonSet"
          resourceName={activeModal.ds.name}
          isLoading={isActing}
          onConfirm={handleRestart}
          variant="delete"
        />
      )}

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={clusterId}
          namespace={activeModal.ds.namespace}
          resourceType="daemonsets"
          resourceName={activeModal.ds.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="DaemonSet"
          resourceName={activeModal.ds.name}
          isLoading={isActing}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
