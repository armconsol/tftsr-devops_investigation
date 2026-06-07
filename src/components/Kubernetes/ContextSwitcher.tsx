import React from "react";
import { Server } from "lucide-react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";

interface ContextSwitcherProps {
  clusters: { id: string; name: string; context: string; cluster_url?: string }[];
  selectedClusterId: string;
  onClusterChange: (clusterId: string) => void;
}

export function ContextSwitcher({ clusters, selectedClusterId, onClusterChange }: ContextSwitcherProps) {
  const selectedCluster = clusters.find((c) => c.id === selectedClusterId);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Server className="w-5 h-5" />
          Context Switcher
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div>
            <label className="text-sm font-medium text-muted-foreground mb-2 block">
              Current Cluster
            </label>
            <Select value={selectedClusterId} onValueChange={onClusterChange}>
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select cluster" />
              </SelectTrigger>
              <SelectContent>
                {clusters.map((cluster) => (
                  <SelectItem key={cluster.id} value={cluster.id}>
                    <div className="flex items-center gap-2">
                      <Server className="w-4 h-4" />
                      {cluster.name}
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {selectedCluster && (
            <div className="p-4 bg-muted rounded-md space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Context</span>
                <Badge variant="secondary">{selectedCluster.context}</Badge>
              </div>
              {selectedCluster.cluster_url && (
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Cluster URL</span>
                  <span className="text-sm font-mono truncate max-w-[200px]">{selectedCluster.cluster_url}</span>
                </div>
              )}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
