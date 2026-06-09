import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { ResourceQuotaInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface ResourceQuotaListProps {
  resourcequotas: ResourceQuotaInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; rq: ResourceQuotaInfo; yaml: string }
  | { type: "delete"; rq: ResourceQuotaInfo }
  | null;

export function ResourceQuotaList({ resourcequotas, clusterId, namespace, onRefresh }: ResourceQuotaListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (rq: ResourceQuotaInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "resourcequotas", namespace, rq.name);
      setActiveModal({ type: "edit", rq, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(clusterId, "resourcequotas", namespace, activeModal.rq.name);
      setActiveModal(null);
      onRefresh?.();
    } finally {
      setIsDeleting(false);
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
              <TableHead>CPU Req</TableHead>
              <TableHead>Mem Req</TableHead>
              <TableHead>CPU Limit</TableHead>
              <TableHead>Mem Limit</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {resourcequotas.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground">
                  No resource quotas found
                </TableCell>
              </TableRow>
            ) : (
              resourcequotas.map((rq) => (
                <TableRow key={`${rq.name}-${rq.namespace}`}>
                  <TableCell className="font-medium">{rq.name}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{rq.namespace}</TableCell>
                  <TableCell className="text-sm font-mono">{rq.request_cpu || "—"}</TableCell>
                  <TableCell className="text-sm font-mono">{rq.request_memory || "—"}</TableCell>
                  <TableCell className="text-sm font-mono">{rq.limit_cpu || "—"}</TableCell>
                  <TableCell className="text-sm font-mono">{rq.limit_memory || "—"}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{rq.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(rq),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", rq }),
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

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={clusterId}
          namespace={namespace}
          resourceType="resourcequotas"
          resourceName={activeModal.rq.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="ResourceQuota"
          resourceName={activeModal.rq.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
