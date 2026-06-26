import React from "react";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Terminal } from "./Terminal";
import { SearchBar } from "./SearchBar";
import { MetricsChart } from "./MetricsChart";
import { YamlEditor } from "./YamlEditor";
import { useKubernetesStore } from "@/stores/kubernetesStore";
import { useStore } from "zustand";

interface ApplicationViewProps {
  clusterId: string;
  namespace: string;
}

export function ApplicationView({ clusterId, namespace }: ApplicationViewProps) {
  const [activeTab, setActiveTab] = React.useState("overview");
  const clusters = useStore(useKubernetesStore, (state) => state.clusters);
  const selectedCluster = clusters.find((c: { id: string }) => c.id === clusterId);

  return (
    <div className="h-full overflow-hidden flex flex-col">
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-xl font-semibold">Application View</h2>
          {selectedCluster && (
            <span className="text-sm text-muted-foreground">{selectedCluster.name}</span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <SearchBar
            query={useStore(useKubernetesStore, (state) => state.globalSearchQuery)}
            onQueryChange={(q) => useKubernetesStore.getState().setGlobalSearchQuery(q)}
          />
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-5 mb-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="workloads">Workloads</TabsTrigger>
          <TabsTrigger value="infrastructure">Infrastructure</TabsTrigger>
          <TabsTrigger value="terminal">Terminal</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="overview" className="h-full overflow-y-auto">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              <MetricsChart
                title="CPU Usage"
                data={{
                  labels: ["00:00", "04:00", "08:00", "12:00", "16:00", "20:00"],
                  datasets: [
                    {
                      label: "CPU Cores",
                      data: [0.5, 0.8, 1.2, 1.5, 1.1, 0.9],
                      borderColor: "hsl(var(--primary))",
                    },
                  ],
                }}
              />
              <MetricsChart
                title="Memory Usage"
                data={{
                  labels: ["00:00", "04:00", "08:00", "12:00", "16:00", "20:00"],
                  datasets: [
                    {
                      label: "Memory (GB)",
                      data: [2.1, 2.3, 2.8, 3.1, 2.9, 2.5],
                      borderColor: "hsl(var(--primary))",
                    },
                  ],
                }}
              />
              <MetricsChart
                title="Network I/O"
                data={{
                  labels: ["00:00", "04:00", "08:00", "12:00", "16:00", "20:00"],
                  datasets: [
                    {
                      label: "Received (MB)",
                      data: [100, 150, 200, 180, 220, 190],
                      borderColor: "hsl(var(--primary))",
                    },
                    {
                      label: "Sent (MB)",
                      data: [50, 75, 100, 90, 110, 95],
                      borderColor: "hsl(var(--secondary))",
                    },
                  ],
                }}
                type="bar"
              />
              <MetricsChart
                title="Pod Status"
                data={{
                  labels: ["Running", "Pending", "Failed", "Unknown"],
                  datasets: [
                    {
                      label: "Count",
                      data: [45, 3, 1, 0],
                      backgroundColor: "hsl(var(--success))",
                    },
                  ],
                }}
                type="bar"
              />
            </div>
          </TabsContent>

          <TabsContent value="workloads" className="h-full overflow-y-auto">
            <div className="text-center py-12 text-muted-foreground">
              <p>Workloads will be displayed here</p>
            </div>
          </TabsContent>

          <TabsContent value="infrastructure" className="h-full overflow-y-auto">
            <div className="text-center py-12 text-muted-foreground">
              <p>Infrastructure resources will be displayed here</p>
            </div>
          </TabsContent>

          <TabsContent value="terminal" className="h-full">
            <Terminal clusterId={clusterId} namespace={namespace} />
          </TabsContent>

          <TabsContent value="yaml" className="h-full">
            <YamlEditor onChange={() => {}} />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
