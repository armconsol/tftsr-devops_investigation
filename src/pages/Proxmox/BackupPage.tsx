import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw, Plus } from 'lucide-react';
import { BackupJobList } from '@/components/Proxmox';
import { listProxmoxClusters, listProxmoxBackupJobs } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';

export function ProxmoxBackupPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState<string>('');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [jobs, setJobs] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [showNewJobDialog, setShowNewJobDialog] = useState(false);
  
  // New job form state
  const [jobName, setJobName] = useState('');
  const [jobNode, setJobNode] = useState('');
  const [jobSchedule, setJobSchedule] = useState('');
  const [jobVms, setJobVms] = useState('');

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) setSelectedClusterId(cls[0].id);
      })
      .catch((err) => {
        console.error('Failed to load clusters:', err);
        toast.error('Failed to load clusters');
      });
  }, []);

  const loadJobs = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoading(true);
    try {
      const data = await listProxmoxBackupJobs(clusterId, '');
      setJobs(data);
    } catch (err) {
      console.error('Failed to load backup jobs:', err);
      toast.error('Failed to load backup jobs');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) loadJobs(selectedClusterId);
  }, [selectedClusterId, loadJobs]);

  const handleNewJob = () => {
    setJobName('');
    setJobNode('');
    setJobSchedule('');
    setJobVms('');
    setShowNewJobDialog(true);
  };

  const handleSubmitNewJob = async () => {
    if (!jobName || !jobNode || !jobSchedule) {
      toast.error('Job name, node, and schedule are required');
      return;
    }

    try {
      toast.info(`Creating backup job ${jobName} - implementation pending`);
      setShowNewJobDialog(false);
    } catch (error) {
      console.error('Failed to create backup job:', error);
      toast.error(`Failed to create backup job: ${error}`);
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
      />

      <Dialog open={showNewJobDialog} onOpenChange={setShowNewJobDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create New Backup Job</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="jobName">Job Name</Label>
              <Input
                id="jobName"
                value={jobName}
                onChange={(e) => setJobName(e.target.value)}
                placeholder="daily-backup"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="jobNode">Node</Label>
              <Input
                id="jobNode"
                value={jobNode}
                onChange={(e) => setJobNode(e.target.value)}
                placeholder="pve"
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
              <p className="text-xs text-muted-foreground">
                Example: "0 2 * * *" for daily at 2:00 AM
              </p>
            </div>
            <div className="space-y-2">
              <Label htmlFor="jobVms">VMs to Backup (comma-separated IDs)</Label>
              <Input
                id="jobVms"
                value={jobVms}
                onChange={(e) => setJobVms(e.target.value)}
                placeholder="100, 101, 102"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowNewJobDialog(false)}>
              Cancel
            </Button>
            <Button onClick={handleSubmitNewJob}>
              Create Job
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
