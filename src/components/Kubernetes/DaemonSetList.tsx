import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Button } from "@/components/ui";
import { RotateCcw, Pencil, Trash2, FileText, Settings } from "lucide-react";
import type { DaemonSetInfo } from "@/lib/tauriCommands";
import {
  restartDaemonsetCmd,
  deleteResourceCmd,
  getResourceYamlCmd,
} from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";
import { openWorkloadLogsTab } from "@/lib/logsDock";
import { useColumnConfig } from "@/hooks/useColumnConfig";
import { DEFAULT_COLUMNS } from "@/config/defaultColumns";
import { ColumnConfigModal } from "@/components/tables/ColumnConfigModal";

interface DaemonSetListProps {
  daemonsets: DaemonSetInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "restart"; ds: DaemonSetInfo }
  | { type: "edit"; ds: DaemonSetInfo; yaml: string }
  | { type: "delete"; ds: DaemonSetInfo }
  | null;

export function DaemonSetList({ daemonsets, clusterId, namespace: _namespace, onRefresh }: DaemonSetListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isActing, setIsActing] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [showColumnConfig, setShowColumnConfig] = useState(false);

  // Configurable columns
  const columnConfig = useColumnConfig("daemonsets", DEFAULT_COLUMNS.daemonsets);
  const { isColumnVisible } = columnConfig;

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
      <div className="flex items-center justify-between mb-2">
        <div className="text-sm text-muted-foreground">
          {daemonsets.length} {daemonsets.length === 1 ? "daemonset" : "daemonsets"}
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
              {isColumnVisible("upToDate") && <TableHead>Up-to-date</TableHead>}
              {isColumnVisible("available") && <TableHead>Available</TableHead>}
              {isColumnVisible("age") && <TableHead>Age</TableHead>}
              {isColumnVisible("actions") && <TableHead className="text-right">Actions</TableHead>}
            </TableRow>
          </TableHeader>
          <TableBody>
            {daemonsets.length === 0 ? (
              <TableRow>
                <TableCell colSpan={9} className="text-center text-muted-foreground">
                  No daemonsets found
                </TableCell>
              </TableRow>
            ) : (
              daemonsets.map((ds) => (
                <TableRow key={ds.name}>
                  {isColumnVisible("name") && (
                    <TableCell className="font-medium">{ds.name}</TableCell>
                  )}
                  {isColumnVisible("namespace") && (
                    <TableCell className="text-muted-foreground">{ds.namespace}</TableCell>
                  )}
                  {isColumnVisible("desired") && <TableCell>{ds.desired}</TableCell>}
                  {isColumnVisible("current") && <TableCell>{ds.current}</TableCell>}
                  {isColumnVisible("ready") && <TableCell>{ds.ready}</TableCell>}
                  {isColumnVisible("upToDate") && <TableCell>{ds.up_to_date}</TableCell>}
                  {isColumnVisible("available") && <TableCell>{ds.available}</TableCell>}
                  {isColumnVisible("age") && (
                    <TableCell className="text-muted-foreground">{ds.age}</TableCell>
                  )}
                  {isColumnVisible("actions") && (
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
                          onClick: () =>
                            openWorkloadLogsTab({
                              clusterId,
                              namespace: ds.namespace,
                              workloadName: ds.name,
                              workloadType: "daemonset",
                            }),
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
                  )}
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

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

      <ColumnConfigModal
        open={showColumnConfig}
        onOpenChange={setShowColumnConfig}
        resourceType="DaemonSets"
        columnConfig={columnConfig}
        columnLabels={{
          name: "Name",
          namespace: "Namespace",
          desired: "Desired",
          current: "Current",
          ready: "Ready",
          upToDate: "Up-to-date",
          available: "Available",
          age: "Age",
          actions: "Actions",
        }}
      />
    </>
  );
}
