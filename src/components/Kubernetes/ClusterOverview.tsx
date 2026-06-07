import React from "react";
import { Server, Database, Globe } from "lucide-react";
import { MetricsChart } from "./MetricsChart";

interface ClusterOverviewProps {
  clusterId: string;
}

export function ClusterOverview({ clusterId }: ClusterOverviewProps) {
  return (
    <div className="h-full overflow-y-auto">
      <div className="mb-6">
        <h2 className="text-2xl font-semibold">Cluster Overview</h2>
        <p className="text-muted-foreground">Cluster ID: {clusterId}</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div className="bg-card rounded-lg p-4 border">
          <div className="flex items-center justify-between pb-2">
            <h3 className="text-sm font-medium">Nodes</h3>
            <Server className="h-4 w-4 text-muted-foreground" />
          </div>
          <div className="text-2xl font-bold">15</div>
          <p className="text-xs text-muted-foreground">+2 since last week</p>
        </div>

        <div className="bg-card rounded-lg p-4 border">
          <div className="flex items-center justify-between pb-2">
            <h3 className="text-sm font-medium">Pods</h3>
            <Database className="h-4 w-4 text-muted-foreground" />
          </div>
          <div className="text-2xl font-bold">247</div>
          <p className="text-xs text-muted-foreground">+15 since last week</p>
        </div>

        <div className="bg-card rounded-lg p-4 border">
          <div className="flex items-center justify-between pb-2">
            <h3 className="text-sm font-medium">Workloads</h3>
            <Globe className="h-4 w-4 text-muted-foreground" />
          </div>
          <div className="text-2xl font-bold">32</div>
          <p className="text-xs text-muted-foreground">+4 since last week</p>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <MetricsChart
          title="Cluster CPU Usage"
          data={{
            labels: ["00:00", "04:00", "08:00", "12:00", "16:00", "20:00"],
            datasets: [
              {
                label: "CPU Cores",
                data: [12.5, 14.8, 18.2, 22.5, 19.1, 15.9],
                borderColor: "hsl(var(--primary))",
              },
            ],
          }}
        />
        <MetricsChart
          title="Cluster Memory Usage"
          data={{
            labels: ["00:00", "04:00", "08:00", "12:00", "16:00", "20:00"],
            datasets: [
              {
                label: "Memory (GB)",
                data: [45.1, 48.3, 52.8, 58.1, 55.9, 50.5],
                borderColor: "hsl(var(--primary))",
              },
            ],
          }}
        />
      </div>

      <div className="bg-card rounded-lg border">
        <div className="border-b px-6 py-4">
          <h3 className="font-semibold">Cluster Resources</h3>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <h4 className="font-medium">Allocatable Resources</h4>
              <div className="space-y-2 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">CPU (cores)</span>
                  <span className="font-mono">32</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">Memory (GB)</span>
                  <span className="font-mono">128</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">Pods</span>
                  <span className="font-mono">110</span>
                </div>
              </div>
            </div>
            <div className="space-y-2">
              <h4 className="font-medium">Used Resources</h4>
              <div className="space-y-2 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">CPU (cores)</span>
                  <span className="font-mono">18.5 (58%)</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">Memory (GB)</span>
                  <span className="font-mono">52.3 (41%)</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">Pods</span>
                  <span className="font-mono">247 (22%)</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="bg-card rounded-lg border mt-6">
        <div className="border-b px-6 py-4">
          <h3 className="font-semibold">Recent Events</h3>
        </div>
        <div className="p-6">
          <div className="space-y-4">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">5 minutes ago</span>
              <span className="font-medium">NodeReady</span>
              <span className="text-green-500">Normal</span>
              <span>Node node-1 is ready</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">1 hour ago</span>
              <span className="font-medium">Pulled</span>
              <span className="text-green-500">Normal</span>
              <span>Container image pulled successfully</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">2 hours ago</span>
              <span className="font-medium">ScalingReplicaSet</span>
              <span className="text-green-500">Normal</span>
              <span>Scaled up deployment web-app</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
