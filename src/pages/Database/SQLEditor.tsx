// SQL Editor Page

import { useState } from 'react';
import { Button } from '@/components/ui';
import { Play, Save } from 'lucide-react';
import { useDatabaseStore } from '@/stores/databaseStore';
import { toast } from 'sonner';

export function SQLEditor() {
  const { queryText, setQueryText, isExecuting } = useDatabaseStore();

  const handleExecute = async () => {
    try {
      // TODO: Call executeDatabaseQueryCmd
      toast.success('Query executed');
    } catch (error) {
      toast.error('Query failed: ' + String(error));
    }
  };

  return (
    <div className="flex flex-col h-full p-6">
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">SQL Editor</h1>
        <div className="flex gap-2">
          <Button onClick={handleExecute} disabled={isExecuting}>
            <Play className="w-4 h-4 mr-2" />
            Execute (Ctrl+Enter)
          </Button>
          <Button variant="outline">
            <Save className="w-4 h-4 mr-2" />
            Save as Bookmark
          </Button>
        </div>
      </div>

      <div className="flex-1 border rounded-lg">
        <textarea
          className="w-full h-full p-4 font-mono text-sm resize-none"
          placeholder="Enter SQL query here..."
          value={queryText}
          onChange={(e) => setQueryText(e.target.value)}
        />
      </div>

      <div className="mt-4 border rounded-lg p-4 h-64 overflow-auto">
        <p className="text-sm text-muted-foreground">Query results will appear here</p>
      </div>
    </div>
  );
}
