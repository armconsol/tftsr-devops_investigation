// Query Builder Canvas — React Flow workspace with table nodes and join edges

import { useCallback, memo } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  Handle,
  Position,
  addEdge,
  applyNodeChanges,
  applyEdgeChanges,
  type Node,
  type Edge,
  type Connection,
  type NodeChange,
  type EdgeChange,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { JoinEdge } from './JoinEdge';
import type { SchemaTable } from '@/pages/Database/QueryBuilder';

interface QueryBuilderTableNodeData {
  tableName: string;
  alias: string;
  columns: Array<{ name: string; data_type: string; primary_key: boolean }>;
  selectedColumns: Set<string>;
  onColumnToggle: (columnName: string, selected: boolean) => void;
}

const QueryBuilderTableNode = memo(({ data }: { data: QueryBuilderTableNodeData }) => {
  return (
    <div className="bg-background border-2 border-primary rounded-lg shadow-lg min-w-[220px]">
      <div className="bg-primary text-primary-foreground px-3 py-2 rounded-t-md">
        <div className="font-semibold text-sm">{data.tableName}</div>
        {data.alias !== data.tableName && (
          <div className="text-xs opacity-80">alias: {data.alias}</div>
        )}
      </div>
      <div className="p-2 max-h-64 overflow-y-auto">
        {data.columns.map((col) => {
          const handleId = col.name;
          return (
            <div
              key={col.name}
              className="px-2 py-1 text-xs flex items-center justify-between hover:bg-muted rounded relative"
            >
              <label className="flex items-center gap-2 cursor-pointer flex-1">
                <input
                  type="checkbox"
                  checked={data.selectedColumns.has(col.name)}
                  onChange={(e) => data.onColumnToggle(col.name, e.target.checked)}
                  className="w-3 h-3"
                />
                <span className="flex items-center gap-1">
                  {col.primary_key && <span className="text-yellow-500">🔑</span>}
                  <span className="font-mono">{col.name}</span>
                </span>
              </label>
              <span className="text-[10px] text-muted-foreground">{col.data_type}</span>
              <Handle
                type="source"
                position={Position.Right}
                id={handleId}
                style={{ top: 'auto', background: '#6366f1', width: 8, height: 8 }}
                isConnectable={true}
              />
              <Handle
                type="target"
                position={Position.Left}
                id={handleId}
                style={{ top: 'auto', background: '#6366f1', width: 8, height: 8 }}
                isConnectable={true}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
});

QueryBuilderTableNode.displayName = 'QueryBuilderTableNode';

const nodeTypes = { qbTable: QueryBuilderTableNode };
const edgeTypes = { joinEdge: JoinEdge };

interface QueryBuilderCanvasProps {
  nodes: Node[];
  edges: Edge[];
  setNodes: React.Dispatch<React.SetStateAction<Node[]>>;
  setEdges: React.Dispatch<React.SetStateAction<Edge[]>>;
  availableTables: SchemaTable[];
  onColumnSelect: (tableId: string, tableName: string, columnName: string, selected: boolean) => void;
}

export function QueryBuilderCanvas({
  nodes,
  edges,
  setNodes,
  setEdges,
  availableTables: _availableTables,
  onColumnSelect,
}: QueryBuilderCanvasProps) {
  const onNodesChange = useCallback(
    (changes: NodeChange[]) => setNodes((nds) => applyNodeChanges(changes, nds)),
    [setNodes]
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges((eds) => applyEdgeChanges(changes, eds)),
    [setEdges]
  );

  const onConnect = useCallback(
    (params: Connection) =>
      setEdges((eds) =>
        addEdge(
          {
            ...params,
            type: 'joinEdge',
            data: { joinType: 'INNER' },
          },
          eds
        )
      ),
    [setEdges]
  );

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      const tableData = event.dataTransfer.getData('application/reactflow');
      if (!tableData) return;

      try {
        const table: SchemaTable = JSON.parse(tableData);
        const id = `node-${crypto.randomUUID()}`;
        const bounds = (event.target as HTMLElement).getBoundingClientRect();
        const position = {
          x: event.clientX - bounds.left,
          y: event.clientY - bounds.top,
        };

        const selectedColumns = new Set<string>();

        const newNode: Node = {
          id,
          type: 'qbTable',
          position,
          data: {
            tableName: table.name,
            alias: table.name,
            columns: table.columns,
            selectedColumns,
            onColumnToggle: (columnName: string, selected: boolean) => {
              setNodes((nds) =>
                nds.map((n) => {
                  if (n.id === id) {
                    const sc = new Set(n.data.selectedColumns);
                    if (selected) sc.add(columnName);
                    else sc.delete(columnName);
                    return { ...n, data: { ...n.data, selectedColumns: sc } };
                  }
                  return n;
                })
              );
              onColumnSelect(id, table.name, columnName, selected);
            },
          },
        };

        setNodes((nds) => nds.concat(newNode));
      } catch (e) {
        console.error('Failed to parse dropped table:', e);
      }
    },
    [setNodes, onColumnSelect]
  );

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  return (
    <div onDrop={onDrop} onDragOver={onDragOver} style={{ width: '100%', height: '100%' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        fitView
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}
