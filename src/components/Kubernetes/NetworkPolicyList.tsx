import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { NetworkPolicyInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface NetworkPolicyListProps {
  networkpolicies: NetworkPolicyInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; np: NetworkPolicyInfo; yaml: string }
  | { type: "delete"; np: NetworkPolicyInfo }
  | null;

export function NetworkPolicyList({ networkpolicies, clusterId, namespace, onRefresh }: NetworkPolicyListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (np: NetworkPolicyInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "networkpolicies", namespace, np.name);
      setActiveModal({ type: "edit", np, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(clusterId, "networkpolicies", namespace, activeModal.np.name);
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
              <TableHead>Pod Selector</TableHead>
              <TableHead>Policy Types</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {networkpolicies.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground">
                  No network policies found
                </TableCell>
              </TableRow>
            ) : (
              networkpolicies.map((np) => (
                <TableRow key={`${np.name}-${np.namespace}`}>
                  <TableCell className="font-medium">{np.name}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{np.namespace}</TableCell>
                  <TableCell className="text-sm font-mono truncate max-w-48">{np.pod_selector}</TableCell>
                  <TableCell className="text-sm">{np.policy_types.join(", ") || "—"}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{np.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(np),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", np }),
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
          resourceType="networkpolicies"
          resourceName={activeModal.np.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="NetworkPolicy"
          resourceName={activeModal.np.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
