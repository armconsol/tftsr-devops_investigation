// Database Connection Manager Page

import { useState, useEffect } from 'react';
import { Button, Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui';
import { Plus, RefreshCw, Trash2, Edit, TestTube } from 'lucide-react';
import { ConnectionForm, type ConnectionFormData } from '@/components/Database/ConnectionForm';
import { useDatabaseStore } from '@/stores/databaseStore';
import {
  listDatabaseConnectionsCmd,
  createDatabaseConnectionCmd,
  updateDatabaseConnectionCmd,
  deleteDatabaseConnectionCmd,
  testDatabaseConnectionCmd,
  establishDbSshTunnelCmd,
  type DatabaseConnection,
} from '@/lib/tauriCommands';
import { toast } from 'sonner';

export function ConnectionManager() {
  const { connections, setConnections, setActiveConnection } = useDatabaseStore();
  const [loading, setLoading] = useState(false);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingConnection, setEditingConnection] = useState<DatabaseConnection | null>(null);
  const [testingId, setTestingId] = useState<string | null>(null);

  useEffect(() => {
    loadConnections();
  }, []);

  async function loadConnections() {
    setLoading(true);
    try {
      const conns = await listDatabaseConnectionsCmd();
      setConnections(conns);
      toast.success(`Loaded ${conns.length} connections`);
    } catch (error) {
      toast.error('Failed to load connections: ' + String(error));
    } finally {
      setLoading(false);
    }
  }

  const handleCreate = async (data: ConnectionFormData) => {
    try {
      const connection = await createDatabaseConnectionCmd(data);
      await syncSshTunnelConfig(connection.id, data);
      toast.success('Connection created');
      setDialogOpen(false);
      loadConnections();
    } catch (error) {
      toast.error('Failed to create connection: ' + String(error));
    }
  };

  const handleUpdate = async (data: ConnectionFormData) => {
    if (!editingConnection) return;

    try {
      await updateDatabaseConnectionCmd({
        id: editingConnection.id,
        ...data,
      });
      await syncSshTunnelConfig(editingConnection.id, data);
      toast.success('Connection updated');
      setDialogOpen(false);
      setEditingConnection(null);
      loadConnections();
    } catch (error) {
      toast.error('Failed to update connection: ' + String(error));
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this connection?')) return;

    try {
      await deleteDatabaseConnectionCmd(id);
      toast.success('Connection deleted');
      loadConnections();
    } catch (error) {
      toast.error('Failed to delete connection: ' + String(error));
    }
  };

  const syncSshTunnelConfig = async (connectionId: string, data: ConnectionFormData) => {
    if (!data.ssh_enabled) {
      return;
    }

    const result = await establishDbSshTunnelCmd(
      connectionId,
      data.ssh_hostname || '',
      data.ssh_port || 22,
      data.ssh_username || '',
      data.ssh_auth_method || 'password',
      data.ssh_auth_method === 'password' ? data.ssh_password : undefined,
      data.ssh_auth_method === 'key' ? data.ssh_private_key : undefined,
      data.ssh_auth_method === 'key' ? data.ssh_key_passphrase : undefined
    );

    if (!result.success) {
      throw new Error(result.message);
    }
  };

  const handleTest = async (id: string) => {
    setTestingId(id);
    try {
      const result = await testDatabaseConnectionCmd(id);
      if (result.success) {
        toast.success(`Connected successfully (${result.latency_ms}ms)`);
      } else {
        toast.error(result.message);
      }
    } catch (error) {
      toast.error('Connection test failed: ' + String(error));
    } finally {
      setTestingId(null);
    }
  };

  const openCreateDialog = () => {
    setEditingConnection(null);
    setDialogOpen(true);
  };

  const openEditDialog = (conn: DatabaseConnection) => {
    setEditingConnection(conn);
    setDialogOpen(true);
  };

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Database Connections</h1>
        <div className="flex gap-2">
          <Button onClick={loadConnections} disabled={loading}>
            <RefreshCw className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button onClick={openCreateDialog}>
            <Plus className="w-4 h-4 mr-2" />
            Add Connection
          </Button>
        </div>
      </div>

      <div className="grid gap-4">
        {connections.length === 0 ? (
          <div className="text-center py-12 text-muted-foreground">
            No connections configured. Click "Add Connection" to get started.
          </div>
        ) : (
          connections.map((conn) => (
            <div key={conn.id} className="border rounded-lg p-4 hover:bg-muted transition-colors">
              <div className="flex justify-between items-start">
                <div className="flex-1">
                  <h3 className="font-semibold">{conn.name}</h3>
                  <p className="text-sm text-muted-foreground">
                    {conn.db_type} • {conn.host}:{conn.port}
                    {conn.database_name && ` / ${conn.database_name}`}
                  </p>
                  <p className="text-xs text-muted-foreground mt-1">
                    {conn.username} • {conn.ssl_enabled ? 'SSL enabled' : 'No SSL'}
                    {conn.ssh_enabled ? ' • SSH tunnel enabled' : ''}
                  </p>
                </div>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => handleTest(conn.id)}
                    disabled={testingId === conn.id}
                  >
                    <TestTube className="w-4 h-4 mr-1" />
                    {testingId === conn.id ? 'Testing...' : 'Test'}
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => setActiveConnection(conn.id)}
                  >
                    Connect
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => openEditDialog(conn)}
                  >
                    <Edit className="w-4 h-4" />
                  </Button>
                  <Button
                    size="sm"
                    variant="destructive"
                    onClick={() => handleDelete(conn.id)}
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>
              {editingConnection ? 'Edit Connection' : 'Create Connection'}
            </DialogTitle>
          </DialogHeader>
          <ConnectionForm
            connection={editingConnection || undefined}
            onSubmit={editingConnection ? handleUpdate : handleCreate}
            onCancel={() => {
              setDialogOpen(false);
              setEditingConnection(null);
            }}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
}
