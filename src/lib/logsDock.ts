import { useBottomPanelStore, BottomPanelTabType } from "@/stores/bottomPanelStore";
import type { LogsTabData } from "@/components/dock/LogsTab";

/**
 * Open a streaming log viewer for a single pod in the bottom dock
 * (freelens-style), de-duplicating on the pod name.
 */
export function openPodLogsTab(params: {
  clusterId: string;
  namespace: string;
  podName: string;
  containers: string[];
}): void {
  const data: LogsTabData = {
    clusterId: params.clusterId,
    namespace: params.namespace,
    podName: params.podName,
    containers: params.containers,
  };
  useBottomPanelStore.getState().openTab({
    type: BottomPanelTabType.POD_LOGS,
    title: `Logs: ${params.podName}`,
    key: `${params.clusterId}/${params.namespace}/${params.podName}`,
    data,
  });
}

/**
 * Open a streaming log viewer for a workload (Deployment, StatefulSet, …) in the
 * bottom dock. The tab resolves the workload's pods and exposes a pod picker so
 * live logs can be followed/tailed per pod.
 */
export function openWorkloadLogsTab(params: {
  clusterId: string;
  namespace: string;
  workloadName: string;
  workloadType: string;
}): void {
  const data: LogsTabData = {
    clusterId: params.clusterId,
    namespace: params.namespace,
    workloadName: params.workloadName,
    workloadType: params.workloadType,
  };
  useBottomPanelStore.getState().openTab({
    type: BottomPanelTabType.POD_LOGS,
    title: `Logs: ${params.workloadName}`,
    key: `${params.clusterId}/${params.namespace}/${params.workloadType}/${params.workloadName}`,
    data,
  });
}
