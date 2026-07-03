// Lazy-Loading Schema Tree Component

import { useState } from 'react';
import { ChevronRight, ChevronDown, Database, Table, Columns } from 'lucide-react';
import { Button } from '@/components/ui';

export interface TreeNode {
  id: string;
  label: string;
  type: 'database' | 'table' | 'column';
  children?: TreeNode[];
  isLoading?: boolean;
  isExpanded?: boolean;
  metadata?: {
    data_type?: string;
    primary_key?: boolean;
    nullable?: boolean;
  };
}

interface SchemaTreeProps {
  nodes: TreeNode[];
  onNodeExpand: (nodeId: string) => Promise<TreeNode[]>;
  onNodeClick?: (node: TreeNode) => void;
  onNodeDoubleClick?: (node: TreeNode) => void;
}

export function SchemaTree({
  nodes,
  onNodeExpand,
  onNodeClick,
  onNodeDoubleClick,
}: SchemaTreeProps) {
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());
  const [nodeChildren, setNodeChildren] = useState<Map<string, TreeNode[]>>(new Map());
  const [loadingNodes, setLoadingNodes] = useState<Set<string>>(new Set());

  const handleToggle = async (node: TreeNode) => {
    const isExpanded = expandedNodes.has(node.id);

    if (isExpanded) {
      setExpandedNodes((prev) => {
        const next = new Set(prev);
        next.delete(node.id);
        return next;
      });
    } else {
      // Load children if not already loaded
      if (!nodeChildren.has(node.id)) {
        setLoadingNodes((prev) => new Set(prev).add(node.id));
        try {
          const children = await onNodeExpand(node.id);
          setNodeChildren((prev) => new Map(prev).set(node.id, children));
        } catch (error) {
          console.error('Failed to load node children:', error);
        } finally {
          setLoadingNodes((prev) => {
            const next = new Set(prev);
            next.delete(node.id);
            return next;
          });
        }
      }

      setExpandedNodes((prev) => new Set(prev).add(node.id));
    }
  };

  const renderNode = (node: TreeNode, depth: number = 0) => {
    const isExpanded = expandedNodes.has(node.id);
    const isLoading = loadingNodes.has(node.id);
    const children = nodeChildren.get(node.id) || node.children || [];
    const hasChildren = node.type !== 'column';

    const icon = {
      database: <Database className="w-4 h-4" />,
      table: <Table className="w-4 h-4" />,
      column: <Columns className="w-4 h-4" />,
    }[node.type];

    return (
      <div key={node.id}>
        <div
          className="flex items-center gap-1 py-1 px-2 hover:bg-muted rounded cursor-pointer group"
          style={{ paddingLeft: `${depth * 20 + 8}px` }}
          onClick={() => {
            if (hasChildren) handleToggle(node);
            onNodeClick?.(node);
          }}
          onDoubleClick={() => onNodeDoubleClick?.(node)}
        >
          {hasChildren && (
            <button
              className="p-0.5 hover:bg-accent rounded"
              onClick={(e) => {
                e.stopPropagation();
                handleToggle(node);
              }}
            >
              {isLoading ? (
                <div className="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
              ) : isExpanded ? (
                <ChevronDown className="w-4 h-4" />
              ) : (
                <ChevronRight className="w-4 h-4" />
              )}
            </button>
          )}

          {!hasChildren && <div className="w-5" />}

          <span className="text-muted-foreground">{icon}</span>

          <span className="text-sm flex-1">
            {node.label}
            {node.metadata?.primary_key && ' 🔑'}
          </span>

          {node.metadata?.data_type && (
            <span className="text-xs text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity">
              {node.metadata.data_type}
            </span>
          )}
        </div>

        {isExpanded && children.length > 0 && (
          <div>{children.map((child) => renderNode(child, depth + 1))}</div>
        )}
      </div>
    );
  };

  return (
    <div className="border rounded-lg overflow-auto" style={{ maxHeight: '600px' }}>
      {nodes.length > 0 ? (
        <div className="p-2">{nodes.map((node) => renderNode(node))}</div>
      ) : (
        <div className="p-8 text-center text-muted-foreground">
          <Database className="w-12 h-12 mx-auto mb-2 opacity-50" />
          <p>No databases available</p>
          <p className="text-sm mt-1">Connect to a database to browse schema</p>
        </div>
      )}
    </div>
  );
}
