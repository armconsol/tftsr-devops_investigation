import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw, Plus } from 'lucide-react';
import { BackupJobList } from '@/components/Proxmox';
import type { BackupJobInfo } from '@/components/Proxmox/BackupJobList';
import {
  listProxmoxBackupJobs,
  createProxmoxBackupJob,
  updateProxmoxBackupJob,
  deleteProxmoxBackupJob,
  triggerProxmoxBackupJob,
} from '@/lib/proxmoxClient';
import { toast } from 'sonner';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { useProxmoxClusters } from '@/hooks/useProxmoxClusters';

export function ProxmoxBackupPage() {
  const { clusters, selectedClusterId, setSelectedClusterId } = useProxmoxClusters();
  const [jobs, setJobs] = useState<BackupJobInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [showNewJobDialog, setShowNewJobDialog] = useState(false);
  const [editingJobId, setEditingJobId] = useState<string | null>(null);

  // Job form state (shared by create + edit dialogs)
  const [jobStorage, setJobStorage] = useState('');
  const [jobSchedule, setJobSchedule] = useState('0 2 * * *');
  const [jobVms, setJobVms] = useState('all');
  const [jobMode, setJobMode] = useState('snapshot');

  const loadJobs = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoading(true);
    try {
      const raw = await listProxmoxBackupJobs(clusterId);
      const normalized: BackupJobInfo[] = (raw as Record<string, unknown>[]).map((job) => {
        const enabledRaw = job.enabled ?? job.enable ?? 1;
        const isEnabled = enabledRaw === 1 || enabledRaw === true || enabledRaw === '1';
        const nextRunRaw = job['next-run'] ?? job.next_run ?? job.nextRun;
        const nextRunStr = nextRunRaw
          ? new Date(Number(nextRunRaw) * 1000).toLocaleString()
          : undefined;
        return {
          id: String(job.id || job.jobid || ''),
          name: String(job.id || job.comment || `job-${job.jobid || '?'}`),
          node: String(job.node || 'all'),
          schedule: String(job.schedule || '-'),
          status: 'idle' as const,
          lastRun: undefined,
          nextRun: nextRunStr,
          size: undefined,
          count: undefined,
          enabled: isEnabled,
          vmid: job.vmid as string | number | undefined,
          storage: job.storage as string | undefined,
          mode: job.mode as string | undefined,
          comment: job.comment as string | undefined,
        };
      });
      setJobs(normalized);
    } catch (err) {
      console.error('Failed to load backup jobs:', err);
      toast.error(`Failed to load backup jobs: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) loadJobs(selectedClusterId);
  }, [selectedClusterId, loadJobs]);

  const handleNewJob = () => {
    setEditingJobId(null);
    setJobStorage('');
    setJobSchedule('0 2 * * *');
    setJobVms('all');
    setJobMode('snapshot');
    setShowNewJobDialog(true);
  };

  const handleEditJob = (job: BackupJobInfo) => {
    setEditingJobId(job.id);
    setJobStorage(job.storage ?? '');
    setJobSchedule(job.schedule && job.schedule !== '-' ? job.schedule : '0 2 * * *');
    setJobVms(job.vmid != null ? String(job.vmid) : 'all');
    setJobMode(job.mode ?? 'snapshot');
    setShowNewJobDialog(true);
  };

  const handleTriggerJob = async (job: BackupJobInfo) => {
    try {
      await triggerProxmoxBackupJob(selectedClusterId, job.id);
      toast.success('Backup job started');
      await loadJobs(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to run backup job: ${err}`);
    }
  };

  const handleSubmitNewJob = async () => {
    if (!jobStorage.trim()) { toast.error('Storage is required'); return; }
    try {
      if (editingJobId) {
        await updateProxmoxBackupJob(selectedClusterId, editingJobId, {
          storage: jobStorage.trim(),
          vmid: jobVms.trim() || 'all',
          mode: jobMode,
          schedule: jobSchedule.trim() || '0 2 * * *',
        });
        toast.success('Backup job updated');
      } else {
        await createProxmoxBackupJob({
          clusterId: selectedClusterId,
          storage: jobStorage.trim(),
          vmid: jobVms.trim() || 'all',
          mode: jobMode,
          schedule: jobSchedule.trim() || '0 2 * * *',
          enabled: true,
        });
        toast.success('Backup job created');
      }
      setShowNewJobDialog(false);
      setEditingJobId(null);
      await loadJobs(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to ${editingJobId ? 'update' : 'create'} backup job: ${err}`);
    }
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleDeleteJob = async (job: any) => {
    try {
      await deleteProxmoxBackupJob(selectedClusterId, job.id);
      toast.success('Backup job deleted');
      await loadJobs(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to delete backup job: ${err}`);
    }
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleEnableJob = async (job: any) => {
    try {
      await updateProxmoxBackupJob(selectedClusterId, job.id, { enabled: true });
      toast.success('Backup job enabled');
      await loadJobs(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to enable backup job: ${err}`);
    }
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleDisableJob = async (job: any) => {
    try {
      await updateProxmoxBackupJob(selectedClusterId, job.id, { enabled: false });
      toast.success('Backup job disabled');
      await loadJobs(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to disable backup job: ${err}`);
    }
  };

  if (clusters.length === 0 && !isLoading) {
    return (
      <div className="space-y-4">
        <div>
          <h1 className="text-2xl font-bold">Backup Jobs</h1>
          <p className="text-muted-foreground">Manage Proxmox backup schedules</p>
        </div>
        <div className="text-center py-12 text-muted-foreground">
          <p>No Proxmox clusters configured.</p>
          <p className="text-sm mt-1">Add a remote connection first.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Backup Jobs</h1>
          <p className="text-muted-foreground">Manage Proxmox backup schedules</p>
        </div>
        <div className="flex items-center space-x-2">
          {clusters.length > 1 && (
            <select
              className="rounded-md border px-3 py-1.5 text-sm bg-background"
              value={selectedClusterId}
              onChange={(e) => setSelectedClusterId(e.target.value)}
            >
              {clusters.map((c) => (
                <option key={c.id} value={c.id}>{c.name}</option>
              ))}
            </select>
          )}
          <Button variant="outline" size="sm" onClick={() => loadJobs(selectedClusterId)}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
          <Button size="sm" onClick={handleNewJob}>
            <Plus className="mr-2 h-4 w-4" />
            New Job
          </Button>
        </div>
      </div>

      <BackupJobList
        jobs={jobs}
        onRefresh={() => loadJobs(selectedClusterId)}
        onTrigger={handleTriggerJob}
        onEdit={handleEditJob}
        onDelete={handleDeleteJob}
        onEnable={handleEnableJob}
        onDisable={handleDisableJob}
      />

      <Dialog open={showNewJobDialog} onOpenChange={(open) => {
        setShowNewJobDialog(open);
        if (!open) setEditingJobId(null);
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingJobId ? 'Edit Backup Job' : 'Create New Backup Job'}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="jobStorage">Storage</Label>
              <Input
                id="jobStorage"
                value={jobStorage}
                onChange={(e) => setJobStorage(e.target.value)}
                placeholder="e.g. local"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="jobSchedule">Schedule (cron format)</Label>
              <Input
                id="jobSchedule"
                value={jobSchedule}
                onChange={(e) => setJobSchedule(e.target.value)}
                placeholder="0 2 * * *"
              />
              <p className="text-xs text-muted-foreground">Example: "0 2 * * *" for daily at 2:00 AM</p>
            </div>
            <div className="space-y-2">
              <Label htmlFor="jobVms">VM IDs (comma-separated, or "all")</Label>
              <Input
                id="jobVms"
                value={jobVms}
                onChange={(e) => setJobVms(e.target.value)}
                placeholder="all"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="jobMode">Mode</Label>
              <select
                id="jobMode"
                className="w-full rounded-md border px-3 py-2 text-sm bg-background"
                value={jobMode}
                onChange={(e) => setJobMode(e.target.value)}
              >
                <option value="snapshot">Snapshot</option>
                <option value="suspend">Suspend</option>
                <option value="stop">Stop</option>
              </select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowNewJobDialog(false)}>
              Cancel
            </Button>
            <Button onClick={handleSubmitNewJob}>
              {editingJobId ? 'Save Changes' : 'Create Job'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
