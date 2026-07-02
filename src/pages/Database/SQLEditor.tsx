// SQL Editor Page with Multi-Tab Support

import { useState, useEffect } from 'react';
import { Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui';
import { Play, Save, FileText, Database, Wand2 } from 'lucide-react';
import { MonacoSQLEditor } from '@/components/Database/MonacoSQLEditor';
import { QueryResultsPanel } from '@/components/Database/QueryResultsPanel';
import { EditorTabs } from '@/components/Database/EditorTabs';
import { useDatabaseStore } from '@/stores/databaseStore';
import {
  executeDatabaseQueryCmd,
  createQueryBookmarkCmd,
  getQueryHistoryCmd,
  listDatabaseConnectionsCmd,
  explainQueryCmd,
  type QueryHistory,
  type ExplainResult,
} from '@/lib/tauriCommands';
import { formatSQLForDatabase } from '@/lib/sqlFormatter';
import { toast } from 'sonner';
import { ExplainPlanVisualization } from '@/components/Database/ExplainPlanVisualization';

export function SQLEditor() {
  const {
    connections,
    setConnections,
    editorTabs,
    activeTabId,
    addTab,
    closeTab,
    switchTab,
    updateTabQuery,
    updateTabConnection,
    updateTabResults,
    setTabExecuting,
    getActiveTab,
    queryHistory,
    setQueryHistory,
  } = useDatabaseStore();

  const [page] = useState(0);
  const [pageSize] = useState(100);
  const [bookmarkDialog, setBookmarkDialog] = useState(false);
  const [bookmarkName, setBookmarkName] = useState('');
  const [explainDialog, setExplainDialog] = useState(false);
  const [explainResult, setExplainResult] = useState<ExplainResult | null>(null);
  const [explainLoading, setExplainLoading] = useState(false);

  const activeTab = getActiveTab();

  useEffect(() => {
    loadConnections();
    if (editorTabs.length > 0 && !activeTabId) {
      switchTab(editorTabs[0].id);
    }
  }, []);

  useEffect(() => {
    if (activeTab?.connectionId) {
      loadQueryHistory(activeTab.connectionId);
    }
  }, [activeTab?.connectionId]);

  const loadConnections = async () => {
    try {
      const conns = await listDatabaseConnectionsCmd();
      setConnections(conns);
      if (conns.length > 0 && activeTab && !activeTab.connectionId) {
        updateTabConnection(activeTab.id, conns[0].id);
      }
    } catch (error) {
      toast.error('Failed to load connections: ' + String(error));
    }
  };

  const loadQueryHistory = async (connectionId: string) => {
    try {
      const history = await getQueryHistoryCmd(connectionId, 50);
      setQueryHistory(history);
    } catch (error) {
      console.error('Failed to load query history:', error);
    }
  };

  const handleExecute = async () => {
    const tab = getActiveTab();
    if (!tab) {
      toast.error('No active tab');
      return;
    }
    if (!tab.connectionId) {
      toast.error('Please select a database connection');
      return;
    }
    if (!tab.content.trim()) {
      toast.error('Please enter a query');
      return;
    }

    setTabExecuting(tab.id, true);

    try {
      const result = await executeDatabaseQueryCmd(tab.connectionId, tab.content, page, pageSize);
      updateTabResults(tab.id, result, null);
      toast.success(`Query executed: ${result.total_rows} rows in ${result.execution_time_ms}ms`);
      if (tab.connectionId) {
        loadQueryHistory(tab.connectionId);
      }
    } catch (error) {
      const errorMsg = String(error);
      updateTabResults(tab.id, null, errorMsg);
      toast.error('Query failed: ' + errorMsg);
    }
  };

  const handleExplain = async () => {
    const tab = getActiveTab();
    if (!tab) return;
    if (!tab.connectionId) {
      toast.error('Please select a database connection');
      return;
    }
    if (!tab.content.trim()) {
      toast.error('Please enter a query');
      return;
    }

    setExplainLoading(true);
    setExplainDialog(true);
    try {
      const result = await explainQueryCmd(tab.connectionId, tab.content);
      setExplainResult(result);
    } catch (error) {
      toast.error('Failed to generate plan: ' + String(error));
      setExplainResult(null);
    } finally {
      setExplainLoading(false);
    }
  };

  const handleFormat = () => {
    const tab = getActiveTab();
    if (!tab || !tab.connectionId || !tab.content) return;

    const conn = connections.find((c) => c.id === tab.connectionId);
    if (conn) {
      const formatted = formatSQLForDatabase(tab.content, conn.db_type);
      updateTabQuery(tab.id, formatted);
      toast.success('Query formatted');
    }
  };

  const handleSaveBookmark = async () => {
    const tab = getActiveTab();
    if (!bookmarkName.trim()) {
      toast.error('Please enter a bookmark name');
      return;
    }
    if (!tab) {
      toast.error('No active tab');
      return;
    }

    try {
      await createQueryBookmarkCmd({
        name: bookmarkName,
        query_text: tab.content,
        connection_id: tab.connectionId || undefined,
      });
      toast.success('Bookmark saved');
      setBookmarkDialog(false);
      setBookmarkName('');
    } catch (error) {
      toast.error('Failed to save bookmark: ' + String(error));
    }
  };

  const handleHistoryItemClick = (item: QueryHistory) => {
    const tab = getActiveTab();
    if (tab) {
      updateTabQuery(tab.id, item.query_text);
      toast.info('Query loaded from history');
    }
  };

  const handleConnectionChange = (connectionId: string) => {
    const tab = getActiveTab();
    if (tab) {
      updateTabConnection(tab.id, connectionId);
    }
  };

  const activeConnection = activeTab?.connectionId
    ? connections.find((c) => c.id === activeTab.connectionId)
    : null;

  return (
    <div className="flex flex-col h-full">
      <div className="p-4 border-b flex items-center justify-between gap-4">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-bold">SQL Editor</h1>
          <Select value={activeTab?.connectionId || ''} onValueChange={handleConnectionChange}>
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
          <Button
            onClick={() => setBookmarkDialog(true)}
            variant="outline"
            disabled={!activeTab?.content}
          >
            <Save className="w-4 h-4 mr-2" />
            Save
          </Button>
          <Button
            onClick={handleExplain}
            variant="outline"
            disabled={!activeTab?.content || !activeTab?.connectionId}
          >
            <Wand2 className="w-4 h-4 mr-2" />
            Explain
          </Button>
          <Button
            onClick={handleExecute}
            disabled={activeTab?.isExecuting || !activeTab?.connectionId}
          >
            <Play className="w-4 h-4 mr-2" />
            Execute (Ctrl+Enter)
          </Button>
        </div>
      </div>

      <div className="flex-1 flex flex-col overflow-hidden">
        <EditorTabs
          tabs={editorTabs}
          activeTabId={activeTabId}
          onTabChange={switchTab}
          onTabClose={closeTab}
          onTabAdd={addTab}
          onTabUpdate={(tabId, content) => updateTabQuery(tabId, content)}
        >
          {(tab) => (
            <div className="flex flex-col" style={{ height: 'calc(100vh - 200px)' }}>
              <div className="flex-1 p-4 overflow-hidden">
                <MonacoSQLEditor
                  value={tab.content}
                  onChange={(value) => updateTabQuery(tab.id, value)}
                  onExecute={handleExecute}
                  height="100%"
                />
              </div>

              <div className="border-t" style={{ height: '350px' }}>
                <QueryResultsPanel
                  queryResult={tab.results}
                  executionError={tab.error}
                  queryHistory={queryHistory}
                  onHistoryItemClick={handleHistoryItemClick}
                  connectionId={tab.connectionId}
                />
              </div>
            </div>
          )}
        </EditorTabs>
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
              {activeTab?.content || ''}
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

      <Dialog open={explainDialog} onOpenChange={setExplainDialog}>
        <DialogContent className="max-w-5xl max-h-[80vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>Query Execution Plan</DialogTitle>
          </DialogHeader>
          <div className="flex-1 overflow-auto">
            {explainLoading ? (
              <div className="flex items-center justify-center h-64 text-muted-foreground">
                Generating execution plan...
              </div>
            ) : explainResult ? (
              <ExplainPlanVisualization result={explainResult} />
            ) : (
              <div className="flex items-center justify-center h-64 text-muted-foreground">
                No plan available
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
