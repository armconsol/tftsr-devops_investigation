import { describe, it, expect, vi, beforeEach } from "vitest";
import { render } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { ProxmoxShellPage } from "@/pages/Proxmox/ShellPage";

const nodeShellConsoleMock = vi.fn((_props: unknown) => null);

vi.mock("@/components/Proxmox/NodeShellConsole", () => ({
  NodeShellConsole: (props: unknown) => nodeShellConsoleMock(props),
}));

beforeEach(() => {
  nodeShellConsoleMock.mockClear();
});

function renderAt(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <Routes>
        <Route path="/proxmox/shell/:clusterId/:node" element={<ProxmoxShellPage />} />
      </Routes>
    </MemoryRouter>
  );
}

describe("ProxmoxShellPage — cmd query param", () => {
  it("passes no cmd when the query param is absent", () => {
    renderAt("/proxmox/shell/cluster-1/vmhost1");
    expect(nodeShellConsoleMock).toHaveBeenCalledWith(
      expect.objectContaining({ clusterId: "cluster-1", node: "vmhost1", cmd: undefined })
    );
  });

  it("forwards ?cmd=upgrade to NodeShellConsole", () => {
    renderAt("/proxmox/shell/cluster-1/vmhost1?cmd=upgrade");
    expect(nodeShellConsoleMock).toHaveBeenCalledWith(
      expect.objectContaining({ clusterId: "cluster-1", node: "vmhost1", cmd: "upgrade" })
    );
  });

  it("ignores an unrecognized cmd value", () => {
    renderAt("/proxmox/shell/cluster-1/vmhost1?cmd=rm-rf");
    expect(nodeShellConsoleMock).toHaveBeenCalledWith(
      expect.objectContaining({ clusterId: "cluster-1", node: "vmhost1", cmd: undefined })
    );
  });
});
