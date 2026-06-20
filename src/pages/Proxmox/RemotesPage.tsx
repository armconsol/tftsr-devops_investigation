import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { RemotesList } from '@/components/Proxmox';
import { AddRemoteForm } from '@/components/Proxmox';
import { EditRemoteForm } from '@/components/Proxmox';
import { RemoveRemoteDialog } from '@/components/Proxmox';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/index';
import { listProxmoxClusters, addProxmoxCluster, removeProxmoxCluster, updateProxmoxCluster, pingProxmoxCluster } from '@/lib/proxmoxClient';
import { ClusterType } from '@/lib/domain';
import { toast } from 'sonner';

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
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [editingRemote, setEditingRemote] = useState<RemoteInfo | null>(null);
  const [removingRemote, setRemovingRemote] = useState<RemoteInfo | null>(null);

  const loadRemotes = async () => {
    try {
      const clusters = await listProxmoxClusters();
      // TODO: Implement actual status checking via backend connection test
      const remotesList: RemoteInfo[] = clusters.map((c) => ({
        id: c.id,
        name: c.name,
        url: c.url,
        username: c.username,
        type: c.clusterType === 've' ? 'pve' : 'pbs',
        status: (c.connected ? 'connected' : 'disconnected') as RemoteInfo['status'],
      }));
      setRemotes(remotesList);
    } catch (err) {
      console.error('Failed to load remotes:', err);
    }
  };

  useEffect(() => {
    void loadRemotes();
  }, []);

  const generateId = (): string => {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  };

  /**
   * Helper function to parse a Proxmox URL and extract hostname and port.
   * Handles URLs with or without explicit port numbers.
   *
   * @param url - The full URL (e.g., "https://172.0.0.18:8006" or "https://pve.example.com")
   * @param type - The cluster type ('pve' or 'pbs') to determine default port
   * @returns Object with hostname (stripped of protocol and port) and port number
   */
  const parseRemoteUrl = (url: string, type: 'pve' | 'pbs'): { hostname: string; port: number } => {
    let hostname = url.replace(/^https?:\/\//, '');
    let port = type === 'pve' ? 8006 : 8007;

    const portMatch = hostname.match(/:(\d+)$/);
    if (portMatch) {
      port = parseInt(portMatch[1], 10);
      hostname = hostname.replace(/:\d+$/, '');
    }

    return { hostname, port };
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleAddRemote = async (config: any) => {
    try {
      const clusterType = config.type === 'pve' ? 've' : 'pbs';
      const { hostname, port } = parseRemoteUrl(config.url, config.type);

      const id = config.id || generateId();
      await addProxmoxCluster(
        id,
        config.name,
        clusterType as ClusterType,
        { url: hostname, port },
        config.username,
        config.password || ''
      );
      await loadRemotes();
      setShowAddDialog(false);
    } catch (err) {
      console.error('Failed to add remote:', err);
      toast.error('Failed to add remote: ' + String(err));
      throw err;
    }
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleEditRemote = async (config: any) => {
    try {
      const clusterType = config.type === 'pve' ? 've' : 'pbs';
      const { hostname, port } = parseRemoteUrl(config.url, config.type);

      await updateProxmoxCluster(
        config.id,
        config.name,
        clusterType as ClusterType,
        { url: hostname, port },
        config.username,
        config.password || ''
      );
      await loadRemotes();
      setEditingRemote(null);
    } catch (err) {
      console.error('Failed to edit remote:', err);
      toast.error('Failed to edit remote: ' + String(err));
      throw err;
    }
  };

  const handleRemoveRemote = async () => {
    if (removingRemote) {
      try {
        await removeProxmoxCluster(removingRemote.id);
        await loadRemotes();
        setRemovingRemote(null);
      } catch (err) {
        console.error('Failed to remove remote:', err);
        toast.error('Failed to remove remote: ' + String(err));
      }
    }
  };

  const handleConnectRemote = async (remote: RemoteInfo) => {
    try {
      toast.info(`Testing connection to ${remote.name}...`);
      await pingProxmoxCluster(remote.id);
      toast.success(`Connected to ${remote.name}`);
      setRemotes((prev) =>
        prev.map((r) => (r.id === remote.id ? { ...r, status: 'connected' } : r))
      );
    } catch (err) {
      console.error('Failed to connect remote:', err);
      toast.error('Connection failed: ' + String(err));
      setRemotes((prev) =>
        prev.map((r) => (r.id === remote.id ? { ...r, status: 'error' } : r))
      );
    }
  };

  const handleDisconnectRemote = (remote: RemoteInfo) => {
    setRemotes((prev) =>
      prev.map((r) => (r.id === remote.id ? { ...r, status: 'disconnected' } : r))
    );
    toast.info(`Disconnected from ${remote.name}`);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Remotes</h1>
          <p className="text-muted-foreground">Manage Proxmox VE and Backup Server connections</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={() => { void loadRemotes(); }}>
            <RefreshCw className="mr-2 h-4 w-4" />
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
        onRefresh={() => { void loadRemotes(); }}
        onEdit={(remote) => {
          setEditingRemote(remote as RemoteInfo | null);
        }}
        onDelete={(remote) => {
          setRemovingRemote(remote as RemoteInfo | null);
        }}
        onConnect={(remote) => {
          void handleConnectRemote(remote as RemoteInfo);
        }}
        onDisconnect={(remote) => {
          handleDisconnectRemote(remote as RemoteInfo);
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
