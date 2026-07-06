import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { ProxmoxTasksPage } from "@/pages/Proxmox/TasksPage";

vi.mock("@tauri-apps/api/core");

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

const cluster = { id: "cluster-1", name: "TFTSR" };
const tasks = [
  {
    upid: "UPID:vmhost1:00001111:aaa:aptupdate::root@pam:",
    node: "vmhost1",
    pid: 111,
    starttime: 1700000000,
    type: "aptupdate",
    user: "root@pam",
    exitstatus: "OK",
  },
  {
    upid: "UPID:vmhost2:00002222:bbb:vzdump::root@pam:",
    node: "vmhost2",
    pid: 222,
    starttime: 1700000100,
    type: "vzdump",
    user: "root@pam",
    exitstatus: "OK",
  },
];

function setupInvoke() {
  mockInvoke.mockImplementation((cmd: string) => {
    switch (cmd) {
      case "list_proxmox_clusters":
        return Promise.resolve([cluster]);
      case "list_cluster_tasks":
        return Promise.resolve(tasks);
      case "search_task_logs":
        return Promise.resolve([
          {
            node: "vmhost1",
            upid: tasks[0].upid,
            matches: [{ n: 3, t: "Failed to fetch http://deb.debian.org" }],
          },
          { node: "vmhost2", upid: tasks[1].upid, matches: [] },
        ]);
      case "get_proxmox_task_log":
        return Promise.resolve([
          { n: 1, t: "starting" },
          { n: 2, t: "downloading" },
          { n: 3, t: "Failed to fetch http://deb.debian.org" },
        ]);
      default:
        return Promise.resolve(undefined);
    }
  });
}

beforeEach(() => {
  mockInvoke.mockReset();
  setupInvoke();
});

describe("TasksPage — log search", () => {
  it("renders a search field for task logs", async () => {
    render(<ProxmoxTasksPage />);
    await waitFor(() => expect(screen.getByText("aptupdate")).toBeInTheDocument());
    expect(screen.getByPlaceholderText(/search task logs/i)).toBeInTheDocument();
  });

  it("searches loaded task logs and shows matches", async () => {
    const user = userEvent.setup();
    render(<ProxmoxTasksPage />);
    await waitFor(() => expect(screen.getByText("aptupdate")).toBeInTheDocument());

    await user.type(screen.getByPlaceholderText(/search task logs/i), "fetch");
    await user.click(screen.getByRole("button", { name: /search logs/i }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("search_task_logs", {
        clusterId: "cluster-1",
        query: "fetch",
        targets: [
          { node: "vmhost1", upid: tasks[0].upid },
          { node: "vmhost2", upid: tasks[1].upid },
        ],
      })
    );

    await waitFor(() =>
      expect(screen.getByText(/Failed to fetch http:\/\/deb.debian.org/)).toBeInTheDocument()
    );
    // Only vmhost1's task had matches; vmhost2's zero-match result must not
    // produce a "View Full Log" entry in the search results section.
    expect(screen.getAllByRole("button", { name: /view full log/i })).toHaveLength(1);
  });

  it("disables the search button when the query is too short", async () => {
    const user = userEvent.setup();
    render(<ProxmoxTasksPage />);
    await waitFor(() => expect(screen.getByText("aptupdate")).toBeInTheDocument());

    await user.type(screen.getByPlaceholderText(/search task logs/i), "a");
    expect(screen.getByRole("button", { name: /search logs/i })).toBeDisabled();
  });

  it("opens the full task log dialog when a matched task is clicked", async () => {
    const user = userEvent.setup();
    render(<ProxmoxTasksPage />);
    await waitFor(() => expect(screen.getByText("aptupdate")).toBeInTheDocument());

    await user.type(screen.getByPlaceholderText(/search task logs/i), "fetch");
    await user.click(screen.getByRole("button", { name: /search logs/i }));
    await waitFor(() => expect(screen.getByText(/Failed to fetch/)).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /view full log/i }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_proxmox_task_log", {
        clusterId: "cluster-1",
        node: "vmhost1",
        upid: tasks[0].upid,
      })
    );
    await waitFor(() => expect(screen.getByText("downloading")).toBeInTheDocument());
  });
});
