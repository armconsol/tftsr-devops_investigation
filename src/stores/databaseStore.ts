// Database management store

import { create } from 'zustand';
import type {
  DatabaseConnection,
  QueryResult,
  QueryHistory,
  QueryBookmark,
} from '@/lib/tauriCommands';

interface DatabaseState {
  // Connections
  connections: DatabaseConnection[];
  activeConnectionId: string | null;

  // Query state
  queryText: string;
  queryResults: QueryResult | null;
  isExecuting: boolean;
  executionError: string | null;

  // UI state
  selectedDatabase: string | null;
  selectedTable: string | null;

  // History and Bookmarks
  queryHistory: QueryHistory[];
  queryBookmarks: QueryBookmark[];

  // Actions
  setConnections: (connections: DatabaseConnection[]) => void;
  setActiveConnection: (id: string | null) => void;
  setQueryText: (text: string) => void;
  setQueryResults: (results: QueryResult | null) => void;
  setIsExecuting: (executing: boolean) => void;
  setExecutionError: (error: string | null) => void;
  setSelectedDatabase: (database: string | null) => void;
  setSelectedTable: (table: string | null) => void;
  setQueryHistory: (history: QueryHistory[]) => void;
  setQueryBookmarks: (bookmarks: QueryBookmark[]) => void;
  clearQueryResults: () => void;
}

export const useDatabaseStore = create<DatabaseState>((set) => ({
  // Initial state
  connections: [],
  activeConnectionId: null,
  queryText: '',
  queryResults: null,
  isExecuting: false,
  executionError: null,
  selectedDatabase: null,
  selectedTable: null,
  queryHistory: [],
  queryBookmarks: [],

  // Actions
  setConnections: (connections) => set({ connections }),
  setActiveConnection: (id) => set({ activeConnectionId: id }),
  setQueryText: (text) => set({ queryText: text }),
  setQueryResults: (results) => set({ queryResults: results }),
  setIsExecuting: (executing) => set({ isExecuting: executing }),
  setExecutionError: (error) => set({ executionError: error }),
  setSelectedDatabase: (database) => set({ selectedDatabase: database }),
  setSelectedTable: (table) => set({ selectedTable: table }),
  setQueryHistory: (history) => set({ queryHistory: history }),
  setQueryBookmarks: (bookmarks) => set({ queryBookmarks: bookmarks }),
  clearQueryResults: () => set({ queryResults: null, executionError: null }),
}));
