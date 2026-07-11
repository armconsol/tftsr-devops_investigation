import { useState, useEffect, useCallback } from "react";
import { listProxmoxNodes, ProxmoxNodeSummary } from "@/lib/proxmoxClient";
import { useProxmoxStore } from "@/stores/proxmoxStore";

export interface UseProxmoxNodesResult {
  nodes: ProxmoxNodeSummary[];
  nodeNames: string[];
  selectedNode: string;
  setSelectedNode: (node: string) => void;
  loading: boolean;
  error: string | null;
  reload: () => void;
}

/**
 * Loads the nodes for a Proxmox cluster (datacenter) and selects one so
 * node-scoped pages (Network, Ceph, Updates, Node Details) can populate
 * immediately instead of forcing the user to type a node name.
 *
 * The selection is persisted per-cluster in `useProxmoxStore` so switching
 * between Proxmox subpages keeps the node the user picked instead of
 * resetting back to the first node every time.
 */
export function useProxmoxNodes(clusterId: string): UseProxmoxNodesResult {
  const [nodes, setNodes] = useState<ProxmoxNodeSummary[]>([]);
  const [selectedNode, setSelectedNodeState] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getPersistedNode = useProxmoxStore((s) => s.getSelectedNode);
  const persistSelectedNode = useProxmoxStore((s) => s.setSelectedNode);

  const setSelectedNode = useCallback(
    (node: string) => {
      setSelectedNodeState(node);
      if (clusterId) persistSelectedNode(clusterId, node);
    },
    [clusterId, persistSelectedNode]
  );

  const load = useCallback(
    async (cId: string) => {
      if (!cId) {
        setNodes([]);
        setSelectedNodeState("");
        setError(null);
        return;
      }
      setLoading(true);
      setError(null);
      try {
        const raw = await listProxmoxNodes(cId);
        const parsed: ProxmoxNodeSummary[] = (raw ?? [])
          .filter((n): n is ProxmoxNodeSummary => !!n && typeof n.node === "string")
          .sort((a, b) => a.node.localeCompare(b.node));
        setNodes(parsed);
        const persisted = getPersistedNode(cId);
        setSelectedNodeState((prev) => {
          let resolved: string;
          if (prev && parsed.some((n) => n.node === prev)) {
            resolved = prev;
          } else if (persisted && parsed.some((n) => n.node === persisted)) {
            resolved = persisted;
          } else {
            resolved = parsed.length > 0 ? parsed[0].node : "";
          }
          // Keep the shared store in sync with whatever ends up selected —
          // including an implicit default, not just an explicit user pick —
          // so another page for the same cluster sees it too.
          if (resolved) persistSelectedNode(cId, resolved);
          return resolved;
        });
      } catch (e) {
        setNodes([]);
        setSelectedNodeState("");
        setError(String(e));
      } finally {
        setLoading(false);
      }
    },
    [getPersistedNode, persistSelectedNode]
  );

  useEffect(() => {
    // Reset local selection when the cluster changes so a node name from a
    // different datacenter isn't briefly shown; `load` restores the
    // persisted selection for this cluster once nodes are fetched.
    setSelectedNodeState("");
    void load(clusterId);
  }, [clusterId, load]);

  const reload = useCallback(() => void load(clusterId), [clusterId, load]);

  return {
    nodes,
    nodeNames: nodes.map((n) => n.node),
    selectedNode,
    setSelectedNode,
    loading,
    error,
    reload,
  };
}
