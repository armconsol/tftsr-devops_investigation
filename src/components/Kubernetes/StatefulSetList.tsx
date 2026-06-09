import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Button } from "@/components/ui";
import { Scale, RotateCcw, Pencil, Trash2, FileText, Settings } from "lucide-react";
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
import { WorkloadLogsModal } from "./WorkloadLogsModal";
import { useColumnConfig } from "@/hooks/useColumnConfig";
import { DEFAULT_COLUMNS } from "@/config/defaultColumns";
import { ColumnConfigModal } from "@/components/tables/ColumnConfigModal";

interface StatefulSetListProps {
  statefulsets: StatefulSetInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "scale"; ss: StatefulSetInfo }
  | { type: "restart"; ss: StatefulSetInfo }
  | { type: "logs"; ss: StatefulSetInfo }
  | { type: "edit"; ss: StatefulSetInfo; yaml: string }
  | { type: "delete"; ss: StatefulSetInfo }
  | null;

export function StatefulSetList({ statefulsets, clusterId, namespace: _namespace, onRefresh }: StatefulSetListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isActing, setIsActing] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [showColumnConfig, setShowColumnConfig] = useState(false);

  // Configurable columns
  const columnConfig = useColumnConfig("statefulsets", DEFAULT_COLUMNS.statefulsets);
  const { isColumnVisible } = columnConfig;

  const openEdit = async (ss: StatefulSetInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "statefulsets", ss.namespace, ss.name);
      setActiveModal({ type: "edit", ss, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRestart = async () => {
    if (activeModal?.type !== "restart") return;
    setIsActing(true);
    try {
      await restartStatefulsetCmd(clusterId, activeModal.ss.namespace, activeModal.ss.name);
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
      await deleteResourceCmd(clusterId, "statefulsets", activeModal.ss.namespace, activeModal.ss.name);
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
      <div className="flex items-center justify-between mb-2">
        <div className="text-sm text-muted-foreground">
          {statefulsets.length} {statefulsets.length === 1 ? "statefulset" : "statefulsets"}
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
              {isColumnVisible("ready") && <TableHead>Ready</TableHead>}
              {isColumnVisible("replicas") && <TableHead>Replicas</TableHead>}
              {isColumnVisible("age") && <TableHead>Age</TableHead>}
              {isColumnVisible("actions") && <TableHead className="text-right">Actions</TableHead>}
            </TableRow>
          </TableHeader>
          <TableBody>
            {statefulsets.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground">
                  No statefulsets found
                </TableCell>
              </TableRow>
            ) : (
              statefulsets.map((ss) => (
                <TableRow key={ss.name}>
                  {isColumnVisible("name") && (
                    <TableCell className="font-medium">{ss.name}</TableCell>
                  )}
                  {isColumnVisible("namespace") && (
                    <TableCell className="text-muted-foreground">{ss.namespace}</TableCell>
                  )}
                  {isColumnVisible("ready") && <TableCell>{ss.ready}</TableCell>}
                  {isColumnVisible("replicas") && <TableCell>{ss.replicas}</TableCell>}
                  {isColumnVisible("age") && (
                    <TableCell className="text-muted-foreground">{ss.age}</TableCell>
                  )}
                  {isColumnVisible("actions") && (
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
                          label: "Logs",
                          icon: FileText,
                          onClick: () => setActiveModal({ type: "logs", ss }),
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
          namespace={activeModal.ss.namespace}
          workloadType="statefulset"
          workloadName={activeModal.ss.name}
          labels={activeModal.ss.labels}
        />
      )}

      {activeModal?.type === "scale" && (
        <ScaleModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="StatefulSet"
          resourceName={activeModal.ss.name}
          currentReplicas={activeModal.ss.replicas}
          onScale={(replicas) =>
            scaleStatefulsetCmd(clusterId, activeModal.ss.namespace, activeModal.ss.name, replicas).then(() => {
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
          namespace={activeModal.ss.namespace}
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

      <ColumnConfigModal
        open={showColumnConfig}
        onOpenChange={setShowColumnConfig}
        resourceType="StatefulSets"
        columnConfig={columnConfig}
        columnLabels={{
          name: "Name",
          namespace: "Namespace",
          ready: "Ready",
          replicas: "Replicas",
          age: "Age",
          actions: "Actions",
        }}
      />
    </>
  );
}
