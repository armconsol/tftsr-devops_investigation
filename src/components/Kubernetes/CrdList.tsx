import React, { useCallback, useEffect, useState } from "react";
import { RefreshCw, ChevronRight, ChevronDown } from "lucide-react";
import { Badge, Button } from "@/components/ui";
import { listCrdsCmd } from "@/lib/tauriCommands";
import type { CrdInfo } from "@/lib/tauriCommands";
import { CustomResourceList } from "./CustomResourceList";

interface CrdListProps {
  clusterId: string;
  onSelectCrd?: (crd: CrdInfo) => void;
}

function scopeVariant(scope: string): "default" | "secondary" {
  return scope === "Namespaced" ? "default" : "secondary";
}

export function CrdList({ clusterId, onSelectCrd }: CrdListProps) {
  const [crds, setCrds] = useState<CrdInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedCrd, setExpandedCrd] = useState<string | null>(null);

  const loadCrds = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listCrdsCmd(clusterId);
      setCrds(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [clusterId]);

  useEffect(() => {
    void loadCrds();
  }, [loadCrds]);

  const handleRowClick = (crd: CrdInfo) => {
    const key = crd.name;
    setExpandedCrd((prev) => (prev === key ? null : key));
    onSelectCrd?.(crd);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-32 text-muted-foreground">
        <RefreshCw className="h-5 w-5 animate-spin mr-2" />
        Loading CRDs…
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <span className="text-sm text-muted-foreground">
          {crds.length} custom resource definition{crds.length !== 1 ? "s" : ""}
        </span>
        <Button size="sm" variant="outline" onClick={() => void loadCrds()}>
          <RefreshCw className="h-3.5 w-3.5 mr-1" />
          Refresh
        </Button>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </div>
      )}

      <div className="border rounded-md overflow-hidden">
        {crds.length === 0 ? (
          <div className="flex items-center justify-center h-32 text-muted-foreground text-sm">
            No custom resource definitions found
          </div>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b text-muted-foreground">
                <th className="text-left px-4 py-3 font-medium">Name</th>
                <th className="text-left px-4 py-3 font-medium">Kind</th>
                <th className="text-left px-4 py-3 font-medium">Group</th>
                <th className="text-left px-4 py-3 font-medium">Version</th>
                <th className="text-left px-4 py-3 font-medium">Scope</th>
                <th className="text-left px-4 py-3 font-medium">Age</th>
              </tr>
            </thead>
            <tbody>
              {crds.map((crd) => {
                const isExpanded = expandedCrd === crd.name;
                return (
                  <React.Fragment key={crd.name}>
                    <tr
                      className="border-b last:border-0 hover:bg-muted/30 transition-colors cursor-pointer"
                      onClick={() => handleRowClick(crd)}
                    >
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-1.5 font-mono text-xs">
                          {isExpanded ? (
                            <ChevronDown className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />
                          ) : (
                            <ChevronRight className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />
                          )}
                          {crd.name}
                        </div>
                      </td>
                      <td className="px-4 py-3 font-medium">{crd.kind}</td>
                      <td className="px-4 py-3 text-muted-foreground font-mono text-xs">{crd.group}</td>
                      <td className="px-4 py-3 font-mono text-xs">{crd.version}</td>
                      <td className="px-4 py-3">
                        <Badge variant={scopeVariant(crd.scope)}>
                          {crd.scope}
                        </Badge>
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">{crd.age}</td>
                    </tr>
                    {isExpanded && (
                      <tr className="border-b bg-muted/10">
                        <td colSpan={6} className="px-6 py-3">
                          <CustomResourceList
                            clusterId={clusterId}
                            namespace={crd.scope === "Namespaced" ? "" : ""}
                            group={crd.group}
                            version={crd.version}
                            resource={crd.plural}
                            kind={crd.kind}
                            printerColumns={
                              crd.versions.find((v) => v.name === crd.version)?.printer_columns || []
                            }
                          />
                        </td>
                      </tr>
                    )}
                  </React.Fragment>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
