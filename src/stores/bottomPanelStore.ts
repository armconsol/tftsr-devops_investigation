import { create } from "zustand";
import { persist } from "zustand/middleware";

// ─── Types ────────────────────────────────────────────────────────────────────

export enum BottomPanelTabType {
  POD_LOGS = "POD_LOGS",
  TERMINAL = "TERMINAL",
  EDIT_RESOURCE = "EDIT_RESOURCE",
  CREATE_RESOURCE = "CREATE_RESOURCE",
  INSTALL_CHART = "INSTALL_CHART",
  UPGRADE_CHART = "UPGRADE_CHART",
}

/**
 * Per-tab data payload. The shape is intentionally loose because each tab type
 * needs a different set of fields. Consumers should narrow `data` based on
 * `type` when rendering.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type BottomPanelTabData = Record<string, any>;

export interface BottomPanelTab {
  id: string;
  type: BottomPanelTabType;
  title: string;
  /** Optional dedup key — re-opening a tab with the same type+key focuses the existing tab. */
  key?: string;
  data?: BottomPanelTabData;
}

export interface OpenTabOptions {
  type: BottomPanelTabType;
  title: string;
  key?: string;
  data?: BottomPanelTabData;
}

// ─── Constants ────────────────────────────────────────────────────────────────

export const DEFAULT_PANEL_HEIGHT = 320;
export const MIN_PANEL_HEIGHT = 120;
export const MAX_PANEL_HEIGHT = 900;

// ─── Store ────────────────────────────────────────────────────────────────────

interface BottomPanelState {
  isOpen: boolean;
  height: number;
  tabs: BottomPanelTab[];
  activeTabId: string | null;
  /** Monotonically increasing counter used to build unique tab ids. */
  nextTabIndex: number;

  openPanel: () => void;
  closePanel: () => void;
  togglePanel: () => void;

  setHeight: (height: number) => void;

  openTab: (options: OpenTabOptions) => string;
  closeTab: (id: string) => void;
  closeActiveTab: () => void;
  setActiveTab: (id: string) => void;
  nextTab: () => void;
  previousTab: () => void;
}

function clampHeight(h: number): number {
  if (Number.isNaN(h)) return DEFAULT_PANEL_HEIGHT;
  if (h < MIN_PANEL_HEIGHT) return MIN_PANEL_HEIGHT;
  if (h > MAX_PANEL_HEIGHT) return MAX_PANEL_HEIGHT;
  return Math.round(h);
}

export const useBottomPanelStore = create<BottomPanelState>()(
  persist(
    (set, get) => ({
      isOpen: false,
      height: DEFAULT_PANEL_HEIGHT,
      tabs: [],
      activeTabId: null,
      nextTabIndex: 1,

      openPanel: () => set({ isOpen: true }),
      closePanel: () => set({ isOpen: false }),
      togglePanel: () => set((s) => ({ isOpen: !s.isOpen })),

      setHeight: (height) => set({ height: clampHeight(height) }),

      openTab: ({ type, title, key, data }) => {
        // De-dup on (type, key) when key is provided
        if (key) {
          const existing = get().tabs.find((t) => t.type === type && t.key === key);
          if (existing) {
            set({ activeTabId: existing.id, isOpen: true });
            return existing.id;
          }
        }

        const idx = get().nextTabIndex;
        const id = `tab-${idx}-${type.toLowerCase()}`;
        const tab: BottomPanelTab = { id, type, title, key, data };
        set((s) => ({
          tabs: [...s.tabs, tab],
          activeTabId: id,
          isOpen: true,
          nextTabIndex: s.nextTabIndex + 1,
        }));
        return id;
      },

      closeTab: (id) =>
        set((s) => {
          const idx = s.tabs.findIndex((t) => t.id === id);
          if (idx === -1) return s;
          const nextTabs = s.tabs.filter((t) => t.id !== id);

          let nextActive: string | null = s.activeTabId;
          if (s.activeTabId === id) {
            // Prefer the tab that was just before the closed one; otherwise the new last tab.
            const fallback = nextTabs[idx - 1] ?? nextTabs[nextTabs.length - 1] ?? null;
            nextActive = fallback?.id ?? null;
          }

          return {
            tabs: nextTabs,
            activeTabId: nextActive,
            isOpen: nextTabs.length > 0 ? s.isOpen : false,
          };
        }),

      closeActiveTab: () => {
        const id = get().activeTabId;
        if (id) get().closeTab(id);
      },

      setActiveTab: (id) => set({ activeTabId: id, isOpen: true }),

      nextTab: () =>
        set((s) => {
          if (s.tabs.length === 0) return s;
          const idx = s.tabs.findIndex((t) => t.id === s.activeTabId);
          const nextIdx = (idx + 1) % s.tabs.length;
          return { activeTabId: s.tabs[nextIdx]!.id };
        }),

      previousTab: () =>
        set((s) => {
          if (s.tabs.length === 0) return s;
          const idx = s.tabs.findIndex((t) => t.id === s.activeTabId);
          const prevIdx = (idx - 1 + s.tabs.length) % s.tabs.length;
          return { activeTabId: s.tabs[prevIdx]!.id };
        }),
    }),
    {
      name: "tftsr-bottom-panel",
      // Only persist height — tabs and open state are session-only
      partialize: (s) => ({ height: s.height }),
    }
  )
);
