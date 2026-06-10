import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Button } from "@/components/ui";
import { Scale, RotateCcw, Undo2, Pencil, Trash2, FileText, Settings } from "lucide-react";
import type { DeploymentInfo } from "@/lib/tauriCommands";
import {
  scaleDeploymentCmd,
  restartDeploymentCmd,
  rollbackDeploymentCmd,
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

interface DeploymentListProps {
  deployments: DeploymentInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "scale"; deployment: DeploymentInfo }
  | { type: "restart"; deployment: DeploymentInfo }
  | { type: "rollback"; deployment: DeploymentInfo }
  | { type: "logs"; deployment: DeploymentInfo }
  | { type: "edit"; deployment: DeploymentInfo; yaml: string }
  | { type: "delete"; deployment: DeploymentInfo }
  | null;

export function DeploymentList({ deployments, clusterId, namespace: _namespace, onRefresh }: DeploymentListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isActing, setIsActing] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [showColumnConfig, setShowColumnConfig] = useState(false);

  // Configurable columns
  const columnConfig = useColumnConfig("deployments", DEFAULT_COLUMNS.deployments);
  const { isColumnVisible } = columnConfig;

  const openEdit = async (deployment: DeploymentInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "deployments", deployment.namespace, deployment.name);
      setActiveModal({ type: "edit", deployment, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRestart = async () => {
    if (activeModal?.type !== "restart") return;
    setIsActing(true);
    try {
      await restartDeploymentCmd(clusterId, activeModal.deployment.namespace, activeModal.deployment.name);
      setActiveModal(null);
      onRefresh?.();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsActing(false);
    }
  };

  const handleRollback = async () => {
    if (activeModal?.type !== "rollback") return;
    setIsActing(true);
    try {
      await rollbackDeploymentCmd(clusterId, activeModal.deployment.namespace, activeModal.deployment.name);
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
      await deleteResourceCmd(clusterId, "deployments", activeModal.deployment.namespace, activeModal.deployment.name);
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
          {deployments.length} {deployments.length === 1 ? "deployment" : "deployments"}
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
              {isColumnVisible("upToDate") && <TableHead>Up-to-date</TableHead>}
              {isColumnVisible("available") && <TableHead>Available</TableHead>}
              {isColumnVisible("age") && <TableHead>Age</TableHead>}
              {isColumnVisible("actions") && <TableHead className="text-right">Actions</TableHead>}
            </TableRow>
          </TableHeader>
          <TableBody>
            {deployments.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No deployments found
                </TableCell>
              </TableRow>
            ) : (
              deployments.map((deployment) => (
                <TableRow key={deployment.name}>
                  {isColumnVisible("name") && (
                    <TableCell className="font-medium">{deployment.name}</TableCell>
                  )}
                  {isColumnVisible("namespace") && (
                    <TableCell className="text-muted-foreground">{deployment.namespace}</TableCell>
                  )}
                  {isColumnVisible("ready") && <TableCell>{deployment.ready}</TableCell>}
                  {isColumnVisible("upToDate") && <TableCell>{deployment.up_to_date}</TableCell>}
                  {isColumnVisible("available") && <TableCell>{deployment.available}</TableCell>}
                  {isColumnVisible("age") && (
                    <TableCell className="text-muted-foreground">{deployment.age}</TableCell>
                  )}
                  {isColumnVisible("actions") && (
                    <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Scale",
                          icon: Scale,
                          onClick: () => setActiveModal({ type: "scale", deployment }),
                        },
                        {
                          label: "Restart",
                          icon: RotateCcw,
                          onClick: () => setActiveModal({ type: "restart", deployment }),
                        },
                        {
                          label: "Rollback",
                          icon: Undo2,
                          onClick: () => setActiveModal({ type: "rollback", deployment }),
                        },
                        {
                          label: "Logs",
                          icon: FileText,
                          onClick: () => setActiveModal({ type: "logs", deployment }),
                        },
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(deployment),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", deployment }),
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
          namespace={activeModal.deployment.namespace}
          workloadType="deployment"
          workloadName={activeModal.deployment.name}
          labels={activeModal.deployment.labels}
        />
      )}

      {activeModal?.type === "scale" && (
        <ScaleModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Deployment"
          resourceName={activeModal.deployment.name}
          currentReplicas={activeModal.deployment.replicas}
          onScale={(replicas) =>
            scaleDeploymentCmd(clusterId, activeModal.deployment.namespace, activeModal.deployment.name, replicas).then(() => {
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
          resourceType="Deployment"
          resourceName={activeModal.deployment.name}
          isLoading={isActing}
          onConfirm={handleRestart}
          variant="delete"
        />
      )}

      {activeModal?.type === "rollback" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Deployment"
          resourceName={activeModal.deployment.name}
          isLoading={isActing}
          onConfirm={handleRollback}
          variant="delete"
        />
      )}

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={clusterId}
          namespace={activeModal.deployment.namespace}
          resourceType="deployments"
          resourceName={activeModal.deployment.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Deployment"
          resourceName={activeModal.deployment.name}
          isLoading={isActing}
          onConfirm={handleDelete}
        />
      )}

      <ColumnConfigModal
        open={showColumnConfig}
        onOpenChange={setShowColumnConfig}
        resourceType="Deployments"
        columnConfig={columnConfig}
        columnLabels={{
          name: "Name",
          namespace: "Namespace",
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
