// SQL Editor Page

import { useState, useEffect } from 'react';
import { Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui';
import { Play, Save, FileText, Database } from 'lucide-react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { MonacoSQLEditor } from '@/components/Database/MonacoSQLEditor';
import { QueryResultsPanel } from '@/components/Database/QueryResultsPanel';
import { useDatabaseStore } from '@/stores/databaseStore';
import {
  executeDatabaseQueryCmd,
  createQueryBookmarkCmd,
  getQueryHistoryCmd,
  listDatabaseConnectionsCmd,
  type QueryHistory,
} from '@/lib/tauriCommands';
import { formatSQLForDatabase } from '@/lib/sqlFormatter';
import { toast } from 'sonner';

export function SQLEditor() {
  const {
    connections,
    setConnections,
    activeConnectionId,
    setActiveConnection,
    queryText,
    setQueryText,
    queryResults,
    setQueryResults,
    isExecuting,
    setIsExecuting,
    executionError,
    setExecutionError,
    queryHistory,
    setQueryHistory,
  } = useDatabaseStore();

  const [page, setPage] = useState(0);
  const [pageSize] = useState(100);
  const [bookmarkDialog, setBookmarkDialog] = useState(false);
  const [bookmarkName, setBookmarkName] = useState('');

  useEffect(() => {
    loadConnections();
  }, []);

  useEffect(() => {
    if (activeConnectionId) {
      loadQueryHistory();
    }
  }, [activeConnectionId]);

  const loadConnections = async () => {
    try {
      const conns = await listDatabaseConnectionsCmd();
      setConnections(conns);
      if (conns.length > 0 && !activeConnectionId) {
        setActiveConnection(conns[0].id);
      }
    } catch (error) {
      toast.error('Failed to load connections: ' + String(error));
    }
  };

  const loadQueryHistory = async () => {
    if (!activeConnectionId) return;

    try {
      const history = await getQueryHistoryCmd(activeConnectionId, 50);
      setQueryHistory(history);
    } catch (error) {
      console.error('Failed to load query history:', error);
    }
  };

  const handleExecute = async () => {
    if (!activeConnectionId) {
      toast.error('Please select a database connection');
      return;
    }

    if (!queryText.trim()) {
      toast.error('Please enter a query');
      return;
    }

    setIsExecuting(true);
    setExecutionError(null);

    try {
      const result = await executeDatabaseQueryCmd(activeConnectionId, queryText, page, pageSize);
      setQueryResults(result);
      toast.success(`Query executed: ${result.total_rows} rows in ${result.execution_time_ms}ms`);
      loadQueryHistory(); // Refresh history
    } catch (error) {
      const errorMsg = String(error);
      setExecutionError(errorMsg);
      toast.error('Query failed: ' + errorMsg);
    } finally {
      setIsExecuting(false);
    }
  };

  const handleFormat = () => {
    if (!activeConnectionId || !queryText) return;

    const conn = connections.find((c) => c.id === activeConnectionId);
    if (conn) {
      const formatted = formatSQLForDatabase(queryText, conn.db_type);
      setQueryText(formatted);
      toast.success('Query formatted');
    }
  };

  const handleSaveBookmark = async () => {
    if (!bookmarkName.trim()) {
      toast.error('Please enter a bookmark name');
      return;
    }

    try {
      await createQueryBookmarkCmd({
        name: bookmarkName,
        query_text: queryText,
        connection_id: activeConnectionId || undefined,
      });
      toast.success('Bookmark saved');
      setBookmarkDialog(false);
      setBookmarkName('');
    } catch (error) {
      toast.error('Failed to save bookmark: ' + String(error));
    }
  };

  const handleHistoryItemClick = (item: QueryHistory) => {
    setQueryText(item.query_text);
    toast.info('Query loaded from history');
  };

  const activeConnection = connections.find((c) => c.id === activeConnectionId);

  return (
    <div className="flex flex-col h-full">
      <div className="p-4 border-b flex items-center justify-between gap-4">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-bold">SQL Editor</h1>
          <Select value={activeConnectionId || ''} onValueChange={setActiveConnection}>
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
          <Button onClick={handleFormat} variant="outline" size="sm">
            <FileText className="w-4 h-4 mr-2" />
            Format
          </Button>
          <Button onClick={() => setBookmarkDialog(true)} variant="outline" disabled={!queryText}>
            <Save className="w-4 h-4 mr-2" />
            Save
          </Button>
          <Button onClick={handleExecute} disabled={isExecuting || !activeConnectionId}>
            <Play className="w-4 h-4 mr-2" />
            Execute (Ctrl+Enter)
          </Button>
        </div>
      </div>

      <div className="flex-1 flex flex-col overflow-hidden">
        <div className="flex-1 p-4 overflow-hidden">
          <MonacoSQLEditor
            value={queryText}
            onChange={setQueryText}
            onExecute={handleExecute}
            height="100%"
          />
        </div>

        <div className="border-t" style={{ height: '350px' }}>
          <QueryResultsPanel
            queryResult={queryResults}
            executionError={executionError}
            queryHistory={queryHistory}
            onHistoryItemClick={handleHistoryItemClick}
          />
        </div>
      </div>

      <Dialog open={bookmarkDialog} onOpenChange={setBookmarkDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Save Query Bookmark</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium">Bookmark Name</label>
              <Input
                value={bookmarkName}
                onChange={(e) => setBookmarkName(e.target.value)}
                placeholder="e.g., Get Active Users"
              />
            </div>
            <pre className="p-2 bg-muted rounded text-xs overflow-x-auto max-h-32">
              {queryText}
            </pre>
            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => setBookmarkDialog(false)}>
                Cancel
              </Button>
              <Button onClick={handleSaveBookmark}>Save</Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
