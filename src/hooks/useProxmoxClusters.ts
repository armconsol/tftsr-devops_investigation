import { useState, useEffect, useCallback, useRef } from "react";
import { listProxmoxClusters } from "@/lib/proxmoxClient";
import type { ClusterInfo } from "@/lib/domain";
import { useProxmoxStore } from "@/stores/proxmoxStore";

export interface UseProxmoxClustersResult {
  clusters: ClusterInfo[];
  selectedClusterId: string;
  setSelectedClusterId: (id: string) => void;
  loading: boolean;
  error: string | null;
  reload: () => void;
}

/**
 * Loads Proxmox clusters and selects one, persisting the choice in
 * `useProxmoxStore` so every Proxmox subpage shares the same selected host
 * instead of each page defaulting back to the first cluster on navigation.
 *
 * `filter` narrows the cluster list (e.g. PBS-only pages) without affecting
 * the shared persisted selection for other pages.
 */
export function useProxmoxClusters(
  filter?: (cluster: ClusterInfo) => boolean
): UseProxmoxClustersResult {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterIdState] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const persistSelectedClusterId = useProxmoxStore((s) => s.setSelectedClusterId);

  // Keep the latest filter in a ref so callers can pass an inline arrow
  // function without that new reference identity re-triggering the load
  // effect (and re-fetching clusters) on every render.
  const filterRef = useRef(filter);
  useEffect(() => {
    filterRef.current = filter;
  }, [filter]);

  const setSelectedClusterId = useCallback(
    (id: string) => {
      setSelectedClusterIdState(id);
      persistSelectedClusterId(id);
    },
    [persistSelectedClusterId]
  );

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const all = await listProxmoxClusters();
      const activeFilter = filterRef.current;
      const filtered = activeFilter ? all.filter(activeFilter) : all;
      // Read the store directly (not via the reactive selector) so this
      // callback doesn't change identity every time any page updates the
      // shared selection — that would re-trigger a full cluster reload here.
      const persistedClusterId = useProxmoxStore.getState().selectedClusterId;
      setClusters(filtered);
      setSelectedClusterIdState((prev) => {
        let resolved: string;
        if (prev && filtered.some((c) => c.id === prev)) {
          resolved = prev;
        } else if (persistedClusterId && filtered.some((c) => c.id === persistedClusterId)) {
          resolved = persistedClusterId;
        } else {
          resolved = filtered.length > 0 ? filtered[0].id : "";
        }
        // Keep the shared store in sync with whatever ends up selected here —
        // including an implicit default, not just an explicit user pick —
        // so other (unfiltered) pages see it too. Skip this when a filter is
        // active: a filtered view's fallback (e.g. PBS-only) is only valid
        // for its own restricted list and must not overwrite the global
        // selection that unfiltered pages rely on.
        if (resolved && !activeFilter) {
          persistSelectedClusterId(resolved);
        }
        return resolved;
      });
    } catch (e) {
      setClusters([]);
      setSelectedClusterIdState("");
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [persistSelectedClusterId]);

  useEffect(() => {
    void load();
  }, [load]);

  return {
    clusters,
    selectedClusterId,
    setSelectedClusterId,
    loading,
    error,
    reload: () => void load(),
  };
}
