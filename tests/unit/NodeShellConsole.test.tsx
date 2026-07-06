import { describe, it, expect, vi, beforeEach } from "vitest";
import { render } from "@testing-library/react";
import { NodeShellConsole } from "@/components/Proxmox/NodeShellConsole";

const openNodeShellMock = vi.fn((..._args: unknown[]) => new Promise(() => {}));

vi.mock("@/lib/proxmoxClient", () => ({
  openNodeShell: (a: string, b: string, c?: string) => openNodeShellMock(a, b, c),
}));

beforeEach(() => {
  openNodeShellMock.mockClear();
});

describe("NodeShellConsole — cmd forwarding", () => {
  it("opens a plain login shell when no cmd is given", () => {
    render(<NodeShellConsole clusterId="cluster-1" node="vmhost1" />);
    expect(openNodeShellMock).toHaveBeenCalledWith("cluster-1", "vmhost1", undefined);
  });

  it("forwards an explicit cmd (e.g. upgrade) to openNodeShell", () => {
    render(<NodeShellConsole clusterId="cluster-1" node="vmhost1" cmd="upgrade" />);
    expect(openNodeShellMock).toHaveBeenCalledWith("cluster-1", "vmhost1", "upgrade");
  });
});
