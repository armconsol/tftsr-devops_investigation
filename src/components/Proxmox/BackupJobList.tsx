import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Play, Trash2 } from 'lucide-react';

interface BackupJobInfo {
  id: string;
  name: string;
  node: string;
  schedule: string;
  status: 'idle' | 'running' | 'success' | 'failed';
  lastRun?: string;
  nextRun?: string;
  size?: number;
  count?: number;
  enabled: boolean;
  storage?: string;
  vmid?: string | number;
  mode?: string;
  comment?: string;
}

interface BackupJobListProps {
  jobs: BackupJobInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onTrigger?: (job: BackupJobInfo) => void;
  onEdit?: (job: BackupJobInfo) => void;
  onDelete?: (job: BackupJobInfo) => void;
  onEnable?: (job: BackupJobInfo) => void;
  onDisable?: (job: BackupJobInfo) => void;
}

export function BackupJobList({
  jobs,
  onRefresh,
  isLoading,
  onTrigger,
  onEdit,
  onDelete,
  onEnable,
  onDisable,
}: BackupJobListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Backup Jobs</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm">
            <span className="mr-2 h-4 w-4">+</span>
            New Job
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>ID</TableHead>
                <TableHead>Storage</TableHead>
                <TableHead>VMs</TableHead>
                <TableHead>Node</TableHead>
                <TableHead>Schedule</TableHead>
                <TableHead>Enabled</TableHead>
                <TableHead>Next Run</TableHead>
                <TableHead>Mode</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {jobs.map((job) => (
                <TableRow key={job.id}>
                  <TableCell className="font-medium font-mono text-xs">{job.name}</TableCell>
                  <TableCell>{job.storage || '-'}</TableCell>
                  <TableCell className="text-xs">{job.vmid ? String(job.vmid) : 'all'}</TableCell>
                  <TableCell>{job.node || 'all'}</TableCell>
                  <TableCell className="font-mono text-xs">{job.schedule}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      job.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                    }`}>
                      {job.enabled ? 'enabled' : 'disabled'}
                    </span>
                  </TableCell>
                  <TableCell className="text-xs">{job.nextRun || '-'}</TableCell>
                  <TableCell className="text-xs">{job.mode || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onTrigger?.(job)}
                        title="Trigger Now"
                      >
                        <Play className="h-4 w-4" />
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(job)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => job.enabled ? onDisable?.(job) : onEnable?.(job)}
                        title={job.enabled ? 'Disable' : 'Enable'}
                      >
                        {job.enabled ? (
                          <span className="h-4 w-4 text-xs">⏸️</span>
                        ) : (
                          <span className="h-4 w-4 text-xs">▶️</span>
                        )}
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(job)}
                        title="Delete"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        title="More"
                      >
                        <MoreHorizontal className="h-4 w-4" />
                      </button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}
