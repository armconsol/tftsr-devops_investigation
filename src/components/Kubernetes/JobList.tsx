import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Button } from "@/components/ui";
import { Pencil, Trash2, FileText, Settings } from "lucide-react";
import type { JobInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";
import { WorkloadLogsModal } from "./WorkloadLogsModal";
import { useColumnConfig } from "@/hooks/useColumnConfig";
import { DEFAULT_COLUMNS } from "@/config/defaultColumns";
import { ColumnConfigModal } from "@/components/tables/ColumnConfigModal";

interface JobListProps {
  jobs: JobInfo[];
  clusterId?: string;
  _clusterId?: string;
  namespace?: string;
  _namespace?: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "logs"; job: JobInfo }
  | { type: "edit"; job: JobInfo; yaml: string }
  | { type: "delete"; job: JobInfo }
  | null;

export function JobList({
  jobs,
  clusterId,
  _clusterId,
  onRefresh,
}: JobListProps) {
  const cid = clusterId ?? _clusterId ?? "";
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [showColumnConfig, setShowColumnConfig] = useState(false);

  // Configurable columns
  const columnConfig = useColumnConfig("jobs", DEFAULT_COLUMNS.jobs);
  const { isColumnVisible } = columnConfig;

  const openEdit = async (job: JobInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(cid, "jobs", job.namespace, job.name);
      setActiveModal({ type: "edit", job, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(cid, "jobs", activeModal.job.namespace, activeModal.job.name);
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
      <div className="flex items-center justify-between mb-2">
        <div className="text-sm text-muted-foreground">
          {jobs.length} {jobs.length === 1 ? "job" : "jobs"}
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
              {isColumnVisible("completions") && <TableHead>Completions</TableHead>}
              {isColumnVisible("duration") && <TableHead>Duration</TableHead>}
              {isColumnVisible("age") && <TableHead>Age</TableHead>}
              {isColumnVisible("labels") && <TableHead>Labels</TableHead>}
              {isColumnVisible("actions") && <TableHead className="text-right">Actions</TableHead>}
            </TableRow>
          </TableHeader>
          <TableBody>
            {jobs.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No jobs found
                </TableCell>
              </TableRow>
            ) : (
              jobs.map((job) => (
                <TableRow key={`${job.name}-${job.namespace}`}>
                  {isColumnVisible("name") && (
                    <TableCell className="font-medium">{job.name}</TableCell>
                  )}
                  {isColumnVisible("namespace") && (
                    <TableCell className="text-muted-foreground">{job.namespace}</TableCell>
                  )}
                  {isColumnVisible("completions") && <TableCell>{job.completions}</TableCell>}
                  {isColumnVisible("duration") && <TableCell>{job.duration}</TableCell>}
                  {isColumnVisible("age") && (
                    <TableCell className="text-muted-foreground">{job.age}</TableCell>
                  )}
                  {isColumnVisible("labels") && (
                    <TableCell>
                      {Object.entries(job.labels)
                        .map(([k, v]) => `${k}=${v}`)
                        .join(", ")}
                    </TableCell>
                  )}
                  {isColumnVisible("actions") && (
                    <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Logs",
                          icon: FileText,
                          onClick: () => setActiveModal({ type: "logs", job }),
                        },
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(job),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", job }),
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
          namespace={activeModal.job.namespace}
          workloadType="job"
          workloadName={activeModal.job.name}
          labels={activeModal.job.labels}
        />
      )}

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={cid}
          namespace={activeModal.job.namespace}
          resourceType="jobs"
          resourceName={activeModal.job.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Job"
          resourceName={activeModal.job.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}

      <ColumnConfigModal
        open={showColumnConfig}
        onOpenChange={setShowColumnConfig}
        resourceType="Jobs"
        columnConfig={columnConfig}
        columnLabels={{
          name: "Name",
          namespace: "Namespace",
          completions: "Completions",
          duration: "Duration",
          age: "Age",
          labels: "Labels",
          actions: "Actions",
        }}
      />
    </>
  );
}
