import { describe, it, expect, beforeEach } from "vitest";
import { openPodLogsTab, openWorkloadLogsTab } from "@/lib/logsDock";
import { useBottomPanelStore, BottomPanelTabType } from "@/stores/bottomPanelStore";

describe("logsDock helpers", () => {
  beforeEach(() => {
    useBottomPanelStore.setState({ tabs: [], activeTabId: null, isOpen: false });
  });

  it("openPodLogsTab opens a POD_LOGS tab in single-pod mode", () => {
    openPodLogsTab({
      clusterId: "c1",
      namespace: "default",
      podName: "web-abc",
      containers: ["app", "sidecar"],
    });

    const tabs = useBottomPanelStore.getState().tabs;
    expect(tabs).toHaveLength(1);
    const tab = tabs[0];
    expect(tab.type).toBe(BottomPanelTabType.POD_LOGS);
    expect(tab.title).toBe("Logs: web-abc");
    expect(tab.data?.podName).toBe("web-abc");
    expect(tab.data?.containers).toEqual(["app", "sidecar"]);
    expect(tab.data?.workloadName).toBeUndefined();
    expect(useBottomPanelStore.getState().isOpen).toBe(true);
  });

  it("openPodLogsTab de-duplicates on cluster/namespace/pod", () => {
    const params = {
      clusterId: "c1",
      namespace: "default",
      podName: "web-abc",
      containers: ["app"],
    };
    openPodLogsTab(params);
    openPodLogsTab(params);
    expect(useBottomPanelStore.getState().tabs).toHaveLength(1);
  });

  it("openWorkloadLogsTab opens a POD_LOGS tab in workload mode", () => {
    openWorkloadLogsTab({
      clusterId: "c1",
      namespace: "default",
      workloadName: "web",
      workloadType: "deployment",
    });

    const tab = useBottomPanelStore.getState().tabs[0];
    expect(tab.type).toBe(BottomPanelTabType.POD_LOGS);
    expect(tab.title).toBe("Logs: web");
    expect(tab.data?.workloadName).toBe("web");
    expect(tab.data?.workloadType).toBe("deployment");
    expect(tab.data?.podName).toBeUndefined();
  });

  it("openWorkloadLogsTab de-duplicates per workload type+name", () => {
    const params = {
      clusterId: "c1",
      namespace: "default",
      workloadName: "web",
      workloadType: "deployment",
    };
    openWorkloadLogsTab(params);
    openWorkloadLogsTab(params);
    expect(useBottomPanelStore.getState().tabs).toHaveLength(1);
  });
});
