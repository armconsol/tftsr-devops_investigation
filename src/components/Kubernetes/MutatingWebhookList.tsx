import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { WebhookConfigInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface MutatingWebhookListProps {
  items: WebhookConfigInfo[];
  clusterId: string;
  namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; wh: WebhookConfigInfo; yaml: string }
  | { type: "delete"; wh: WebhookConfigInfo }
  | null;

export function MutatingWebhookList({ items, clusterId, onRefresh }: MutatingWebhookListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (wh: WebhookConfigInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "mutatingwebhookconfigurations", "", wh.name);
      setActiveModal({ type: "edit", wh, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(clusterId, "mutatingwebhookconfigurations", "", activeModal.wh.name);
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
              <TableHead>Webhooks</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} className="text-center text-muted-foreground">
                  No mutating webhook configurations found
                </TableCell>
              </TableRow>
            ) : (
              items.map((wh) => (
                <TableRow key={wh.name}>
                  <TableCell className="font-medium">{wh.name}</TableCell>
                  <TableCell className="text-sm">{wh.webhooks}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{wh.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(wh),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", wh }),
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
          namespace=""
          resourceType="mutatingwebhookconfigurations"
          resourceName={activeModal.wh.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="MutatingWebhookConfiguration"
          resourceName={activeModal.wh.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
