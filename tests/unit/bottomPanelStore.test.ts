import { describe, it, expect, beforeEach } from "vitest";
import {
  useBottomPanelStore,
  BottomPanelTabType,
  DEFAULT_PANEL_HEIGHT,
  MIN_PANEL_HEIGHT,
  MAX_PANEL_HEIGHT,
} from "@/stores/bottomPanelStore";

describe("bottomPanelStore", () => {
  beforeEach(() => {
    localStorage.clear();
    // Reset store to initial state
    useBottomPanelStore.setState({
      isOpen: false,
      height: DEFAULT_PANEL_HEIGHT,
      tabs: [],
      activeTabId: null,
      nextTabIndex: 1,
    });
  });

  describe("initial state", () => {
    it("is closed by default", () => {
      expect(useBottomPanelStore.getState().isOpen).toBe(false);
    });

    it("has default height", () => {
      expect(useBottomPanelStore.getState().height).toBe(DEFAULT_PANEL_HEIGHT);
    });

    it("has no tabs", () => {
      expect(useBottomPanelStore.getState().tabs).toEqual([]);
      expect(useBottomPanelStore.getState().activeTabId).toBeNull();
    });
  });

  describe("openPanel / closePanel / togglePanel", () => {
    it("openPanel sets isOpen to true", () => {
      useBottomPanelStore.getState().openPanel();
      expect(useBottomPanelStore.getState().isOpen).toBe(true);
    });

    it("closePanel sets isOpen to false", () => {
      useBottomPanelStore.getState().openPanel();
      useBottomPanelStore.getState().closePanel();
      expect(useBottomPanelStore.getState().isOpen).toBe(false);
    });

    it("togglePanel flips isOpen", () => {
      const { togglePanel } = useBottomPanelStore.getState();
      togglePanel();
      expect(useBottomPanelStore.getState().isOpen).toBe(true);
      togglePanel();
      expect(useBottomPanelStore.getState().isOpen).toBe(false);
    });
  });

  describe("setHeight", () => {
    it("clamps to minimum", () => {
      useBottomPanelStore.getState().setHeight(MIN_PANEL_HEIGHT - 50);
      expect(useBottomPanelStore.getState().height).toBe(MIN_PANEL_HEIGHT);
    });

    it("clamps to maximum", () => {
      useBottomPanelStore.getState().setHeight(MAX_PANEL_HEIGHT + 1000);
      expect(useBottomPanelStore.getState().height).toBe(MAX_PANEL_HEIGHT);
    });

    it("accepts a value within range", () => {
      useBottomPanelStore.getState().setHeight(450);
      expect(useBottomPanelStore.getState().height).toBe(450);
    });
  });

  describe("openTab", () => {
    it("creates a new tab and opens the panel", () => {
      const id = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.POD_LOGS,
        title: "Pod Logs: nginx",
        data: { clusterId: "c1", namespace: "default", podName: "nginx", containers: ["nginx"] },
      });

      const state = useBottomPanelStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0]!.id).toBe(id);
      expect(state.tabs[0]!.type).toBe(BottomPanelTabType.POD_LOGS);
      expect(state.activeTabId).toBe(id);
      expect(state.isOpen).toBe(true);
    });

    it("returns existing tab id when same type+key already open", () => {
      const a = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.POD_LOGS,
        title: "nginx",
        key: "default/nginx",
      });
      const b = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.POD_LOGS,
        title: "nginx",
        key: "default/nginx",
      });
      expect(a).toBe(b);
      expect(useBottomPanelStore.getState().tabs).toHaveLength(1);
    });

    it("supports all 6 tab types", () => {
      const types = [
        BottomPanelTabType.POD_LOGS,
        BottomPanelTabType.TERMINAL,
        BottomPanelTabType.EDIT_RESOURCE,
        BottomPanelTabType.CREATE_RESOURCE,
        BottomPanelTabType.INSTALL_CHART,
        BottomPanelTabType.UPGRADE_CHART,
      ];
      for (const t of types) {
        useBottomPanelStore.getState().openTab({ type: t, title: t });
      }
      expect(useBottomPanelStore.getState().tabs).toHaveLength(6);
    });
  });

  describe("closeTab", () => {
    it("removes the tab and updates active tab", () => {
      const a = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.TERMINAL,
        title: "term1",
      });
      const b = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.TERMINAL,
        title: "term2",
      });
      useBottomPanelStore.getState().closeTab(b);
      expect(useBottomPanelStore.getState().tabs).toHaveLength(1);
      expect(useBottomPanelStore.getState().activeTabId).toBe(a);
    });

    it("closes the panel when last tab is removed", () => {
      const id = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.TERMINAL,
        title: "term1",
      });
      useBottomPanelStore.getState().closeTab(id);
      expect(useBottomPanelStore.getState().isOpen).toBe(false);
      expect(useBottomPanelStore.getState().activeTabId).toBeNull();
    });

    it("focuses previous tab when active tab is closed", () => {
      const a = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.TERMINAL,
        title: "a",
      });
      const b = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.TERMINAL,
        title: "b",
      });
      const c = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.TERMINAL,
        title: "c",
      });
      // active is c — close c, should fall back to b
      useBottomPanelStore.getState().closeTab(c);
      expect(useBottomPanelStore.getState().activeTabId).toBe(b);
      // close a (not active) — active should remain b
      useBottomPanelStore.getState().closeTab(a);
      expect(useBottomPanelStore.getState().activeTabId).toBe(b);
    });
  });

  describe("setActiveTab", () => {
    it("updates the active tab id", () => {
      const a = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.TERMINAL,
        title: "a",
      });
      const b = useBottomPanelStore.getState().openTab({
        type: BottomPanelTabType.TERMINAL,
        title: "b",
      });
      useBottomPanelStore.getState().setActiveTab(a);
      expect(useBottomPanelStore.getState().activeTabId).toBe(a);
      useBottomPanelStore.getState().setActiveTab(b);
      expect(useBottomPanelStore.getState().activeTabId).toBe(b);
    });
  });

  describe("nextTab / previousTab", () => {
    it("cycles forward through tabs", () => {
      const a = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "a" });
      const b = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "b" });
      const c = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "c" });
      useBottomPanelStore.getState().setActiveTab(a);

      useBottomPanelStore.getState().nextTab();
      expect(useBottomPanelStore.getState().activeTabId).toBe(b);
      useBottomPanelStore.getState().nextTab();
      expect(useBottomPanelStore.getState().activeTabId).toBe(c);
      // wraps
      useBottomPanelStore.getState().nextTab();
      expect(useBottomPanelStore.getState().activeTabId).toBe(a);
    });

    it("cycles backward through tabs", () => {
      const a = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "a" });
      const b = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "b" });
      const c = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "c" });
      useBottomPanelStore.getState().setActiveTab(a);

      useBottomPanelStore.getState().previousTab();
      expect(useBottomPanelStore.getState().activeTabId).toBe(c);
      useBottomPanelStore.getState().previousTab();
      expect(useBottomPanelStore.getState().activeTabId).toBe(b);
    });
  });

  describe("closeActiveTab", () => {
    it("removes whatever tab is currently active", () => {
      const a = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "a" });
      useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "b" });
      useBottomPanelStore.getState().setActiveTab(a);
      useBottomPanelStore.getState().closeActiveTab();
      const remaining = useBottomPanelStore.getState().tabs.map((t) => t.id);
      expect(remaining).not.toContain(a);
    });
  });

  describe("persistence", () => {
    it("persists height to localStorage", () => {
      useBottomPanelStore.getState().setHeight(420);
      const raw = localStorage.getItem("tftsr-bottom-panel");
      expect(raw).not.toBeNull();
      expect(raw!).toContain("420");
    });
  });
});
