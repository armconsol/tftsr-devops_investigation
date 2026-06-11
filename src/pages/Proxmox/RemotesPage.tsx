import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { RemotesList } from '@/components/Proxmox';
import { AddRemoteForm } from '@/components/Proxmox';
import { EditRemoteForm } from '@/components/Proxmox';
import { RemoveRemoteDialog } from '@/components/Proxmox';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/index';
import { addProxmoxClusterCmd, listProxmoxClustersCmd, removeProxmoxClusterCmd } from '@/lib/tauriCommands';

interface RemoteInfo {
  id: string;
  name: string;
  url: string;
  username: string;
  type: 'pve' | 'pbs';
  status: 'connected' | 'disconnected' | 'error';
}

export function ProxmoxRemotesPage() {
  const [remotes, setRemotes] = useState<RemoteInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [editingRemote, setEditingRemote] = useState<RemoteInfo | null>(null);
  const [removingRemote, setRemovingRemote] = useState<RemoteInfo | null>(null);

  const loadClusters = useCallback(async () => {
    try {
      const clusters = await listProxmoxClustersCmd();
      const mapped: RemoteInfo[] = clusters.map(c => ({
        id: c.id,
        name: c.name,
        url: `${c.url}:${c.port}`,
        username: c.username,
        type: c.clusterType === 've' ? 'pve' : 'pbs',
        status: 'connected',
      }));
      setRemotes(mapped);
    } catch (err) {
      console.error('Failed to load clusters:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadClusters();
  }, [loadClusters]);

  const handleAddRemote = async (config: any) => {
    try {
      const cluster = await addProxmoxClusterCmd(
        Date.now().toString(),
        config.name,
        config.type,
        config.url.replace(/^https?:\/\//, '').split(':')[0],
        parseInt(config.url.split(':').pop()) || (config.type === 'pve' ? 8006 : 8007),
        config.username,
        config.password || ''
      );
      const newRemote: RemoteInfo = {
        id: cluster.id,
        name: cluster.name,
        url: `${cluster.url}:${cluster.port}`,
        username: cluster.username,
        type: cluster.clusterType === 've' ? 'pve' : 'pbs',
        status: 'connected',
      };
      setRemotes([...remotes, newRemote]);
      setShowAddDialog(false);
    } catch (err) {
      console.error('Failed to add remote:', err);
      alert('Failed to add cluster. Check console for details.');
    }
  };

  const handleEditRemote = async (config: any) => {
    try {
      await addProxmoxClusterCmd(
        config.id,
        config.name,
        config.type === 'pve' ? 've' : 'pbs',
        config.url.split(':')[0],
        parseInt(config.url.split(':').pop()) || (config.type === 'pve' ? 8006 : 8007),
        config.username,
        ''
      );
      setRemotes(remotes.map(r => r.id === config.id ? { ...r, ...config } as RemoteInfo : r));
      setEditingRemote(null);
    } catch (err) {
      console.error('Failed to update remote:', err);
      alert('Failed to update cluster. Check console for details.');
    }
  };

  const handleRemoveRemote = async () => {
    if (removingRemote) {
      try {
        await removeProxmoxClusterCmd(removingRemote.id);
        setRemotes(remotes.filter(r => r.id !== removingRemote.id));
        setRemovingRemote(null);
      } catch (err) {
        console.error('Failed to remove remote:', err);
        alert('Failed to remove cluster. Check console for details.');
      }
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Remotes</h1>
          <p className="text-muted-foreground">Manage Proxmox VE and Backup Server connections</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={loadClusters} disabled={loading}>
            <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button onClick={() => setShowAddDialog(true)}>
            <span className="mr-2 h-4 w-4">+</span>
            Add Remote
          </Button>
        </div>
      </div>

      <RemotesList
        remotes={remotes}
        isLoading={loading}
        onRefresh={loadClusters}
        onEdit={(remote) => {
          setEditingRemote(remote as RemoteInfo | null);
        }}
        onDelete={(remote) => {
          setRemovingRemote(remote as RemoteInfo | null);
        }}
      />

      {showAddDialog && (
        <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
          <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
            <DialogHeader>
              <DialogTitle>Add New Remote</DialogTitle>
            </DialogHeader>
            <AddRemoteForm onAdd={handleAddRemote} onCancel={() => setShowAddDialog(false)} />
          </DialogContent>
        </Dialog>
      )}

      {editingRemote !== null && (
        <Dialog open={true} onOpenChange={() => setEditingRemote(null)}>
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle>Edit Remote</DialogTitle>
            </DialogHeader>
            <EditRemoteForm
              remote={editingRemote}
              onSave={handleEditRemote}
              onCancel={() => setEditingRemote(null)}
            />
          </DialogContent>
        </Dialog>
      )}

      {removingRemote !== null && (
        <Dialog open={true} onOpenChange={() => setRemovingRemote(null)}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Remove Remote</DialogTitle>
            </DialogHeader>
            <RemoveRemoteDialog
              remote={removingRemote}
              onConfirm={handleRemoveRemote}
              onCancel={() => setRemovingRemote(null)}
            />
          </DialogContent>
        </Dialog>
      )}
    </div>
  );
}
