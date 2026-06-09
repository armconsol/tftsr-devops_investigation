import React from "react";
import { Layers, Box, Server, Activity } from "lucide-react";
import type {
  PodInfo,
  DeploymentInfo,
  StatefulSetInfo,
  DaemonSetInfo,
  JobInfo,
  CronJobInfo,
} from "@/lib/tauriCommands";

interface WorkloadOverviewProps {
  clusterId: string;
  resources: {
    pods: PodInfo[];
    deployments: DeploymentInfo[];
    statefulsets: StatefulSetInfo[];
    daemonsets: DaemonSetInfo[];
    jobs: JobInfo[];
    cronjobs: CronJobInfo[];
  };
}

interface SummaryCardProps {
  title: string;
  value: number;
  subtitle?: string;
  icon: React.ReactNode;
}

function SummaryCard({ title, value, subtitle, icon }: SummaryCardProps) {
  return (
    <div className="bg-card rounded-lg p-4 border">
      <div className="flex items-center justify-between pb-2">
        <h3 className="text-sm font-medium">{title}</h3>
        {icon}
      </div>
      <div className="text-2xl font-bold">{value}</div>
      {subtitle && (
        <p className="text-xs text-muted-foreground mt-1">{subtitle}</p>
      )}
    </div>
  );
}

export function WorkloadOverview({ resources }: WorkloadOverviewProps) {
  const { pods, deployments, statefulsets, daemonsets, jobs, cronjobs } = resources;

  const runningPods = pods.filter((p) => p.status === "Running").length;
  const pendingPods = pods.filter((p) => p.status === "Pending").length;
  const failedPods = pods.filter((p) => p.status === "Failed").length;

  const readyDeployments = deployments.filter((d) => d.ready === `${d.replicas}/${d.replicas}`).length;

  const readyStatefulSets = statefulsets.filter((s) => {
    const parts = s.ready.split("/");
    return parts.length === 2 && parts[0] === parts[1];
  }).length;

  const healthyDaemonSets = daemonsets.filter(
    (ds) => ds.desired === ds.ready
  ).length;

  const completedJobs = jobs.filter((j) => {
    const parts = j.completions.split("/");
    return parts.length === 2 && parts[0] === parts[1];
  }).length;

  return (
    <div className="h-full overflow-y-auto space-y-6 p-6">
      <div>
        <h2 className="text-2xl font-semibold">Workload Overview</h2>
        <p className="text-muted-foreground text-sm mt-0.5">
          Summary of all workload resources in the selected namespace
        </p>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
        <SummaryCard
          title="Pods"
          value={pods.length}
          subtitle={`Running: ${runningPods} · Pending: ${pendingPods} · Failed: ${failedPods}`}
          icon={<Box className="h-4 w-4 text-muted-foreground" />}
        />
        <SummaryCard
          title="Deployments"
          value={deployments.length}
          subtitle={`Ready: ${readyDeployments}/${deployments.length}`}
          icon={<Layers className="h-4 w-4 text-muted-foreground" />}
        />
        <SummaryCard
          title="StatefulSets"
          value={statefulsets.length}
          subtitle={`Ready: ${readyStatefulSets}/${statefulsets.length}`}
          icon={<Server className="h-4 w-4 text-muted-foreground" />}
        />
        <SummaryCard
          title="DaemonSets"
          value={daemonsets.length}
          subtitle={`Healthy: ${healthyDaemonSets}/${daemonsets.length}`}
          icon={<Activity className="h-4 w-4 text-muted-foreground" />}
        />
        <SummaryCard
          title="Jobs"
          value={jobs.length}
          subtitle={`Completed: ${completedJobs}/${jobs.length}`}
          icon={<Activity className="h-4 w-4 text-muted-foreground" />}
        />
        <SummaryCard
          title="Cron Jobs"
          value={cronjobs.length}
          subtitle={cronjobs.length > 0 ? `Active: ${cronjobs.reduce((acc, cj) => acc + cj.active, 0)}` : undefined}
          icon={<Activity className="h-4 w-4 text-muted-foreground" />}
        />
      </div>

      {pods.length > 0 && (
        <div className="bg-card rounded-lg border">
          <div className="border-b px-6 py-4">
            <h3 className="font-semibold">Pod Status Breakdown</h3>
          </div>
          <div className="p-6">
            <div className="flex gap-6 text-sm">
              <div className="flex items-center gap-2">
                <span className="inline-block w-3 h-3 rounded-full bg-green-500" />
                <span>Running: {runningPods}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="inline-block w-3 h-3 rounded-full bg-yellow-500" />
                <span>Pending: {pendingPods}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="inline-block w-3 h-3 rounded-full bg-red-500" />
                <span>Failed: {failedPods}</span>
              </div>
              {pods.length - runningPods - pendingPods - failedPods > 0 && (
                <div className="flex items-center gap-2">
                  <span className="inline-block w-3 h-3 rounded-full bg-gray-400" />
                  <span>Other: {pods.length - runningPods - pendingPods - failedPods}</span>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
