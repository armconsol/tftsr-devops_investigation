// Schema Explorer Page with Tree View

import { useState, useEffect } from 'react';
import { Button, Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui';
import { RefreshCw, Table, Eye } from 'lucide-react';
import { SchemaTree, type TreeNode } from '@/components/Database/SchemaTree';
import { TableBrowser } from '@/components/Database/TableBrowser';
import { useDatabaseStore } from '@/stores/databaseStore';
import { getDatabasesCmd, getTablesCmd, getTableSchemaCmd } from '@/lib/tauriCommands';
import { toast } from 'sonner';

export function SchemaExplorer() {
  const { activeConnectionId, setSelectedDatabase, setSelectedTable } = useDatabaseStore();
  const [treeNodes, setTreeNodes] = useState<TreeNode[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedNode, setSelectedNode] = useState<TreeNode | null>(null);
  const [browserOpen, setBrowserOpen] = useState(false);
  const [selectedTableContext, setSelectedTableContext] = useState<{
    database: string;
    table: string;
  } | null>(null);

  useEffect(() => {
    if (activeConnectionId) {
      loadDatabases();
    }
  }, [activeConnectionId]);

  async function loadDatabases() {
    if (!activeConnectionId) {
      toast.error('No active connection');
      return;
    }

    setLoading(true);
    try {
      const databases = await getDatabasesCmd(activeConnectionId);
      const nodes: TreeNode[] = databases.map((db) => ({
        id: `db-${db}`,
        label: db,
        type: 'database',
      }));
      setTreeNodes(nodes);
    } catch (error) {
      toast.error('Failed to load databases: ' + String(error));
    } finally {
      setLoading(false);
    }
  }

  const handleNodeExpand = async (nodeId: string): Promise<TreeNode[]> => {
    if (!activeConnectionId) return [];

    const parts = nodeId.split('-');
    const type = parts[0];
    const name = parts.slice(1).join('-');

    try {
      if (type === 'db') {
        // Load tables for database
        const tables = await getTablesCmd(activeConnectionId, name);
        return tables.map((table) => ({
          id: `table-${name}-${table}`,
          label: table,
          type: 'table',
        }));
      } else if (type === 'table') {
        // Load columns for table
        const dbName = parts[1];
        const tableName = parts.slice(2).join('-');
        const schema = await getTableSchemaCmd(activeConnectionId, dbName, tableName);

        return schema.columns.map((column) => ({
          id: `column-${dbName}-${tableName}-${column.name}`,
          label: column.name,
          type: 'column',
          metadata: {
            data_type: column.data_type,
            primary_key: column.primary_key,
            nullable: column.nullable,
          },
        }));
      }
    } catch (error) {
      toast.error('Failed to load schema: ' + String(error));
    }

    return [];
  };

  const handleNodeClick = (node: TreeNode) => {
    setSelectedNode(node);

    if (node.type === 'database') {
      const dbName = node.id.replace('db-', '');
      setSelectedDatabase(dbName);
    } else if (node.type === 'table') {
      const parts = node.id.split('-');
      const dbName = parts[1];
      const tableName = parts.slice(2).join('-');
      setSelectedDatabase(dbName);
      setSelectedTable(tableName);
      setSelectedTableContext({ database: dbName, table: tableName });
    }
  };

  const handleViewData = () => {
    if (!selectedNode || selectedNode.type !== 'table' || !selectedTableContext) {
      toast.error('Please select a table');
      return;
    }

    setBrowserOpen(true);
  };

  return (
    <div className="flex flex-col h-full p-6">
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">Schema Explorer</h1>
        <div className="flex gap-2">
          {selectedNode?.type === 'table' && (
            <Button onClick={handleViewData} variant="outline">
              <Eye className="w-4 h-4 mr-2" />
              View Data
            </Button>
          )}
          <Button onClick={loadDatabases} disabled={loading || !activeConnectionId}>
            <RefreshCw className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {!activeConnectionId ? (
        <div className="flex-1 flex items-center justify-center text-muted-foreground">
          <div className="text-center">
            <Table className="w-16 h-16 mx-auto mb-4 opacity-50" />
            <p className="text-lg">No Active Connection</p>
            <p className="text-sm mt-2">Select a database connection to explore its schema</p>
          </div>
        </div>
      ) : (
        <div className="flex-1 overflow-auto">
          <SchemaTree
            nodes={treeNodes}
            onNodeExpand={handleNodeExpand}
            onNodeClick={handleNodeClick}
            onNodeDoubleClick={handleNodeClick}
          />
        </div>
      )}

      {selectedNode && (
        <div className="mt-4 p-4 border rounded-lg bg-muted">
          <h3 className="font-semibold mb-2">Selected: {selectedNode.label}</h3>
          <p className="text-sm text-muted-foreground">Type: {selectedNode.type}</p>
          {selectedNode.metadata && (
            <div className="mt-2 text-sm">
              {selectedNode.metadata.data_type && (
                <p>Data Type: {selectedNode.metadata.data_type}</p>
              )}
              {selectedNode.metadata.primary_key && <p className="text-yellow-600">Primary Key</p>}
              {selectedNode.metadata.nullable !== undefined && (
                <p>Nullable: {selectedNode.metadata.nullable ? 'Yes' : 'No'}</p>
              )}
            </div>
          )}
        </div>
      )}

      <Dialog open={browserOpen} onOpenChange={setBrowserOpen}>
        <DialogContent className="max-w-7xl h-[85vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>
              Table Data
              {selectedTableContext
                ? ` — ${selectedTableContext.database}.${selectedTableContext.table}`
                : ''}
            </DialogTitle>
          </DialogHeader>
          <div className="flex-1 overflow-auto">
            {activeConnectionId && selectedTableContext ? (
              <TableBrowser
                connectionId={activeConnectionId}
                database={selectedTableContext.database}
                table={selectedTableContext.table}
              />
            ) : null}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
