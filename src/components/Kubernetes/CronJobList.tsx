import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Button } from "@/components/ui";
import { PauseCircle, PlayCircle, Play, Pencil, Trash2, FileText, Settings } from "lucide-react";
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
import { WorkloadLogsModal } from "./WorkloadLogsModal";
import { useColumnConfig } from "@/hooks/useColumnConfig";
import { DEFAULT_COLUMNS } from "@/config/defaultColumns";
import { ColumnConfigModal } from "@/components/tables/ColumnConfigModal";

interface CronJobListProps {
  cronJobs: CronJobInfo[];
  clusterId?: string;
  _clusterId?: string;
  namespace?: string;
  _namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "logs"; cj: CronJobInfo }
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
  const [showColumnConfig, setShowColumnConfig] = useState(false);

  // Configurable columns
  const columnConfig = useColumnConfig("cronjobs", DEFAULT_COLUMNS.cronjobs);
  const { isColumnVisible } = columnConfig;

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
      <div className="flex items-center justify-between mb-2">
        <div className="text-sm text-muted-foreground">
          {cronJobs.length} {cronJobs.length === 1 ? "cron job" : "cron jobs"}
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
              {isColumnVisible("schedule") && <TableHead>Schedule</TableHead>}
              {isColumnVisible("active") && <TableHead>Active</TableHead>}
              {isColumnVisible("lastSchedule") && <TableHead>Last Schedule</TableHead>}
              {isColumnVisible("age") && <TableHead>Age</TableHead>}
              {isColumnVisible("labels") && <TableHead>Labels</TableHead>}
              {isColumnVisible("actions") && <TableHead className="text-right">Actions</TableHead>}
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
                  {isColumnVisible("name") && (
                    <TableCell className="font-medium">{cj.name}</TableCell>
                  )}
                  {isColumnVisible("namespace") && (
                    <TableCell className="text-muted-foreground">{cj.namespace}</TableCell>
                  )}
                  {isColumnVisible("schedule") && <TableCell>{cj.schedule}</TableCell>}
                  {isColumnVisible("active") && <TableCell>{cj.active}</TableCell>}
                  {isColumnVisible("lastSchedule") && <TableCell>{cj.last_schedule}</TableCell>}
                  {isColumnVisible("age") && (
                    <TableCell className="text-muted-foreground">{cj.age}</TableCell>
                  )}
                  {isColumnVisible("labels") && (
                    <TableCell>
                      {Object.entries(cj.labels)
                        .map(([k, v]) => `${k}=${v}`)
                        .join(", ")}
                    </TableCell>
                  )}
                  {isColumnVisible("actions") && (
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
                          label: "Logs",
                          icon: FileText,
                          onClick: () => setActiveModal({ type: "logs", cj }),
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
                  )}
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {activeModal?.type === "logs" && (
        <WorkloadLogsModal
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          clusterId={cid}
          namespace={activeModal.cj.namespace}
          workloadType="cronjob"
          workloadName={activeModal.cj.name}
          labels={activeModal.cj.labels}
        />
      )}

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

      <ColumnConfigModal
        open={showColumnConfig}
        onOpenChange={setShowColumnConfig}
        resourceType="CronJobs"
        columnConfig={columnConfig}
        columnLabels={{
          name: "Name",
          namespace: "Namespace",
          schedule: "Schedule",
          active: "Active",
          lastSchedule: "Last Schedule",
          age: "Age",
          labels: "Labels",
          actions: "Actions",
        }}
      />
    </>
  );
}
