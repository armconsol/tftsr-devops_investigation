import React, { useCallback, useEffect, useState } from "react";
import { RefreshCw } from "lucide-react";
import { listCustomResourcesCmd } from "@/lib/tauriCommands";
import type { CustomResourceInfo } from "@/lib/tauriCommands";

interface CustomResourceListProps {
  clusterId: string;
  namespace: string;
  group: string;
  version: string;
  resource: string;
  kind: string;
}

export function CustomResourceList({
  clusterId,
  namespace,
  group,
  version,
  resource,
  kind,
}: CustomResourceListProps) {
  const [items, setItems] = useState<CustomResourceInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadItems = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listCustomResourcesCmd(clusterId, group, version, resource, namespace);
      setItems(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [clusterId, group, version, resource, namespace]);

  useEffect(() => {
    void loadItems();
  }, [loadItems]);

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-muted-foreground text-sm py-2">
        <RefreshCw className="h-4 w-4 animate-spin" />
        Loading {kind} instances…
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
        {error}
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <p className="text-sm text-muted-foreground py-2">
        No {kind} instances found.
      </p>
    );
  }

  const showNamespace = items.some((item) => item.namespace !== "");

  return (
    <div className="rounded-md border overflow-hidden">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b text-muted-foreground bg-muted/30">
            <th className="text-left px-4 py-2 font-medium">Name</th>
            {showNamespace && (
              <th className="text-left px-4 py-2 font-medium">Namespace</th>
            )}
            <th className="text-left px-4 py-2 font-medium">Age</th>
          </tr>
        </thead>
        <tbody>
          {items.map((item) => (
            <tr
              key={`${item.namespace}/${item.name}`}
              className="border-b last:border-0 hover:bg-muted/20 transition-colors"
            >
              <td className="px-4 py-2 font-mono text-xs font-medium">{item.name}</td>
              {showNamespace && (
                <td className="px-4 py-2 text-muted-foreground">{item.namespace || "—"}</td>
              )}
              <td className="px-4 py-2 text-muted-foreground">{item.age}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
