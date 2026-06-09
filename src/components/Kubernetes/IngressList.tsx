import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { IngressInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface IngressListProps {
  ingresses: IngressInfo[];
  clusterId?: string;
  _clusterId?: string;
  namespace?: string;
  _namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; ingress: IngressInfo; yaml: string }
  | { type: "delete"; ingress: IngressInfo }
  | null;

export function IngressList({
  ingresses,
  clusterId,
  _clusterId,
  namespace,
  _namespace,
  onRefresh,
}: IngressListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const ns = namespace ?? _namespace ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (ingress: IngressInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "ingresses", ns, ingress.name);
      setActiveModal({ type: "edit", ingress, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(cid, "ingresses", ns, activeModal.ingress.name);
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
              <TableHead>Class</TableHead>
              <TableHead>Host</TableHead>
              <TableHead>Addresses</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {ingresses.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No ingresses found
                </TableCell>
              </TableRow>
            ) : (
              ingresses.map((ingress) => (
                <TableRow key={`${ingress.name}-${ingress.namespace}`}>
                  <TableCell className="font-medium">{ingress.name}</TableCell>
                  <TableCell>{ingress.namespace}</TableCell>
                  <TableCell>{ingress.class || "-"}</TableCell>
                  <TableCell>{ingress.host}</TableCell>
                  <TableCell>{ingress.addresses.join(", ")}</TableCell>
                  <TableCell className="text-muted-foreground">{ingress.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(ingress),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", ingress }),
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
          clusterId={cid}
          namespace={ns}
          resourceType="ingresses"
          resourceName={activeModal.ingress.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Ingress"
          resourceName={activeModal.ingress.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
