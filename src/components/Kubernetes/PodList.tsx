import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Badge } from "@/components/ui";
import { FileText, Terminal, Link, Pencil, Trash2, Zap } from "lucide-react";
import type { PodInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, forceDeleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { LogsModal } from "./LogsModal";
import { ShellExecModal } from "./ShellExecModal";
import { AttachModal } from "./AttachModal";
import { EditResourceModal } from "./EditResourceModal";

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

  // namespace prop is retained for API compatibility (parent uses it to drive list fetches)
  void namespace;

  const getPodStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case "running":
        return "bg-green-500";
      case "pending":
        return "bg-yellow-500";
      case "succeeded":
      case "completed":
        return "bg-blue-500";
      case "failed":
      case "error":
        return "bg-red-500";
      default:
        return "bg-gray-500";
    }
  };

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
      <div className="overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Ready</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {pods.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  No pods found
                </TableCell>
              </TableRow>
            ) : (
              pods.map((pod) => (
                <TableRow key={pod.name}>
                  <TableCell className="font-medium">{pod.name}</TableCell>
                  <TableCell>
                    <Badge className={`${getPodStatusColor(pod.status)} text-white`}>
                      {pod.status}
                    </Badge>
                  </TableCell>
                  <TableCell>{pod.ready}</TableCell>
                  <TableCell className="text-muted-foreground">{pod.age}</TableCell>
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
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {activeModal?.type === "logs" && (
        <LogsModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          clusterId={clusterId}
          namespace={activeModal.pod.namespace}
          podName={activeModal.pod.name}
          containers={activeModal.pod.containers}
        />
      )}

      {activeModal?.type === "shell" && (
        <ShellExecModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          clusterId={clusterId}
          namespace={activeModal.pod.namespace}
          podName={activeModal.pod.name}
          containers={activeModal.pod.containers}
        />
      )}

      {activeModal?.type === "attach" && (
        <AttachModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          clusterId={clusterId}
          namespace={activeModal.pod.namespace}
          podName={activeModal.pod.name}
          containers={activeModal.pod.containers}
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
    </>
  );
}
