// Database Connection Manager Page

import { useState, useEffect } from 'react';
import { Button } from '@/components/ui';
import { Plus, RefreshCw } from 'lucide-react';
import { useDatabaseStore } from '@/stores/databaseStore';
import { toast } from 'sonner';

export function ConnectionManager() {
  const { connections, setConnections } = useDatabaseStore();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadConnections();
  }, []);

  const loadConnections = async () => {
    setLoading(true);
    try {
      // TODO: Call listDatabaseConnectionsCmd from tauriCommands
      // const conns = await listDatabaseConnectionsCmd();
      // setConnections(conns);
      toast.success('Connections loaded');
    } catch (error) {
      toast.error('Failed to load connections: ' + String(error));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Database Connections</h1>
        <div className="flex gap-2">
          <Button onClick={loadConnections} disabled={loading}>
            <RefreshCw className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          <Button>
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
            <div key={conn.id} className="border rounded-lg p-4">
              <div className="flex justify-between">
                <div>
                  <h3 className="font-semibold">{conn.name}</h3>
                  <p className="text-sm text-muted-foreground">
                    {conn.db_type} • {conn.host}:{conn.port}
                  </p>
                </div>
                <div className="flex gap-2">
                  <Button size="sm" variant="outline">Test</Button>
                  <Button size="sm" variant="outline">Edit</Button>
                  <Button size="sm" variant="destructive">Delete</Button>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
