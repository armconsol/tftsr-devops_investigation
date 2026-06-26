import { useState, useEffect, useCallback } from "react";
import { listProxmoxNodes, ProxmoxNodeSummary } from "@/lib/proxmoxClient";

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
 * Loads the nodes for a Proxmox cluster (datacenter) and auto-selects the first
 * one so node-scoped pages (Network, Ceph, Updates, Node Details) can populate
 * immediately instead of forcing the user to type a node name. Re-loads and
 * re-selects whenever the selected cluster changes.
 */
export function useProxmoxNodes(clusterId: string): UseProxmoxNodesResult {
  const [nodes, setNodes] = useState<ProxmoxNodeSummary[]>([]);
  const [selectedNode, setSelectedNode] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async (cId: string) => {
    if (!cId) {
      setNodes([]);
      setSelectedNode("");
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
      setSelectedNode((prev) => {
        if (prev && parsed.some((n) => n.node === prev)) return prev;
        return parsed.length > 0 ? parsed[0].node : "";
      });
    } catch (e) {
      setNodes([]);
      setSelectedNode("");
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // Reset the selection when the cluster changes so we don't keep a node name
    // that belongs to a different datacenter.
    setSelectedNode("");
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
