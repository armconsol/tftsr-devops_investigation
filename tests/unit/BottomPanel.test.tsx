import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { BottomPanel } from "@/components/BottomPanel";
import {
  useBottomPanelStore,
  BottomPanelTabType,
  DEFAULT_PANEL_HEIGHT,
} from "@/stores/bottomPanelStore";

// Stub the heavier tab content to keep this test focused on the panel chrome.
vi.mock("@/components/dock/LogsTab", () => ({
  LogsTab: () => <div data-testid="logs-tab-stub">logs</div>,
}));
vi.mock("@/components/dock/TerminalTab", () => ({
  TerminalTab: () => <div data-testid="terminal-tab-stub">terminal</div>,
}));
vi.mock("@/components/dock/YamlEditorTab", () => ({
  YamlEditorTab: () => <div data-testid="yaml-tab-stub">yaml</div>,
}));

function resetStore() {
  useBottomPanelStore.setState({
    isOpen: false,
    height: DEFAULT_PANEL_HEIGHT,
    tabs: [],
    activeTabId: null,
    nextTabIndex: 1,
  });
}

describe("BottomPanel", () => {
  beforeEach(() => {
    resetStore();
  });

  it("renders nothing when closed", () => {
    render(<BottomPanel />);
    expect(screen.queryByTestId("bottom-panel")).toBeNull();
  });

  it("renders panel + drag handle when open with a tab", () => {
    useBottomPanelStore.getState().openTab({
      type: BottomPanelTabType.TERMINAL,
      title: "terminal-1",
    });
    render(<BottomPanel />);
    expect(screen.getByTestId("bottom-panel")).toBeInTheDocument();
    expect(screen.getByTestId("bottom-panel-drag-handle")).toBeInTheDocument();
  });

  it("uses height from the store", () => {
    useBottomPanelStore.getState().openTab({
      type: BottomPanelTabType.TERMINAL,
      title: "t",
    });
    useBottomPanelStore.getState().setHeight(420);
    render(<BottomPanel />);
    const panel = screen.getByTestId("bottom-panel");
    expect(panel).toHaveStyle({ height: "420px" });
  });

  it("close button removes the active tab", () => {
    const id = useBottomPanelStore.getState().openTab({
      type: BottomPanelTabType.TERMINAL,
      title: "term",
    });
    render(<BottomPanel />);

    const closeBtn = screen.getByLabelText(`Close tab term`);
    fireEvent.click(closeBtn);

    expect(useBottomPanelStore.getState().tabs.find((t) => t.id === id)).toBeUndefined();
  });

  it("clicking inactive tab makes it active", () => {
    const a = useBottomPanelStore.getState().openTab({
      type: BottomPanelTabType.TERMINAL,
      title: "alpha",
    });
    useBottomPanelStore.getState().openTab({
      type: BottomPanelTabType.TERMINAL,
      title: "beta",
    });
    render(<BottomPanel />);

    fireEvent.click(screen.getByText("alpha"));
    expect(useBottomPanelStore.getState().activeTabId).toBe(a);
  });

  it("collapse-all button closes the panel", () => {
    useBottomPanelStore.getState().openTab({
      type: BottomPanelTabType.TERMINAL,
      title: "term",
    });
    render(<BottomPanel />);

    fireEvent.click(screen.getByLabelText("Hide bottom panel"));
    expect(useBottomPanelStore.getState().isOpen).toBe(false);
  });
});

describe("BottomPanel keyboard shortcuts", () => {
  beforeEach(() => {
    resetStore();
  });

  it("Ctrl+W closes the active tab", () => {
    const id = useBottomPanelStore.getState().openTab({
      type: BottomPanelTabType.TERMINAL,
      title: "term",
    });
    render(<BottomPanel />);

    fireEvent.keyDown(window, { key: "w", ctrlKey: true });
    expect(useBottomPanelStore.getState().tabs.find((t) => t.id === id)).toBeUndefined();
  });

  it("Shift+Escape hides the panel", () => {
    useBottomPanelStore.getState().openTab({
      type: BottomPanelTabType.TERMINAL,
      title: "term",
    });
    render(<BottomPanel />);

    fireEvent.keyDown(window, { key: "Escape", shiftKey: true });
    expect(useBottomPanelStore.getState().isOpen).toBe(false);
  });

  it("Ctrl+. switches to next tab", () => {
    const a = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "a" });
    const b = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "b" });
    useBottomPanelStore.getState().setActiveTab(a);

    render(<BottomPanel />);
    fireEvent.keyDown(window, { key: ".", ctrlKey: true });
    expect(useBottomPanelStore.getState().activeTabId).toBe(b);
  });

  it("Ctrl+, switches to previous tab", () => {
    useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "a" });
    const b = useBottomPanelStore.getState().openTab({ type: BottomPanelTabType.TERMINAL, title: "b" });
    useBottomPanelStore.getState().setActiveTab(b);

    render(<BottomPanel />);
    fireEvent.keyDown(window, { key: ",", ctrlKey: true });
    expect(useBottomPanelStore.getState().activeTabId).not.toBe(b);
  });

  it("ignores shortcuts when panel is closed", () => {
    render(<BottomPanel />);
    // Should not throw
    fireEvent.keyDown(window, { key: "w", ctrlKey: true });
    fireEvent.keyDown(window, { key: ".", ctrlKey: true });
    expect(useBottomPanelStore.getState().tabs).toHaveLength(0);
  });
});
