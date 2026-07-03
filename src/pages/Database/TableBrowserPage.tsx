import { useCallback, useEffect, useRef, useState } from 'react';
import { Button, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui';
import { Database, RefreshCw } from 'lucide-react';
import { TableBrowser } from '@/components/Database/TableBrowser';
import { useDatabaseStore } from '@/stores/databaseStore';
import {
  getDatabasesCmd,
  getTablesCmd,
  listDatabaseConnectionsCmd,
  type DatabaseConnection,
} from '@/lib/tauriCommands';
import { toast } from 'sonner';

export function TableBrowserPage() {
  const { connections, setConnections, activeConnectionId, setActiveConnection } = useDatabaseStore();
  const [loadingConnections, setLoadingConnections] = useState(false);
  const [loadingDatabases, setLoadingDatabases] = useState(false);
  const [loadingTables, setLoadingTables] = useState(false);
  const [databases, setDatabases] = useState<string[]>([]);
  const [tables, setTables] = useState<string[]>([]);
  const [selectedDatabase, setSelectedDatabase] = useState('');
  const [selectedTable, setSelectedTable] = useState('');
  const activeConnectionIdRef = useRef(activeConnectionId);

  useEffect(() => {
    activeConnectionIdRef.current = activeConnectionId;
  }, [activeConnectionId]);

  const loadConnections = useCallback(async () => {
    setLoadingConnections(true);
    try {
      const loaded = await listDatabaseConnectionsCmd();
      setConnections(loaded);

      if (loaded.length === 0) {
        setActiveConnection(null);
        return;
      }

      const nextConnectionId =
        activeConnectionIdRef.current &&
        loaded.some((connection) => connection.id === activeConnectionIdRef.current)
          ? activeConnectionIdRef.current
          : loaded[0].id;
      setActiveConnection(nextConnectionId);
    } catch (error) {
      toast.error('Failed to load connections: ' + String(error));
    } finally {
      setLoadingConnections(false);
    }
  }, [setActiveConnection, setConnections]);

  const loadTables = useCallback(async (connectionId: string, database: string) => {
    if (!database) {
      setTables([]);
      setSelectedTable('');
      return;
    }

    setLoadingTables(true);
    try {
      const loadedTables = await getTablesCmd(connectionId, database);
      setTables(loadedTables);
      setSelectedTable(loadedTables[0] ?? '');
    } catch (error) {
      setTables([]);
      setSelectedTable('');
      toast.error(`Failed to load tables for ${database}: ` + String(error));
    } finally {
      setLoadingTables(false);
    }
  }, []);

  const loadDatabases = useCallback(
    async (connectionId: string) => {
      setLoadingDatabases(true);
      try {
        const loadedDatabases = await getDatabasesCmd(connectionId);
        setDatabases(loadedDatabases);

        const nextDatabase = loadedDatabases[0] ?? '';
        setSelectedDatabase(nextDatabase);
        await loadTables(connectionId, nextDatabase);
      } catch (error) {
        setDatabases([]);
        setTables([]);
        setSelectedDatabase('');
        setSelectedTable('');
        toast.error('Failed to load databases: ' + String(error));
      } finally {
        setLoadingDatabases(false);
      }
    },
    [loadTables]
  );

  useEffect(() => {
    void loadConnections();
  }, [loadConnections]);

  useEffect(() => {
    if (!activeConnectionId) {
      setDatabases([]);
      setTables([]);
      setSelectedDatabase('');
      setSelectedTable('');
      return;
    }

    void loadDatabases(activeConnectionId);
  }, [activeConnectionId, loadDatabases]);

  const handleConnectionChange = (connectionId: string) => {
    setActiveConnection(connectionId);
  };

  const handleDatabaseChange = (database: string) => {
    setSelectedDatabase(database);
    if (!activeConnectionId) {
      return;
    }

    void loadTables(activeConnectionId, database);
  };

  const handleRefresh = () => {
    void loadConnections();
  };

  return (
    <div className="flex flex-col h-full p-6 gap-4">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold">Table Browser</h1>
          <p className="text-sm text-muted-foreground">
            Browse table rows without writing SQL.
          </p>
        </div>

        <Button onClick={handleRefresh} disabled={loadingConnections}>
          <RefreshCw className={`w-4 h-4 mr-2 ${loadingConnections ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      <div className="grid gap-3 md:grid-cols-3">
        <div className="space-y-1">
          <label className="text-sm font-medium">Connection</label>
          <Select value={activeConnectionId || ''} onValueChange={handleConnectionChange}>
            <SelectTrigger>
              <SelectValue placeholder="Select connection" />
            </SelectTrigger>
            <SelectContent>
              {connections.map((connection: DatabaseConnection) => (
                <SelectItem key={connection.id} value={connection.id}>
                  <span className="flex items-center gap-2">
                    <Database className="w-4 h-4" />
                    {connection.name}
                  </span>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1">
          <label className="text-sm font-medium">Database</label>
          <Select value={selectedDatabase} onValueChange={handleDatabaseChange}>
            <SelectTrigger
              className={!activeConnectionId || loadingDatabases ? 'pointer-events-none opacity-50' : ''}
            >
              <SelectValue placeholder="Select database" />
            </SelectTrigger>
            <SelectContent>
              {databases.map((database) => (
                <SelectItem key={database} value={database}>
                  {database}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1">
          <label className="text-sm font-medium">Table</label>
          <Select value={selectedTable} onValueChange={setSelectedTable}>
            <SelectTrigger
              className={!selectedDatabase || loadingTables ? 'pointer-events-none opacity-50' : ''}
            >
              <SelectValue placeholder="Select table" />
            </SelectTrigger>
            <SelectContent>
              {tables.map((table) => (
                <SelectItem key={table} value={table}>
                  {table}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {!activeConnectionId ? (
        <div className="flex-1 flex items-center justify-center text-muted-foreground border rounded-lg">
          No database connection selected
        </div>
      ) : selectedDatabase && selectedTable ? (
        <div className="flex-1 min-h-0 border rounded-lg p-4 overflow-auto">
          <TableBrowser connectionId={activeConnectionId} database={selectedDatabase} table={selectedTable} />
        </div>
      ) : (
        <div className="flex-1 flex items-center justify-center text-muted-foreground border rounded-lg">
          {loadingDatabases || loadingTables ? 'Loading table browser...' : 'Select a database and table'}
        </div>
      )}
    </div>
  );
}
