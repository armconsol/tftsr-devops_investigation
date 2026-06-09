import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Button } from "@/components/ui";
import { Scale, Pencil, Trash2, FileText, Settings } from "lucide-react";
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
import { useColumnConfig } from "@/hooks/useColumnConfig";
import { DEFAULT_COLUMNS } from "@/config/defaultColumns";
import { ColumnConfigModal } from "@/components/tables/ColumnConfigModal";

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
  const [showColumnConfig, setShowColumnConfig] = useState(false);

  // Configurable columns
  const columnConfig = useColumnConfig("replicationcontrollers", DEFAULT_COLUMNS.replicationcontrollers);
  const { isColumnVisible } = columnConfig;

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
      <div className="flex items-center justify-between mb-2">
        <div className="text-sm text-muted-foreground">
          {items.length} {items.length === 1 ? "replication controller" : "replication controllers"}
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setShowColumnConfig(true)}
          className="flex items-center gap-1"
        >
          <Settings className="h-3.5 w-3.5" />
          Columns
        </Button>
      </div>
      <div className="overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              {isColumnVisible("name") && <TableHead>Name</TableHead>}
              {isColumnVisible("namespace") && <TableHead>Namespace</TableHead>}
              {isColumnVisible("desired") && <TableHead>Desired</TableHead>}
              {isColumnVisible("current") && <TableHead>Current</TableHead>}
              {isColumnVisible("ready") && <TableHead>Ready</TableHead>}
              {isColumnVisible("age") && <TableHead>Age</TableHead>}
              {isColumnVisible("actions") && <TableHead className="text-right">Actions</TableHead>}
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
                  {isColumnVisible("name") && (
                    <TableCell className="font-medium">{rc.name}</TableCell>
                  )}
                  {isColumnVisible("namespace") && (
                    <TableCell className="text-muted-foreground">{rc.namespace}</TableCell>
                  )}
                  {isColumnVisible("desired") && <TableCell>{rc.desired}</TableCell>}
                  {isColumnVisible("current") && <TableCell>{rc.current}</TableCell>}
                  {isColumnVisible("ready") && <TableCell>{rc.ready}</TableCell>}
                  {isColumnVisible("age") && (
                    <TableCell className="text-muted-foreground">{rc.age}</TableCell>
                  )}
                  {isColumnVisible("actions") && (
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
                  )}
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

      <ColumnConfigModal
        open={showColumnConfig}
        onOpenChange={setShowColumnConfig}
        resourceType="ReplicationControllers"
        columnConfig={columnConfig}
        columnLabels={{
          name: "Name",
          namespace: "Namespace",
          desired: "Desired",
          current: "Current",
          ready: "Ready",
          age: "Age",
          actions: "Actions",
        }}
      />
    </>
  );
}
