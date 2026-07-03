// SQL Query Preview for Visual Query Builder

import { Button } from '@/components/ui';
import { Copy } from 'lucide-react';
import { toast } from 'sonner';

interface QueryPreviewProps {
  sql: string;
}

export function QueryPreview({ sql }: QueryPreviewProps) {
  const handleCopy = () => {
    if (!sql.trim()) {
      toast.error('No SQL to copy');
      return;
    }
    navigator.clipboard.writeText(sql);
    toast.success('SQL copied to clipboard');
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-2">
        <h3 className="font-semibold text-sm">Generated SQL</h3>
        <Button onClick={handleCopy} size="sm" variant="ghost" disabled={!sql}>
          <Copy className="w-3 h-3 mr-1" />
          Copy
        </Button>
      </div>
      <pre className="flex-1 p-3 bg-muted rounded text-xs font-mono overflow-auto whitespace-pre-wrap">
        {sql || (
          <span className="text-muted-foreground">
            Drag tables onto the canvas to generate a SQL query
          </span>
        )}
      </pre>
    </div>
  );
}
