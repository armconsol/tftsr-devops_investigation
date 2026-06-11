import React from 'react';
import { Card } from '@/components/ui/index';
import { ChevronRight, ChevronDown, Folder, File, Server, Database, Cloud } from 'lucide-react';

interface ResourceNode {
  id: string;
  name: string;
  type: 'remote' | 'cluster' | 'node' | 'vm' | 'ct' | 'storage' | 'datastore' | 'sdn-zone';
  children?: ResourceNode[];
  status?: 'online' | 'offline' | 'error' | 'running' | 'stopped';
  metadata?: Record<string, string>;
}

interface ResourceTreeProps {
  nodes: ResourceNode[];
  onNodeSelect?: (node: ResourceNode) => void;
  onNodeExpand?: (node: ResourceNode) => void;
  onNodeCollapse?: (node: ResourceNode) => void;
  selectedNode?: ResourceNode;
  expandedNodes?: Set<string>;
  onToggleExpand?: (nodeId: string) => void;
  filter?: string;
  className?: string;
}

export function ResourceTree({
  nodes,
  onNodeSelect,
  onNodeExpand,
  onNodeCollapse,
  selectedNode,
  expandedNodes = new Set<string>(),
  onToggleExpand,
  filter = '',
  className = '',
}: ResourceTreeProps) {
  const [expanded, setExpanded] = React.useState<Set<string>>(expandedNodes);

  const handleToggleExpand = (node: ResourceNode) => {
    const newExpanded = new Set(expanded);
    if (newExpanded.has(node.id)) {
      newExpanded.delete(node.id);
      onNodeCollapse?.(node);
    } else {
      newExpanded.add(node.id);
      onNodeExpand?.(node);
    }
    setExpanded(newExpanded);
    onToggleExpand?.(node.id);
  };

  const renderNode = (node: ResourceNode, depth = 0) => {
    const isExpanded = expanded.has(node.id);
    const hasChildren = node.children && node.children.length > 0;
    const isSelected = selectedNode?.id === node.id;

    const getIcon = () => {
      switch (node.type) {
        case 'remote':
          return <Cloud className="h-4 w-4 text-blue-500" />;
        case 'cluster':
          return <Database className="h-4 w-4 text-green-500" />;
        case 'node':
          return <Server className="h-4 w-4 text-orange-500" />;
        case 'vm':
          return <File className="h-4 w-4 text-purple-500" />;
        case 'ct':
          return <File className="h-4 w-4 text-teal-500" />;
        case 'storage':
        case 'datastore':
          return <Folder className="h-4 w-4 text-yellow-500" />;
        case 'sdn-zone':
          return <Cloud className="h-4 w-4 text-indigo-500" />;
        default:
          return <Folder className="h-4 w-4 text-gray-500" />;
      }
    };

    const getBadge = () => {
      if (!node.status) return null;
      const statusColors: Record<string, string> = {
        online: 'bg-green-500',
        running: 'bg-green-500',
        offline: 'bg-red-500',
        error: 'bg-red-500',
        stopped: 'bg-gray-500',
      };
      return (
        <span
          className={`h-2 w-2 rounded-full ${statusColors[node.status] || 'bg-gray-500'}`}
        />
      );
    };

    return (
      <div key={node.id}>
        <div
          className={`flex items-center py-1 px-2 cursor-pointer hover:bg-accent rounded-md ${
            isSelected ? 'bg-accent' : ''
          }`}
          style={{ paddingLeft: `${depth * 16 + 8}px` }}
          onClick={() => onNodeSelect?.(node)}
        >
          <button
            className="mr-1 p-0.5 hover:bg-accent rounded"
            onClick={(e) => {
              e.stopPropagation();
              handleToggleExpand(node);
            }}
          >
            {hasChildren ? (
              isExpanded ? (
                <ChevronDown className="h-4 w-4" />
              ) : (
                <ChevronRight className="h-4 w-4" />
              )
            ) : (
              <span className="w-4" />
            )}
          </button>
          {getIcon()}
          <span className="ml-2 flex-1 truncate">{node.name}</span>
          {getBadge()}
        </div>
        {hasChildren && isExpanded && (
          <div>
            {node.children!.map((child) => renderNode(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  const filteredNodes = filter
    ? nodes.filter((node) =>
        node.name.toLowerCase().includes(filter.toLowerCase())
      )
    : nodes;

  return (
    <Card className={`p-2 overflow-auto ${className}`}>
      <div className="space-y-0.5">{filteredNodes.map((node) => renderNode(node))}</div>
    </Card>
  );
}
