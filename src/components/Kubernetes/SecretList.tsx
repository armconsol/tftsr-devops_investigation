import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Pencil, Trash2, Eye } from "lucide-react";
import type { SecretInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";
import { SecretDataModal } from "./SecretDataModal";

interface SecretListProps {
  secrets: SecretInfo[];
  clusterId?: string;
  _clusterId?: string;
  namespace?: string;
  _namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "view"; secret: SecretInfo; yaml: string }
  | { type: "edit"; secret: SecretInfo; yaml: string }
  | { type: "delete"; secret: SecretInfo }
  | null;

export function SecretList({
  secrets,
  clusterId,
  _clusterId,
  onRefresh,
}: SecretListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openView = async (secret: SecretInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "secrets", secret.namespace, secret.name);
      setActiveModal({ type: "view", secret, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const openEdit = async (secret: SecretInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "secrets", secret.namespace, secret.name);
      setActiveModal({ type: "edit", secret, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(cid, "secrets", activeModal.secret.namespace, activeModal.secret.name);
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
              <TableHead>Type</TableHead>
              <TableHead>Data Keys</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {secrets.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground">
                  No secrets found
                </TableCell>
              </TableRow>
            ) : (
              secrets.map((secret) => (
                <TableRow key={`${secret.name}-${secret.namespace}`}>
                  <TableCell className="font-medium">{secret.name}</TableCell>
                  <TableCell>{secret.namespace}</TableCell>
                  <TableCell>{secret.type}</TableCell>
                  <TableCell>{secret.data_keys}</TableCell>
                  <TableCell className="text-muted-foreground">{secret.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "View Data",
                          icon: Eye,
                          onClick: () => openView(secret),
                        },
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(secret),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", secret }),
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

      {activeModal?.type === "view" && (
        <SecretDataModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          secretName={activeModal.secret.name}
          secretYaml={activeModal.yaml}
        />
      )}

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={cid}
          namespace={activeModal.secret.namespace}
          resourceType="secrets"
          resourceName={activeModal.secret.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Secret"
          resourceName={activeModal.secret.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
