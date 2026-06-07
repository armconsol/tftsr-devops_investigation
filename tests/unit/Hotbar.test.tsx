import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Hotbar } from "@/components/Kubernetes/Hotbar";

// Mock zustand's useStore so Hotbar can render without a real store
vi.mock("zustand", () => ({
  useStore: vi.fn((_store: unknown, selector: (s: unknown) => unknown) =>
    selector({ clusters: [], selectedClusterId: null })
  ),
}));

vi.mock("@/stores/kubernetesStore", () => ({
  useKubernetesStore: vi.fn(),
}));

describe("Hotbar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without error", () => {
    render(
      <Hotbar
        onRefresh={vi.fn()}
        onAddResource={vi.fn()}
        onSettings={vi.fn()}
      />
    );
    expect(screen.getByRole("button", { name: /notification/i })).toBeInTheDocument();
  });

  it("calls onNotifications when bell icon is clicked", () => {
    const onNotifications = vi.fn();

    render(
      <Hotbar
        onRefresh={vi.fn()}
        onAddResource={vi.fn()}
        onSettings={vi.fn()}
        onNotifications={onNotifications}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /notification/i }));
    expect(onNotifications).toHaveBeenCalledTimes(1);
  });

  it("renders bell button even without onNotifications prop", () => {
    render(
      <Hotbar
        onRefresh={vi.fn()}
        onAddResource={vi.fn()}
        onSettings={vi.fn()}
      />
    );

    const bellButton = screen.getByRole("button", { name: /notification/i });
    expect(bellButton).toBeInTheDocument();
    expect(() => fireEvent.click(bellButton)).not.toThrow();
  });

  it("shows notification count badge when notificationCount is provided", () => {
    render(
      <Hotbar
        onRefresh={vi.fn()}
        onAddResource={vi.fn()}
        onSettings={vi.fn()}
        notificationCount={5}
      />
    );

    expect(screen.getByText("5")).toBeInTheDocument();
  });

  it("hides badge when notificationCount is zero", () => {
    render(
      <Hotbar
        onRefresh={vi.fn()}
        onAddResource={vi.fn()}
        onSettings={vi.fn()}
        notificationCount={0}
      />
    );

    expect(screen.queryByText("0")).not.toBeInTheDocument();
  });
});
