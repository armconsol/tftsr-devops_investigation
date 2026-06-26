import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { LeaseInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface LeaseListProps {
  items: LeaseInfo[];
  clusterId: string;
  namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; lease: LeaseInfo; yaml: string }
  | { type: "delete"; lease: LeaseInfo }
  | null;

export function LeaseList({ items, clusterId, onRefresh }: LeaseListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (lease: LeaseInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "leases", lease.namespace, lease.name);
      setActiveModal({ type: "edit", lease, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(clusterId, "leases", activeModal.lease.namespace, activeModal.lease.name);
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
              <TableHead>Holder</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  No leases found
                </TableCell>
              </TableRow>
            ) : (
              items.map((lease) => (
                <TableRow key={`${lease.name}-${lease.namespace}`}>
                  <TableCell className="font-medium">{lease.name}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{lease.namespace}</TableCell>
                  <TableCell className="text-sm font-mono">{lease.holder || "—"}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{lease.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(lease),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", lease }),
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
          namespace={activeModal.lease.namespace}
          resourceType="leases"
          resourceName={activeModal.lease.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Lease"
          resourceName={activeModal.lease.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
