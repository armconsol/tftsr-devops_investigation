import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Scale, RotateCcw, Undo2, Pencil, Trash2 } from "lucide-react";
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
  | { type: "edit"; deployment: DeploymentInfo; yaml: string }
  | { type: "delete"; deployment: DeploymentInfo }
  | null;

export function DeploymentList({ deployments, clusterId, namespace, onRefresh }: DeploymentListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isActing, setIsActing] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (deployment: DeploymentInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "deployments", namespace, deployment.name);
      setActiveModal({ type: "edit", deployment, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRestart = async () => {
    if (activeModal?.type !== "restart") return;
    setIsActing(true);
    try {
      await restartDeploymentCmd(clusterId, namespace, activeModal.deployment.name);
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
      await rollbackDeploymentCmd(clusterId, namespace, activeModal.deployment.name);
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
      await deleteResourceCmd(clusterId, "deployments", namespace, activeModal.deployment.name);
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
              <TableHead>Up-to-date</TableHead>
              <TableHead>Available</TableHead>
              <TableHead>Replicas</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
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
                  <TableCell className="font-medium">{deployment.name}</TableCell>
                  <TableCell>{deployment.ready}</TableCell>
                  <TableCell>{deployment.up_to_date}</TableCell>
                  <TableCell>{deployment.available}</TableCell>
                  <TableCell>{deployment.replicas}</TableCell>
                  <TableCell className="text-muted-foreground">{deployment.age}</TableCell>
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
          resourceType="Deployment"
          resourceName={activeModal.deployment.name}
          currentReplicas={activeModal.deployment.replicas}
          onScale={(replicas) =>
            scaleDeploymentCmd(clusterId, namespace, activeModal.deployment.name, replicas).then(() => {
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
          namespace={namespace}
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
    </>
  );
}
