import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { StorageList } from '@/components/Proxmox';
import type { StorageInfo } from '@/components/Proxmox/StorageList';
import {
  listProxmoxDatastores,
  updateProxmoxStorage,
  deleteProxmoxStorage,
} from '@/lib/proxmoxClient';
import { useProxmoxClusters } from '@/hooks/useProxmoxClusters';
import { toast } from 'sonner';

type StorageRow = StorageInfo;

export function ProxmoxStoragePage() {
  const { clusters, selectedClusterId, setSelectedClusterId } = useProxmoxClusters();
  const [storages, setStorages] = useState<StorageRow[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // Edit-storage dialog state
  const [editOpen, setEditOpen] = useState(false);
  const [editStorage, setEditStorage] = useState<StorageRow | null>(null);
  const [editContent, setEditContent] = useState('');
  const [editNodes, setEditNodes] = useState('');
  const [editDisabled, setEditDisabled] = useState(false);

  const loadStorages = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoading(true);
    try {
      const data = await listProxmoxDatastores(clusterId);
      setStorages(data);
    } catch (err) {
      console.error('Failed to load storages:', err);
      toast.error(`Failed to load storages: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) loadStorages(selectedClusterId);
  }, [selectedClusterId, loadStorages]);

  const handleEdit = (storage: StorageRow) => {
    setEditStorage(storage);
    setEditContent(storage.content ?? '');
    setEditNodes(storage.node ?? '');
    setEditDisabled(storage.status === 'disabled');
    setEditOpen(true);
  };

  const handleEditSubmit = async () => {
    if (!editStorage) return;
    try {
      await updateProxmoxStorage(selectedClusterId, editStorage.storage, {
        content: editContent.trim() || undefined,
        nodes: editNodes.trim(),
        disable: editDisabled,
      });
      toast.success(`Storage "${editStorage.storage}" updated`);
      setEditOpen(false);
      await loadStorages(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to update storage: ${err}`);
    }
  };

  const handleDelete = async (storage: StorageRow) => {
    if (!window.confirm(`Delete storage "${storage.storage}"? This removes the storage configuration from the datacenter.`)) {
      return;
    }
    try {
      await deleteProxmoxStorage(selectedClusterId, storage.storage);
      toast.success(`Storage "${storage.storage}" deleted`);
      await loadStorages(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to delete storage: ${err}`);
    }
  };

  if (clusters.length === 0 && !isLoading) {
    return (
      <div className="space-y-4">
        <div>
          <h1 className="text-2xl font-bold">Storage</h1>
          <p className="text-muted-foreground">Manage storage pools and volumes</p>
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
          <h1 className="text-2xl font-bold">Storage</h1>
          <p className="text-muted-foreground">Manage storage pools and volumes</p>
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
          <Button variant="outline" size="sm" onClick={() => loadStorages(selectedClusterId)}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <StorageList
        storages={storages}
        isLoading={isLoading}
        onRefresh={() => loadStorages(selectedClusterId)}
        onEdit={handleEdit}
        onDelete={handleDelete}
      />

      <Dialog open={editOpen} onOpenChange={setEditOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit Storage{editStorage ? ` — ${editStorage.storage}` : ''}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-1">
              <Label>Content (comma-separated)</Label>
              <Input
                value={editContent}
                onChange={(e) => setEditContent(e.target.value)}
                placeholder="e.g. images,iso,backup"
              />
            </div>
            <div className="space-y-1">
              <Label>Nodes (comma-separated, empty = all)</Label>
              <Input
                value={editNodes}
                onChange={(e) => setEditNodes(e.target.value)}
                placeholder="e.g. vmhost1,vmhost2"
              />
            </div>
            <label className="flex items-center space-x-2 text-sm">
              <input
                type="checkbox"
                checked={editDisabled}
                onChange={(e) => setEditDisabled(e.target.checked)}
              />
              <span>Disabled</span>
            </label>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditOpen(false)}>Cancel</Button>
            <Button onClick={handleEditSubmit}>Save</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
