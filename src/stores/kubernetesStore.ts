import { create } from "zustand";
import type { ClusterInfo, ContextInfo, ResourceInfo } from "@/lib/tauriCommands";

export type ResourceType = 
  | "pods" 
  | "services" 
  | "deployments" 
  | "statefulsets" 
  | "daemonsets" 
  | "replicasets" 
  | "jobs" 
  | "cronjobs" 
  | "ingresses" 
  | "persistentvolumes" 
  | "persistentvolumeclaims" 
  | "configmaps" 
  | "secrets" 
  | "serviceaccounts" 
  | "roles" 
  | "clusterroles" 
  | "rolebindings" 
  | "clusterrolebindings" 
  | "nodes" 
  | "events" 
  | "hpas";

interface KubernetesState {
  // Selection state
  selectedClusterId: string | null;
  selectedNamespace: string;
  
  // Data state
  clusters: ClusterInfo[];
  contexts: ContextInfo[];
  namespaces: Record<string, string[]>; // clusterId -> [namespaces]
  
  // Loaded resources tracking
  loadedResources: Set<ResourceType>;
  
  // Terminal sessions
  terminalSessions: Record<string, { 
    id: string; 
    clusterId: string; 
    namespace: string; 
    pod: string; 
    container: string; 
    command: string 
  }>;
  nextTerminalId: number;
  
  // Search state
  globalSearchQuery: string;
  searchResults: Record<ResourceType, ResourceInfo[]>;
  
  // Bulk selection
  bulkSelection: Record<ResourceType, string[]>; // resourceType -> [resourceNames]
  
  // Actions
  setSelectedCluster: (clusterId: string) => void;
  setSelectedNamespace: (namespace: string) => void;
  addCluster: (cluster: ClusterInfo) => void;
  removeCluster: (clusterId: string) => void;
  updateCluster: (clusterId: string, updates: Partial<ClusterInfo>) => void;
  addContext: (context: ContextInfo) => void;
  setNamespaces: (clusterId: string, namespaces: string[]) => void;
  markResourceLoaded: (type: ResourceType) => void;
  markResourceUnloaded: (type: ResourceType) => void;
  isResourceLoaded: (type: ResourceType) => boolean;
  addTerminalSession: (session: { clusterId: string; namespace: string; pod: string; container: string; command: string }) => string;
  removeTerminalSession: (sessionId: string) => void;
  setGlobalSearchQuery: (query: string) => void;
  setSearchResults: (type: ResourceType, results: ResourceInfo[]) => void;
  addToBulkSelection: (type: ResourceType, resourceName: string) => void;
  removeFromBulkSelection: (type: ResourceType, resourceName: string) => void;
  clearBulkSelection: (type: ResourceType) => void;
  getBulkSelectionCount: (type: ResourceType) => number;
}

export const useKubernetesStore = create<KubernetesState>()((set, get) => ({
  // Selection state
  selectedClusterId: null,
  selectedNamespace: "all",
  
  // Data state
  clusters: [],
  contexts: [],
  namespaces: {},
  
  // Loaded resources tracking
  loadedResources: new Set<ResourceType>() as Set<ResourceType>,
  
  // Terminal sessions
  terminalSessions: {},
  nextTerminalId: 1,
  
  // Search state
  globalSearchQuery: "",
  searchResults: {} as Record<ResourceType, ResourceInfo[]>,
  
  // Bulk selection
  bulkSelection: {} as Record<ResourceType, string[]>,
  
  // Actions
  setSelectedCluster: (clusterId) => set({ selectedClusterId: clusterId, selectedNamespace: "all" }),
  
  setSelectedNamespace: (namespace) => set({ selectedNamespace: namespace }),
  
  addCluster: (cluster) => set((state) => ({
    clusters: [...state.clusters, cluster],
  })),
  
  removeCluster: (clusterId) => set((state) => ({
    clusters: state.clusters.filter((c) => c.id !== clusterId),
    selectedClusterId: state.selectedClusterId === clusterId ? null : state.selectedClusterId,
  })),
  
  updateCluster: (clusterId, updates) => set((state) => ({
    clusters: state.clusters.map((c) => 
      c.id === clusterId ? { ...c, ...updates } : c
    ),
  })),
  
  addContext: (context) => set((state) => ({
    contexts: [...state.contexts, context],
  })),
  
  setNamespaces: (clusterId, namespaces) => set((state) => ({
    namespaces: { ...state.namespaces, [clusterId]: namespaces },
  })),
  
  markResourceLoaded: (type) => set((state) => {
    const newSet = new Set(state.loadedResources);
    newSet.add(type);
    return { loadedResources: newSet };
  }),
  
  markResourceUnloaded: (type) => set((state) => {
    const newSet = new Set(state.loadedResources);
    newSet.delete(type);
    return { loadedResources: newSet };
  }),
  
  isResourceLoaded: (type) => get().loadedResources.has(type),
  
  addTerminalSession: (session) => {
    const sessionId = `terminal-${get().nextTerminalId}`;
    set((state) => ({
      terminalSessions: { ...state.terminalSessions, [sessionId]: { id: sessionId, ...session } },
      nextTerminalId: state.nextTerminalId + 1,
    }));
    return sessionId;
  },
  
  removeTerminalSession: (sessionId) => set((state) => ({
    terminalSessions: Object.fromEntries(
      Object.entries(state.terminalSessions).filter(([id]) => id !== sessionId)
    ),
  })),
  
  setGlobalSearchQuery: (query) => set({ globalSearchQuery: query }),
  
  setSearchResults: (type, results) => set((state) => ({
    searchResults: { ...state.searchResults, [type]: results },
  })),
  
  addToBulkSelection: (type, resourceName) => set((state) => ({
    bulkSelection: {
      ...state.bulkSelection,
      [type]: [...(state.bulkSelection[type] || []), resourceName],
    },
  })),
  
  removeFromBulkSelection: (type, resourceName) => set((state) => ({
    bulkSelection: {
      ...state.bulkSelection,
      [type]: (state.bulkSelection[type] || []).filter((name) => name !== resourceName),
    },
  })),
  
  clearBulkSelection: (type) => set((state) => ({
    bulkSelection: { ...state.bulkSelection, [type]: [] },
  })),
  
  getBulkSelectionCount: (type) => (get().bulkSelection[type] || []).length,
}));
