import React, { useState } from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { RemotesList } from '@/components/Proxmox';
import { AddRemoteForm } from '@/components/Proxmox';
import { EditRemoteForm } from '@/components/Proxmox';
import { RemoveRemoteDialog } from '@/components/Proxmox';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/index';

interface RemoteInfo {
  id: string;
  name: string;
  url: string;
  username: string;
  type: 'pve' | 'pbs';
  status: 'connected' | 'disconnected' | 'error';
}

export function ProxmoxRemotesPage() {
  const [remotes, setRemotes] = useState<RemoteInfo[]>([
    { id: '1', name: 'Production Cluster', url: 'https://pve1.example.com:8006', username: 'root@pam', type: 'pve', status: 'connected' },
    { id: '2', name: 'Backup Server', url: 'https://pbs1.example.com:8007', username: 'root@pam', type: 'pbs', status: 'connected' },
  ]);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [editingRemote, setEditingRemote] = useState<RemoteInfo | null>(null);
  const [removingRemote, setRemovingRemote] = useState<RemoteInfo | null>(null);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleAddRemote = (config: any) => {
    const newRemote: RemoteInfo = {
      id: String(remotes.length + 1),
      name: String(config.name),
      url: String(config.url),
      username: String(config.username),
      type: config.type as 'pve' | 'pbs',
      status: 'connected',
    };
    setRemotes([...remotes, newRemote]);
    setShowAddDialog(false);
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleEditRemote = (config: any) => {
    setRemotes(remotes.map(r => r.id === String(config.id) ? { ...r, ...config } as RemoteInfo : r));
    setEditingRemote(null);
  };

  const handleRemoveRemote = () => {
    if (removingRemote) {
      setRemotes(remotes.filter(r => r.id !== removingRemote.id));
      setRemovingRemote(null);
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
          <Button variant="outline" size="sm">
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
        onRefresh={() => {}}
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
