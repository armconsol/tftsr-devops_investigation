import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Button } from "@/components/ui";
import { StatusBadge } from "@/components/Badge";
import { FileText, Terminal, Link, Pencil, Trash2, Zap, Settings } from "lucide-react";
import type { PodInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, forceDeleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { LogStreamPanel } from "./LogStreamPanel";
import { InteractiveShellModal } from "./InteractiveShellModal";
import { InteractiveAttachModal } from "./InteractiveAttachModal";
import { EditResourceModal } from "./EditResourceModal";
import { useColumnConfig } from "@/hooks/useColumnConfig";
import { useMetrics } from "@/hooks/useMetrics";
import { DEFAULT_COLUMNS } from "@/config/defaultColumns";
import { ColumnConfigModal } from "@/components/tables/ColumnConfigModal";

interface PodListProps {
  pods: PodInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "logs"; pod: PodInfo }
  | { type: "shell"; pod: PodInfo }
  | { type: "attach"; pod: PodInfo }
  | { type: "edit"; pod: PodInfo; yaml: string }
  | { type: "delete"; pod: PodInfo }
  | { type: "force-delete"; pod: PodInfo }
  | null;

export function PodList({ pods, clusterId, namespace, onRefresh }: PodListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [editError, setEditError] = useState<string | null>(null);
  const [showColumnConfig, setShowColumnConfig] = useState(false);

  // Configurable columns
  const columnConfig = useColumnConfig("pods", DEFAULT_COLUMNS.pods);
  const { isColumnVisible } = columnConfig;

  // Live pod metrics — only poll when CPU/Memory columns are actually visible.
  const metricsEnabled = isColumnVisible("cpu") || isColumnVisible("memory");
  const { getPodMetrics } = useMetrics(
    metricsEnabled ? clusterId : null,
    metricsEnabled ? namespace : null
  );

  const openEdit = async (pod: PodInfo) => {
    setEditError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "pods", pod.namespace, pod.name);
      setActiveModal({ type: "edit", pod, yaml });
    } catch (err) {
      setEditError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async (force: boolean) => {
    const modal = activeModal;
    if (!modal || (modal.type !== "delete" && modal.type !== "force-delete")) return;
    setIsDeleting(true);
    try {
      if (force) {
        await forceDeleteResourceCmd(clusterId, "pods", modal.pod.namespace, modal.pod.name);
      } else {
        await deleteResourceCmd(clusterId, "pods", modal.pod.namespace, modal.pod.name);
      }
      setActiveModal(null);
      onRefresh?.();
    } finally {
      setIsDeleting(false);
    }
  };

  const currentPod =
    activeModal && activeModal.type !== "edit" ? activeModal.pod : null;

  return (
    <>
      {editError && (
        <p className="mb-2 text-sm text-destructive">{editError}</p>
      )}
      <div className="flex items-center justify-between mb-2">
        <div className="text-sm text-muted-foreground">
          {pods.length} {pods.length === 1 ? "pod" : "pods"}
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
              {isColumnVisible("status") && <TableHead>Status</TableHead>}
              {isColumnVisible("ready") && <TableHead>Ready</TableHead>}
              {isColumnVisible("restarts") && <TableHead>Restarts</TableHead>}
              {isColumnVisible("age") && <TableHead>Age</TableHead>}
              {isColumnVisible("ip") && <TableHead>IP</TableHead>}
              {isColumnVisible("node") && <TableHead>Node</TableHead>}
              {isColumnVisible("cpu") && <TableHead>CPU</TableHead>}
              {isColumnVisible("memory") && <TableHead>Memory</TableHead>}
              {isColumnVisible("actions") && <TableHead className="text-right">Actions</TableHead>}
            </TableRow>
          </TableHeader>
          <TableBody>
            {pods.length === 0 ? (
              <TableRow>
                <TableCell colSpan={11} className="text-center text-muted-foreground">
                  No pods found
                </TableCell>
              </TableRow>
            ) : (
              pods.map((pod) => {
                const podMetrics = metricsEnabled ? getPodMetrics(pod.name) : undefined;
                return (
                <TableRow key={pod.name}>
                  {isColumnVisible("name") && (
                    <TableCell className="font-medium">{pod.name}</TableCell>
                  )}
                  {isColumnVisible("namespace") && (
                    <TableCell className="text-muted-foreground">{pod.namespace}</TableCell>
                  )}
                  {isColumnVisible("status") && (
                    <TableCell>
                      <StatusBadge status={pod.status} />
                    </TableCell>
                  )}
                  {isColumnVisible("ready") && <TableCell>{pod.ready}</TableCell>}
                  {isColumnVisible("restarts") && <TableCell>{pod.restarts}</TableCell>}
                  {isColumnVisible("age") && (
                    <TableCell className="text-muted-foreground">{pod.age}</TableCell>
                  )}
                  {isColumnVisible("ip") && (
                    <TableCell className="text-muted-foreground font-mono text-xs">{pod.ip || "-"}</TableCell>
                  )}
                  {isColumnVisible("node") && (
                    <TableCell className="text-muted-foreground">{pod.node || "-"}</TableCell>
                  )}
                  {isColumnVisible("cpu") && (
                    <TableCell className="text-muted-foreground font-mono text-xs">
                      {podMetrics?.cpu ?? "-"}
                    </TableCell>
                  )}
                  {isColumnVisible("memory") && (
                    <TableCell className="text-muted-foreground font-mono text-xs">
                      {podMetrics?.memory ?? "-"}
                    </TableCell>
                  )}
                  {isColumnVisible("actions") && (
                    <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Logs",
                          icon: FileText,
                          onClick: () => setActiveModal({ type: "logs", pod }),
                        },
                        {
                          label: "Shell",
                          icon: Terminal,
                          onClick: () => setActiveModal({ type: "shell", pod }),
                        },
                        {
                          label: "Attach",
                          icon: Link,
                          onClick: () => setActiveModal({ type: "attach", pod }),
                        },
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(pod),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", pod }),
                        },
                        {
                          label: "Force Delete",
                          icon: Zap,
                          variant: "destructive",
                          hidden: !(
                            pod.status.toLowerCase() === "running" ||
                            pod.status.toLowerCase() === "pending"
                          ),
                          onClick: () => setActiveModal({ type: "force-delete", pod }),
                        },
                      ]}
                    />
                    </TableCell>
                  )}
                </TableRow>
                );
              })
            )}
          </TableBody>
        </Table>
      </div>

      {activeModal?.type === "logs" && (
        <LogStreamPanel
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          clusterId={clusterId}
          namespace={activeModal.pod.namespace}
          podName={activeModal.pod.name}
          containers={activeModal.pod.containers}
        />
      )}

      {activeModal?.type === "shell" && (
        <InteractiveShellModal
          clusterId={clusterId}
          namespace={activeModal.pod.namespace}
          pod={activeModal.pod.name}
          container={activeModal.pod.containers[0]}
          onClose={() => setActiveModal(null)}
        />
      )}

      {activeModal?.type === "attach" && (
        <InteractiveAttachModal
          clusterId={clusterId}
          namespace={activeModal.pod.namespace}
          pod={activeModal.pod.name}
          container={activeModal.pod.containers[0]}
          onClose={() => setActiveModal(null)}
        />
      )}

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={clusterId}
          namespace={activeModal.pod.namespace}
          resourceType="pods"
          resourceName={activeModal.pod.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && currentPod && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Pod"
          resourceName={currentPod.name}
          isLoading={isDeleting}
          onConfirm={() => handleDelete(false)}
        />
      )}

      {activeModal?.type === "force-delete" && currentPod && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Pod"
          resourceName={currentPod.name}
          variant="force-delete"
          isLoading={isDeleting}
          onConfirm={() => handleDelete(true)}
        />
      )}

      <ColumnConfigModal
        open={showColumnConfig}
        onOpenChange={setShowColumnConfig}
        resourceType="Pods"
        columnConfig={columnConfig}
        columnLabels={{
          name: "Name",
          namespace: "Namespace",
          status: "Status",
          ready: "Ready",
          restarts: "Restarts",
          age: "Age",
          ip: "IP Address",
          node: "Node",
          cpu: "CPU",
          memory: "Memory",
          actions: "Actions",
        }}
      />
    </>
  );
}
