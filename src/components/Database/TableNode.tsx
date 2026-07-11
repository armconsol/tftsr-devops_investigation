// React Flow Custom Node for ER Diagrams

import { memo } from 'react';
import { Handle, Position } from 'reactflow';
import { Table } from 'lucide-react';

export interface TableNodeData {
  name: string;
  columns: Array<{
    name: string;
    data_type: string;
    primary_key: boolean;
  }>;
}

export const TableNode = memo(({ data }: { data: TableNodeData }) => {
  return (
    <div className="bg-background border-2 border-primary rounded-lg shadow-lg min-w-[200px]">
      {/* Table Header */}
      <div className="bg-primary text-primary-foreground px-3 py-2 rounded-t-md flex items-center gap-2">
        <Table className="w-4 h-4" />
        <span className="font-semibold">{data.name}</span>
      </div>

      {/* Columns */}
      <div className="p-2">
        {data.columns.map((column, index) => (
          <div
            key={index}
            className="px-2 py-1 text-sm flex items-center justify-between hover:bg-muted rounded"
          >
            <span className="flex items-center gap-1">
              {column.primary_key && <span className="text-yellow-500">🔑</span>}
              <span className="font-mono">{column.name}</span>
            </span>
            <span className="text-xs text-muted-foreground">{column.data_type}</span>
          </div>
        ))}
      </div>

      {/* Connection Handles */}
      <Handle
        type="target"
        position={Position.Left}
        className="w-2 h-2 !bg-primary"
      />
      <Handle
        type="source"
        position={Position.Right}
        className="w-2 h-2 !bg-primary"
      />
    </div>
  );
});

TableNode.displayName = 'TableNode';
