// ER Diagram Page with React Flow

import { useState, useEffect, useCallback, useRef } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  addEdge,
  useNodesState,
  useEdgesState,
  useReactFlow,
  getRectOfNodes,
  getTransformForBounds,
  type Node,
  type Edge,
  type Connection,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { toPng } from 'html-to-image';
import { Button, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui';
import { RefreshCw, Download } from 'lucide-react';
import { TableNode, type TableNodeData } from '@/components/Database/TableNode';
import { useDatabaseStore } from '@/stores/databaseStore';
import { generateErDiagramCmd, getDatabasesCmd } from '@/lib/tauriCommands';
import { toast } from 'sonner';

const nodeTypes = {
  table: TableNode,
};

export function ERDiagram() {
  const { activeConnectionId } = useDatabaseStore();
  const { getNodes } = useReactFlow();
  const diagramRef = useRef<HTMLDivElement>(null);
  const [databases, setDatabases] = useState<string[]>([]);
  const [selectedDatabase, setSelectedDatabase] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  useEffect(() => {
    if (activeConnectionId) {
      loadDatabases();
    }
  }, [activeConnectionId]);

  const loadDatabases = async () => {
    if (!activeConnectionId) return;

    try {
      const dbs = await getDatabasesCmd(activeConnectionId);
      setDatabases(dbs);
      if (dbs.length > 0) {
        setSelectedDatabase(dbs[0]);
      }
    } catch (error) {
      toast.error('Failed to load databases: ' + String(error));
    }
  };

  const loadDiagram = async () => {
    if (!activeConnectionId || !selectedDatabase) {
      toast.error('Please select a database');
      return;
    }

    setLoading(true);
    try {
      const diagram = await generateErDiagramCmd(activeConnectionId, selectedDatabase);

      // Convert backend diagram data to React Flow nodes and edges
      const flowNodes: Node[] = diagram.tables.map((table: any) => ({
        id: table.name,
        type: 'table',
        position: table.position || { x: Math.random() * 500, y: Math.random() * 500 },
        data: {
          name: table.name,
          columns: table.columns,
        } as TableNodeData,
      }));

      const flowEdges: Edge[] = diagram.relationships.map((rel: any, index: number) => ({
        id: `edge-${index}`,
        source: rel.from_table,
        target: rel.to_table,
        label: `${rel.from_column} → ${rel.to_column}`,
        type: 'default',
        animated: true,
        style: { stroke: '#64748b', strokeWidth: 2 },
      }));

      setNodes(flowNodes);
      setEdges(flowEdges);

      toast.success(`Loaded ${flowNodes.length} tables`);
    } catch (error) {
      toast.error('Failed to generate ER diagram: ' + String(error));
    } finally {
      setLoading(false);
    }
  };

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const handleExport = async () => {
    if (!diagramRef.current || nodes.length === 0) {
      toast.error('No diagram to export');
      return;
    }

    try {
      const nodesBounds = getRectOfNodes(getNodes());
      const transform = getTransformForBounds(
        nodesBounds,
        nodesBounds.width,
        nodesBounds.height,
        0.5,
        2
      );

      const dataUrl = await toPng(diagramRef.current, {
        backgroundColor: '#ffffff',
        width: nodesBounds.width,
        height: nodesBounds.height,
        style: {
          width: String(nodesBounds.width),
          height: String(nodesBounds.height),
          transform: `translate(${transform[0]}px, ${transform[1]}px) scale(${transform[2]})`,
        },
      });

      const link = document.createElement('a');
      link.download = `er-diagram-${selectedDatabase || 'database'}.png`;
      link.href = dataUrl;
      link.click();

      toast.success('Diagram exported as PNG');
    } catch (error) {
      toast.error('Export failed: ' + String(error));
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="p-4 border-b flex items-center justify-between gap-4">
        <h1 className="text-2xl font-bold">ER Diagram</h1>

        <div className="flex items-center gap-2">
          <Select value={selectedDatabase} onValueChange={setSelectedDatabase}>
            <SelectTrigger className="w-64">
              <SelectValue placeholder="Select database" />
            </SelectTrigger>
            <SelectContent>
              {databases.map((db) => (
                <SelectItem key={db} value={db}>
                  {db}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Button onClick={loadDiagram} disabled={loading || !selectedDatabase}>
            <RefreshCw className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
            Generate
          </Button>

          <Button onClick={handleExport} variant="outline" disabled={nodes.length === 0}>
            <Download className="w-4 h-4 mr-2" />
            Export
          </Button>
        </div>
      </div>

      <div className="flex-1" ref={diagramRef}>
        {nodes.length === 0 ? (
          <div className="h-full flex items-center justify-center text-muted-foreground">
            <div className="text-center">
              <p className="text-lg">No Diagram Generated</p>
              <p className="text-sm mt-2">
                Select a database and click "Generate" to visualize relationships
              </p>
            </div>
          </div>
        ) : (
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            nodeTypes={nodeTypes}
            fitView
          >
            <Background />
            <Controls />
            <MiniMap />
          </ReactFlow>
        )}
      </div>

      {nodes.length > 0 && (
        <div className="p-2 border-t text-sm text-muted-foreground">
          {nodes.length} tables • {edges.length} relationships
        </div>
      )}
    </div>
  );
}
