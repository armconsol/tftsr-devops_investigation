// Query Execution Plan Visualization — renders EXPLAIN output as a tree

import { useMemo, useCallback } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  Position,
  type Node,
  type Edge,
} from 'reactflow';
import 'reactflow/dist/style.css';
import type { ExplainResult, ExplainNode } from '@/lib/tauriCommands';

interface ExplainPlanVisualizationProps {
  result: ExplainResult;
}

const NODE_WIDTH = 240;
const NODE_VERTICAL_SPACING = 120;
const NODE_HORIZONTAL_SPACING = 280;

function buildNodesAndEdges(
  root: ExplainNode,
  startX = 0,
  startY = 0,
  depth = 0,
  idCounter = { value: 0 }
): { nodes: Node[]; edges: Edge[]; width: number } {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  const nodeId = `n-${idCounter.value++}`;

  const childResults = (root.children || []).map((child) =>
    buildNodesAndEdges(child, 0, startY + NODE_VERTICAL_SPACING, depth + 1, idCounter)
  );

  const totalChildWidth = childResults.reduce((acc, r) => acc + r.width, 0);
  const subtreeWidth = Math.max(totalChildWidth, NODE_HORIZONTAL_SPACING);

  // Position this node at center of subtree
  const myX = startX + subtreeWidth / 2 - NODE_WIDTH / 2;

  // Build child positions
  let offset = startX;
  childResults.forEach((result) => {
    const childWidth = Math.max(result.width, NODE_HORIZONTAL_SPACING);
    // Shift child nodes by offset
    result.nodes.forEach((n) => {
      n.position = { x: n.position.x + offset, y: n.position.y };
      nodes.push(n);
    });
    result.edges.forEach((e) => edges.push(e));

    // Connect this node to first node of subtree (its root)
    const childRootId = result.nodes[0]?.id;
    if (childRootId) {
      edges.push({
        id: `e-${nodeId}-${childRootId}`,
        source: nodeId,
        target: childRootId,
        type: 'smoothstep',
        animated: false,
      });
    }
    offset += childWidth;
  });

  // Determine color by node type
  const getNodeColor = (nodeType: string): string => {
    const t = nodeType.toLowerCase();
    if (t.includes('seq scan') || t.includes('full table scan') || t.includes('all')) return '#fecaca';
    if (t.includes('index scan') || t.includes('index')) return '#bbf7d0';
    if (t.includes('join') || t.includes('hash')) return '#bfdbfe';
    if (t.includes('sort')) return '#fde68a';
    if (t.includes('aggregate') || t.includes('group')) return '#e9d5ff';
    return '#f3f4f6';
  };

  nodes.unshift({
    id: nodeId,
    position: { x: myX, y: startY },
    data: {
      label: (
        <div className="text-xs">
          <div className="font-bold mb-1">{root.node_type}</div>
          {root.relation_name && (
            <div className="text-gray-700">on {root.relation_name}</div>
          )}
          {root.index_name && (
            <div className="text-gray-700">using {root.index_name}</div>
          )}
          <div className="flex gap-2 mt-1 text-gray-600">
            {root.cost !== null && root.cost !== undefined && (
              <span>cost: {root.cost.toFixed(2)}</span>
            )}
            {root.rows !== null && root.rows !== undefined && (
              <span>rows: {root.rows}</span>
            )}
          </div>
          {root.actual_time_ms !== null && root.actual_time_ms !== undefined && (
            <div className="text-blue-600">actual: {root.actual_time_ms.toFixed(2)}ms</div>
          )}
          {root.extra && <div className="text-gray-500 italic mt-1">{root.extra}</div>}
        </div>
      ),
    },
    style: {
      background: getNodeColor(root.node_type),
      border: '1px solid #6b7280',
      borderRadius: '6px',
      padding: '8px',
      width: NODE_WIDTH,
      fontSize: '12px',
    },
    sourcePosition: Position.Bottom,
    targetPosition: Position.Top,
  });

  return { nodes, edges, width: subtreeWidth };
}

export function ExplainPlanVisualization({ result }: ExplainPlanVisualizationProps) {
  const { nodes, edges } = useMemo(() => {
    if (!result.plan) return { nodes: [], edges: [] };
    const built = buildNodesAndEdges(result.plan);
    return { nodes: built.nodes, edges: built.edges };
  }, [result]);

  const handleNodesChange = useCallback(() => {
    /* readonly */
  }, []);

  if (!result.plan) {
    return (
      <div className="p-4">
        <h3 className="font-semibold mb-2">Raw Execution Plan</h3>
        <pre className="p-3 bg-muted rounded text-xs overflow-auto max-h-96 whitespace-pre-wrap">
          {result.raw_output || 'No plan information available'}
        </pre>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="p-3 border-b bg-muted/30">
        <div className="flex gap-4 text-sm">
          <span><strong>DB:</strong> {result.database_type}</span>
          {result.total_cost !== null && result.total_cost !== undefined && (
            <span><strong>Total cost:</strong> {result.total_cost.toFixed(2)}</span>
          )}
          {result.execution_time_ms !== null && result.execution_time_ms !== undefined && (
            <span><strong>Plan time:</strong> {result.execution_time_ms.toFixed(2)}ms</span>
          )}
        </div>
      </div>

      <div style={{ height: '500px' }}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={handleNodesChange}
          fitView
          fitViewOptions={{ padding: 0.2 }}
          minZoom={0.2}
          maxZoom={2}
        >
          <Background />
          <Controls />
          <MiniMap />
        </ReactFlow>
      </div>

      {result.raw_output && (
        <details className="p-3 border-t bg-muted/30">
          <summary className="cursor-pointer text-sm font-medium">Raw output</summary>
          <pre className="mt-2 p-2 bg-background rounded text-xs overflow-auto max-h-48 whitespace-pre-wrap">
            {result.raw_output}
          </pre>
        </details>
      )}

      <div className="p-3 border-t text-xs text-muted-foreground">
        <strong>Color guide:</strong>{' '}
        <span className="inline-block px-2 py-0.5 rounded mr-1" style={{ background: '#fecaca' }}>Scan</span>
        <span className="inline-block px-2 py-0.5 rounded mr-1" style={{ background: '#bbf7d0' }}>Index</span>
        <span className="inline-block px-2 py-0.5 rounded mr-1" style={{ background: '#bfdbfe' }}>Join</span>
        <span className="inline-block px-2 py-0.5 rounded mr-1" style={{ background: '#fde68a' }}>Sort</span>
        <span className="inline-block px-2 py-0.5 rounded mr-1" style={{ background: '#e9d5ff' }}>Aggregate</span>
      </div>
    </div>
  );
}
