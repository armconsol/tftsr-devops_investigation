// Database management store

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type {
  DatabaseConnection,
  QueryResult,
  QueryHistory,
  QueryBookmark,
} from '@/lib/tauriCommands';

export interface EditorTab {
  id: string;
  title: string;
  content: string;
  connectionId: string | null;
  results: QueryResult | null;
  error: string | null;
  isExecuting: boolean;
}

interface DatabaseState {
  // Connections
  connections: DatabaseConnection[];
  activeConnectionId: string | null;

  // Editor Tabs
  editorTabs: EditorTab[];
  activeTabId: string;

  // Legacy single-tab state (kept for backwards compatibility, but not used)
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

  // Legacy single-tab actions (kept for backwards compatibility)
  setQueryText: (text: string) => void;
  setQueryResults: (results: QueryResult | null) => void;
  setIsExecuting: (executing: boolean) => void;
  setExecutionError: (error: string | null) => void;

  // Tab management actions
  addTab: () => void;
  closeTab: (id: string) => void;
  switchTab: (id: string) => void;
  updateTabQuery: (id: string, query: string) => void;
  updateTabConnection: (id: string, connectionId: string | null) => void;
  updateTabResults: (id: string, results: QueryResult | null, error: string | null) => void;
  setTabExecuting: (id: string, executing: boolean) => void;
  getActiveTab: () => EditorTab | undefined;

  setSelectedDatabase: (database: string | null) => void;
  setSelectedTable: (table: string | null) => void;
  setQueryHistory: (history: QueryHistory[]) => void;
  setQueryBookmarks: (bookmarks: QueryBookmark[]) => void;
  clearQueryResults: () => void;
}

const createDefaultTab = (index: number): EditorTab => ({
  id: crypto.randomUUID(),
  title: `Query ${index}`,
  content: '',
  connectionId: null,
  results: null,
  error: null,
  isExecuting: false,
});

export const useDatabaseStore = create<DatabaseState>()(
  persist(
    (set, get) => ({
      // Initial state
      connections: [],
      activeConnectionId: null,

      // Initialize with one default tab
      editorTabs: [createDefaultTab(1)],
      activeTabId: '',

      // Legacy state
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

      // Legacy actions
      setQueryText: (text) => set({ queryText: text }),
      setQueryResults: (results) => set({ queryResults: results }),
      setIsExecuting: (executing) => set({ isExecuting: executing }),
      setExecutionError: (error) => set({ executionError: error }),

      // Tab management
      addTab: () => {
        const state = get();
        const newTab = createDefaultTab(state.editorTabs.length + 1);
        set({
          editorTabs: [...state.editorTabs, newTab],
          activeTabId: newTab.id,
        });
      },

      closeTab: (id) => {
        const state = get();
        if (state.editorTabs.length <= 1) return; // Prevent closing last tab

        const newTabs = state.editorTabs.filter((t) => t.id !== id);
        const newActiveTabId =
          state.activeTabId === id
            ? newTabs[0].id
            : state.activeTabId;

        set({
          editorTabs: newTabs,
          activeTabId: newActiveTabId,
        });
      },

      switchTab: (id) => set({ activeTabId: id }),

      updateTabQuery: (id, query) => {
        const state = get();
        set({
          editorTabs: state.editorTabs.map((tab) =>
            tab.id === id ? { ...tab, content: query } : tab
          ),
        });
      },

      updateTabConnection: (id, connectionId) => {
        const state = get();
        set({
          editorTabs: state.editorTabs.map((tab) =>
            tab.id === id ? { ...tab, connectionId } : tab
          ),
        });
      },

      updateTabResults: (id, results, error) => {
        const state = get();
        set({
          editorTabs: state.editorTabs.map((tab) =>
            tab.id === id ? { ...tab, results, error, isExecuting: false } : tab
          ),
        });
      },

      setTabExecuting: (id, executing) => {
        const state = get();
        set({
          editorTabs: state.editorTabs.map((tab) =>
            tab.id === id ? { ...tab, isExecuting: executing } : tab
          ),
        });
      },

      getActiveTab: () => {
        const state = get();
        return state.editorTabs.find((t) => t.id === state.activeTabId);
      },

      setSelectedDatabase: (database) => set({ selectedDatabase: database }),
      setSelectedTable: (table) => set({ selectedTable: table }),
      setQueryHistory: (history) => set({ queryHistory: history }),
      setQueryBookmarks: (bookmarks) => set({ queryBookmarks: bookmarks }),
      clearQueryResults: () => set({ queryResults: null, executionError: null }),
    }),
    {
      name: 'tftsr-database-store',
      partialize: (state) => ({
        editorTabs: state.editorTabs,
        activeTabId: state.activeTabId,
      }),
    }
  )
);
