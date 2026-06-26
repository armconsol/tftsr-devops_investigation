import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { RoleBindingInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface RoleBindingListProps {
  roleBindings: RoleBindingInfo[];
  clusterId?: string;
  _clusterId?: string;
  namespace?: string;
  _namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; rb: RoleBindingInfo; yaml: string }
  | { type: "delete"; rb: RoleBindingInfo }
  | null;

export function RoleBindingList({
  roleBindings,
  clusterId,
  _clusterId,
  onRefresh,
}: RoleBindingListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (rb: RoleBindingInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "rolebindings", rb.namespace, rb.name);
      setActiveModal({ type: "edit", rb, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(cid, "rolebindings", activeModal.rb.namespace, activeModal.rb.name);
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
              <TableHead>Role</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {roleBindings.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  No role bindings found
                </TableCell>
              </TableRow>
            ) : (
              roleBindings.map((rb) => (
                <TableRow key={`${rb.name}-${rb.namespace}`}>
                  <TableCell className="font-medium">{rb.name}</TableCell>
                  <TableCell>{rb.namespace}</TableCell>
                  <TableCell>{rb.role}</TableCell>
                  <TableCell className="text-muted-foreground">{rb.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(rb),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", rb }),
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
          namespace={activeModal.rb.namespace}
          resourceType="rolebindings"
          resourceName={activeModal.rb.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="RoleBinding"
          resourceName={activeModal.rb.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
