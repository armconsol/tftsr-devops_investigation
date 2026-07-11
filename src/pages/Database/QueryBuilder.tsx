// Visual Query Builder Page

import { useState, useEffect, useCallback } from 'react';
import { Button, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui';
import { Database, Play, Copy } from 'lucide-react';
import { QueryBuilderCanvas } from '@/components/Database/QueryBuilderCanvas';
import { WhereClauseBuilder, type WhereCondition } from '@/components/Database/WhereClauseBuilder';
import { QueryPreview } from '@/components/Database/QueryPreview';
import { useDatabaseStore } from '@/stores/databaseStore';
import {
  listDatabaseConnectionsCmd,
  getDatabasesCmd,
  getSchemaCmd,
} from '@/lib/tauriCommands';
import { QueryBuilderSidebar } from '@/components/Database/QueryBuilderSidebar';
import { toast } from 'sonner';
import { useNavigate } from 'react-router-dom';
import type { Node, Edge } from 'reactflow';

export interface TableColumn {
  name: string;
  data_type: string;
  primary_key: boolean;
  nullable: boolean;
}

export interface SchemaTable {
  name: string;
  columns: TableColumn[];
}

export interface SelectedColumn {
  tableId: string;
  tableName: string;
  columnName: string;
}

export function QueryBuilder() {
  const navigate = useNavigate();
  const { connections, setConnections, setQueryText } = useDatabaseStore();
  const [activeConnectionId, setActiveConnectionId] = useState<string | null>(null);
  const [availableTables, setAvailableTables] = useState<SchemaTable[]>([]);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [selectedColumns, setSelectedColumns] = useState<SelectedColumn[]>([]);
  const [whereConditions, setWhereConditions] = useState<WhereCondition[]>([]);
  const [generatedSQL, setGeneratedSQL] = useState<string>('');

  useEffect(() => {
    loadConnections();
  }, []);

  useEffect(() => {
    if (activeConnectionId) {
      loadSchema(activeConnectionId);
    }
  }, [activeConnectionId]);

  useEffect(() => {
    generateSQL();
  }, [nodes, edges, selectedColumns, whereConditions]);

  const loadConnections = async () => {
    try {
      const conns = await listDatabaseConnectionsCmd();
      setConnections(conns);
      if (conns.length > 0 && !activeConnectionId) {
        setActiveConnectionId(conns[0].id);
      }
    } catch (error) {
      toast.error('Failed to load connections: ' + String(error));
    }
  };

  const loadSchema = async (connectionId: string) => {
    try {
      const databases = await getDatabasesCmd(connectionId);
      const tables: SchemaTable[] = [];

      for (const dbName of databases) {
        try {
          const schema = await getSchemaCmd(connectionId, dbName);
          for (const tableEntry of schema.tables) {
            tables.push({
              name: tableEntry.name,
              columns: tableEntry.columns.map((col) => ({
                name: col.name,
                data_type: col.data_type,
                primary_key: col.primary_key,
                nullable: col.nullable,
              })),
            });
          }
        } catch (err) {
          // Skip databases we can't read
          console.warn(`Failed to load schema for ${dbName}:`, err);
        }
      }

      setAvailableTables(tables);
    } catch (error) {
      toast.error('Failed to load schema: ' + String(error));
    }
  };

  const generateSQL = useCallback(() => {
    if (nodes.length === 0) {
      setGeneratedSQL('');
      return;
    }

    const parts: string[] = [];

    // SELECT clause
    if (selectedColumns.length > 0) {
      const selectCols = selectedColumns.map((sc) => {
        const table = nodes.find((n) => n.id === sc.tableId);
        const alias = table?.data.alias || sc.tableName;
        return `${alias}.${sc.columnName}`;
      });
      parts.push(`SELECT ${selectCols.join(', ')}`);
    } else {
      parts.push('SELECT *');
    }

    // FROM clause
    const firstTable = nodes[0];
    const firstAlias = firstTable.data.alias || firstTable.data.tableName;
    parts.push(`FROM ${firstTable.data.tableName} AS ${firstAlias}`);

    // JOIN clauses
    for (const edge of edges) {
      const sourceNode = nodes.find((n) => n.id === edge.source);
      const targetNode = nodes.find((n) => n.id === edge.target);

      if (!sourceNode || !targetNode) continue;

      const sourceAlias = sourceNode.data.alias || sourceNode.data.tableName;
      const targetAlias = targetNode.data.alias || targetNode.data.tableName;
      const joinType = edge.data?.joinType || 'INNER';
      const sourceColumn = edge.sourceHandle || 'id';
      const targetColumn = edge.targetHandle || 'id';

      parts.push(
        `${joinType} JOIN ${targetNode.data.tableName} AS ${targetAlias} ON ${sourceAlias}.${sourceColumn} = ${targetAlias}.${targetColumn}`
      );
    }

    // WHERE clause
    if (whereConditions.length > 0) {
      const whereClauses = whereConditions.map((cond) => {
        const table = nodes.find((n) => n.id === cond.tableId);
        const alias = table?.data.alias || cond.tableName;
        const column = `${alias}.${cond.columnName}`;

        switch (cond.operator) {
          case 'IS NULL':
          case 'IS NOT NULL':
            return `${column} ${cond.operator}`;
          case 'IN':
            return `${column} IN (${cond.value})`;
          case 'BETWEEN':
            return `${column} BETWEEN ${cond.value}`;
          case 'LIKE':
            return `${column} LIKE '${cond.value}'`;
          default:
            return `${column} ${cond.operator} '${cond.value}'`;
        }
      });
      parts.push(`WHERE ${whereClauses.join(' AND ')}`);
    }

    const sql = parts.join('\n');
    setGeneratedSQL(sql);
  }, [nodes, edges, selectedColumns, whereConditions]);

  const handleSendToEditor = () => {
    if (!generatedSQL.trim()) {
      toast.error('No query to send');
      return;
    }

    setQueryText(generatedSQL);
    navigate('/database/editor');
    toast.success('Query sent to SQL Editor');
  };

  const handleCopySQL = () => {
    if (!generatedSQL.trim()) {
      toast.error('No query to copy');
      return;
    }

    navigator.clipboard.writeText(generatedSQL);
    toast.success('Query copied to clipboard');
  };

  const handleColumnSelect = (tableId: string, tableName: string, columnName: string, selected: boolean) => {
    if (selected) {
      setSelectedColumns((prev) => [...prev, { tableId, tableName, columnName }]);
    } else {
      setSelectedColumns((prev) =>
        prev.filter(
          (sc) => !(sc.tableId === tableId && sc.columnName === columnName)
        )
      );
    }
  };

  const activeConnection = connections.find((c) => c.id === activeConnectionId);

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b flex items-center justify-between gap-4">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-bold">Visual Query Builder</h1>
          <Select value={activeConnectionId || ''} onValueChange={setActiveConnectionId}>
            <SelectTrigger className="w-64">
              <SelectValue placeholder="Select connection" />
            </SelectTrigger>
            <SelectContent>
              {connections.map((conn) => (
                <SelectItem key={conn.id} value={conn.id}>
                  <div className="flex items-center gap-2">
                    <Database className="w-4 h-4" />
                    {conn.name}
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {activeConnection && (
            <span className="text-sm text-muted-foreground">
              {activeConnection.db_type} • {activeConnection.host}
            </span>
          )}
        </div>

        <div className="flex gap-2">
          <Button onClick={handleCopySQL} variant="outline" size="sm" disabled={!generatedSQL}>
            <Copy className="w-4 h-4 mr-2" />
            Copy SQL
          </Button>
          <Button onClick={handleSendToEditor} disabled={!generatedSQL}>
            <Play className="w-4 h-4 mr-2" />
            Send to Editor
          </Button>
        </div>
      </div>

      {/* Canvas + Sidebar */}
      <div className="flex-1 flex overflow-hidden">
        <QueryBuilderSidebar tables={availableTables} />

        {/* Main Canvas */}
        <div className="flex-1 flex flex-col">
          <div className="flex-1">
            <QueryBuilderCanvas
              nodes={nodes}
              edges={edges}
              setNodes={setNodes}
              setEdges={setEdges}
              availableTables={availableTables}
              onColumnSelect={handleColumnSelect}
            />
          </div>

          {/* Bottom Panel */}
          <div className="border-t" style={{ height: '250px' }}>
            <div className="flex h-full">
              {/* WHERE Clause Builder */}
              <div className="w-1/2 border-r p-4 overflow-y-auto">
                <WhereClauseBuilder
                  nodes={nodes}
                  conditions={whereConditions}
                  onConditionsChange={setWhereConditions}
                />
              </div>

              {/* SQL Preview */}
              <div className="w-1/2 p-4">
                <QueryPreview sql={generatedSQL} />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
