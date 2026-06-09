import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { PauseCircle, PlayCircle, Play, Pencil, Trash2 } from "lucide-react";
import type { CronJobInfo } from "@/lib/tauriCommands";
import {
  suspendCronjobCmd,
  resumeCronjobCmd,
  triggerCronjobCmd,
  deleteResourceCmd,
  getResourceYamlCmd,
} from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface CronJobListProps {
  cronJobs: CronJobInfo[];
  clusterId?: string;
  _clusterId?: string;
  namespace?: string;
  _namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; cj: CronJobInfo; yaml: string }
  | { type: "delete"; cj: CronJobInfo }
  | null;

export function CronJobList({
  cronJobs,
  clusterId,
  _clusterId,
  onRefresh,
}: CronJobListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const openEdit = async (cj: CronJobInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "cronjobs", cj.namespace, cj.name);
      setActiveModal({ type: "edit", cj, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleSuspend = async (cj: CronJobInfo) => {
    setActionError(null);
    try {
      await suspendCronjobCmd(cid, cj.namespace, cj.name);
      onRefresh?.();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleResume = async (cj: CronJobInfo) => {
    setActionError(null);
    try {
      await resumeCronjobCmd(cid, cj.namespace, cj.name);
      onRefresh?.();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleTrigger = async (cj: CronJobInfo) => {
    setActionError(null);
    try {
      await triggerCronjobCmd(cid, cj.namespace, cj.name);
      onRefresh?.();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(cid, "cronjobs", activeModal.cj.namespace, activeModal.cj.name);
      setActiveModal(null);
      onRefresh?.();
    } finally {
      setIsDeleting(false);
    }
  };

  const isSuspended = (cj: CronJobInfo) => {
    const labels = cj.labels ?? {};
    return labels["cronjob.kubernetes.io/suspended"] === "true";
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
              <TableHead>Schedule</TableHead>
              <TableHead>Active</TableHead>
              <TableHead>Last Schedule</TableHead>
              <TableHead>Age</TableHead>
              <TableHead>Labels</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {cronJobs.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground">
                  No cron jobs found
                </TableCell>
              </TableRow>
            ) : (
              cronJobs.map((cj) => (
                <TableRow key={`${cj.name}-${cj.namespace}`}>
                  <TableCell className="font-medium">{cj.name}</TableCell>
                  <TableCell>{cj.namespace}</TableCell>
                  <TableCell>{cj.schedule}</TableCell>
                  <TableCell>{cj.active}</TableCell>
                  <TableCell>{cj.last_schedule}</TableCell>
                  <TableCell className="text-muted-foreground">{cj.age}</TableCell>
                  <TableCell>
                    {Object.entries(cj.labels)
                      .map(([k, v]) => `${k}=${v}`)
                      .join(", ")}
                  </TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Suspend",
                          icon: PauseCircle,
                          hidden: isSuspended(cj),
                          onClick: () => handleSuspend(cj),
                        },
                        {
                          label: "Resume",
                          icon: PlayCircle,
                          hidden: !isSuspended(cj),
                          onClick: () => handleResume(cj),
                        },
                        {
                          label: "Trigger",
                          icon: Play,
                          onClick: () => handleTrigger(cj),
                        },
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(cj),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", cj }),
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
          namespace={activeModal.cj.namespace}
          resourceType="cronjobs"
          resourceName={activeModal.cj.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="CronJob"
          resourceName={activeModal.cj.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
