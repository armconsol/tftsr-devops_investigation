// Query History Page

import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button, Input } from '@/components/ui';
import { RefreshCw, Search, Play, Trash2 } from 'lucide-react';
import { useDatabaseStore } from '@/stores/databaseStore';
import { getQueryHistoryCmd, searchQueryHistoryCmd } from '@/lib/tauriCommands';
import { toast } from 'sonner';
import type { QueryHistory } from '@/stores/databaseStore';

export function QueryHistoryPage() {
  const navigate = useNavigate();
  const { activeConnectionId, updateTabQuery, getActiveTab } = useDatabaseStore();
  const [history, setHistory] = useState<QueryHistory[]>([]);
  const [filteredHistory, setFilteredHistory] = useState<QueryHistory[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (activeConnectionId) {
      loadHistory();
    }
  }, [activeConnectionId]);

  useEffect(() => {
    if (searchTerm) {
      const filtered = history.filter((item) =>
        item.query_text.toLowerCase().includes(searchTerm.toLowerCase())
      );
      setFilteredHistory(filtered);
    } else {
      setFilteredHistory(history);
    }
  }, [searchTerm, history]);

  const loadHistory = async () => {
    if (!activeConnectionId) {
      toast.error('No active connection');
      return;
    }

    setLoading(true);
    try {
      const result = await getQueryHistoryCmd(activeConnectionId, 100);
      setHistory(result);
      setFilteredHistory(result);
    } catch (error) {
      toast.error('Failed to load query history: ' + String(error));
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = async () => {
    if (!activeConnectionId || !searchTerm) return;

    setLoading(true);
    try {
      const results = await searchQueryHistoryCmd(activeConnectionId, searchTerm, 100);
      setFilteredHistory(results);
    } catch (error) {
      toast.error('Search failed: ' + String(error));
    } finally {
      setLoading(false);
    }
  };

  const handleReExecute = (item: QueryHistory) => {
    const activeTab = getActiveTab();
    if (activeTab) {
      updateTabQuery(activeTab.id, item.query_text);
      navigate('/database/editor');
      toast.success('Query loaded in editor');
    } else {
      toast.error('No active editor tab');
    }
  };

  const formatDuration = (ms: number | null) => {
    if (ms === null) return 'N/A';
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  return (
    <div className="flex flex-col h-full p-6">
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">Query History</h1>
        <Button onClick={loadHistory} disabled={loading || !activeConnectionId}>
          <RefreshCw className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      <div className="flex gap-2 mb-4">
        <div className="flex-1 flex gap-2">
          <Input
            placeholder="Search queries..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
          />
          <Button onClick={handleSearch} disabled={loading || !searchTerm}>
            <Search className="w-4 h-4 mr-2" />
            Search
          </Button>
        </div>
      </div>

      {!activeConnectionId ? (
        <div className="flex-1 flex items-center justify-center text-muted-foreground">
          <div className="text-center">
            <p className="text-lg">No Active Connection</p>
            <p className="text-sm mt-2">Select a database connection to view query history</p>
          </div>
        </div>
      ) : (
        <div className="flex-1 overflow-auto">
          {filteredHistory.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              <p>No query history found</p>
              {searchTerm && <p className="text-sm mt-1">Try a different search term</p>}
            </div>
          ) : (
            <div className="space-y-2">
              {filteredHistory.map((item) => (
                <div
                  key={item.id}
                  className="border rounded-lg p-4 hover:bg-muted transition-colors"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <pre className="font-mono text-sm bg-muted p-2 rounded overflow-x-auto">
                        {item.query_text}
                      </pre>

                      <div className="flex items-center gap-4 mt-2 text-sm text-muted-foreground">
                        <span>{new Date(item.executed_at).toLocaleString()}</span>
                        <span>
                          {item.status === 'success' ? (
                            <span className="text-green-600 font-medium">
                              ✓ {item.row_count || 0} rows
                            </span>
                          ) : (
                            <span className="text-red-600 font-medium">✗ Error</span>
                          )}
                        </span>
                        <span>{formatDuration(item.execution_time_ms)}</span>
                      </div>

                      {item.error_message && (
                        <div className="mt-2 text-sm text-red-600 bg-red-50 p-2 rounded">
                          {item.error_message}
                        </div>
                      )}
                    </div>

                    <div className="flex gap-2">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleReExecute(item)}
                        title="Re-execute query"
                      >
                        <Play className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      <div className="mt-4 text-sm text-muted-foreground">
        Showing {filteredHistory.length} of {history.length} queries
      </div>
    </div>
  );
}
