// Query Builder Sidebar — draggable table list

import { Database, FileText } from 'lucide-react';
import type { SchemaTable } from '@/pages/Database/QueryBuilder';

interface QueryBuilderSidebarProps {
  tables: SchemaTable[];
}

export function QueryBuilderSidebar({ tables }: QueryBuilderSidebarProps) {
  return (
    <div className="w-64 border-r bg-muted/30 overflow-y-auto h-full">
      <div className="p-4">
        <div className="flex items-center gap-2 mb-3">
          <Database className="w-5 h-5" />
          <h2 className="font-semibold">Available Tables</h2>
        </div>
        <div className="text-xs text-muted-foreground mb-3">
          Drag tables onto the canvas
        </div>
        <div className="space-y-2">
          {tables.length === 0 ? (
            <div className="text-xs text-muted-foreground p-2">
              No tables. Select a connection to load schema.
            </div>
          ) : (
            tables.map((table) => (
              <div
                key={table.name}
                draggable
                onDragStart={(e) => {
                  e.dataTransfer.setData('application/reactflow', JSON.stringify(table));
                  e.dataTransfer.effectAllowed = 'move';
                }}
                className="p-3 bg-card border rounded-lg cursor-move hover:border-primary transition-colors"
              >
                <div className="flex items-center gap-2">
                  <FileText className="w-4 h-4" />
                  <span className="font-medium text-sm">{table.name}</span>
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  {table.columns.length} columns
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
