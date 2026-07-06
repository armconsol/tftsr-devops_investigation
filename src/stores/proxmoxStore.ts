import { create } from "zustand";
import { persist } from "zustand/middleware";

interface ProxmoxState {
  selectedClusterId: string;
  selectedNodeByCluster: Record<string, string>;
  setSelectedClusterId: (clusterId: string) => void;
  setSelectedNode: (clusterId: string, node: string) => void;
  getSelectedNode: (clusterId: string) => string;
}

/**
 * Persists the user's Proxmox cluster/node selection across every Proxmox
 * subpage (VMs, Ceph, Tasks, Administration, ...), which previously kept
 * this in per-page local state and reset it back to the default host on
 * every navigation.
 */
export const useProxmoxStore = create<ProxmoxState>()(
  persist(
    (set, get) => ({
      selectedClusterId: "",
      selectedNodeByCluster: {},

      setSelectedClusterId: (clusterId) => set({ selectedClusterId: clusterId }),

      setSelectedNode: (clusterId, node) =>
        set((state) => ({
          selectedNodeByCluster: { ...state.selectedNodeByCluster, [clusterId]: node },
        })),

      getSelectedNode: (clusterId) => get().selectedNodeByCluster[clusterId] ?? "",
    }),
    {
      name: "tftsr-proxmox",
    }
  )
);
