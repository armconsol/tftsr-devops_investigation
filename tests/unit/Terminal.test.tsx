import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

// ── xterm mocks ───────────────────────────────────────────────────────────────
// onData callbacks registered by the component — keyed by call order
const onDataHandlers: Array<(data: string) => void> = [];

const mockTerminalInstance = {
  open: vi.fn(),
  write: vi.fn(),
  writeln: vi.fn(),
  dispose: vi.fn(),
  onData: vi.fn((cb: (data: string) => void) => {
    onDataHandlers.push(cb);
  }),
  loadAddon: vi.fn(),
  options: {} as Record<string, unknown>,
};

// Must use function (not arrow) so `new` works
vi.mock("xterm", () => ({
  Terminal: vi.fn(function () {
    return mockTerminalInstance;
  }),
}));

const mockFitAddon = { fit: vi.fn(), dispose: vi.fn() };
vi.mock("xterm-addon-fit", () => ({
  FitAddon: vi.fn(function () {
    return mockFitAddon;
  }),
}));

const mockWebLinksAddon = { dispose: vi.fn() };
vi.mock("xterm-addon-web-links", () => ({
  WebLinksAddon: vi.fn(function () {
    return mockWebLinksAddon;
  }),
}));

// ── Tauri command mock ────────────────────────────────────────────────────────
vi.mock("@/lib/tauriCommands", () => ({
  execPodCmd: vi.fn(),
}));

import * as tauriCommands from "@/lib/tauriCommands";
import { Terminal } from "@/components/Kubernetes/Terminal";

type MockedFn<T extends (...args: unknown[]) => unknown = (...args: unknown[]) => unknown> =
  T & ReturnType<typeof vi.fn>;

const execPodCmdMock = tauriCommands.execPodCmd as MockedFn;

const defaultProps = {
  clusterId: "cluster-1",
  namespace: "default",
};

const withPodProps = {
  ...defaultProps,
  podName: "nginx-abc",
  containerName: "nginx",
};

// ── helper: get the onData handler registered for a session ──────────────────
function getOnDataCallback(): (data: string) => void {
  const cb = onDataHandlers[onDataHandlers.length - 1];
  if (!cb) throw new Error("No onData handler registered — terminal may not have mounted");
  return cb;
}

// ── tests ─────────────────────────────────────────────────────────────────────
describe("Terminal component", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    onDataHandlers.length = 0;
    // Re-wire onData to push into our handler array after clearAllMocks
    mockTerminalInstance.onData.mockImplementation((cb: (data: string) => void) => {
      onDataHandlers.push(cb);
    });
    execPodCmdMock.mockResolvedValue({ stdout: "hello\nworld", stderr: "", exit_code: 0 });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("empty state", () => {
    it("renders without crashing", () => {
      render(<Terminal {...defaultProps} />);
    });

    it("shows 'Select a pod to connect' when no pod/container is provided", () => {
      render(<Terminal {...defaultProps} />);
      expect(screen.getByText(/select a pod to connect/i)).toBeInTheDocument();
    });

    it("does not show a tab bar when there are no sessions", () => {
      render(<Terminal {...defaultProps} />);
      expect(screen.queryByRole("tab")).not.toBeInTheDocument();
    });
  });

  describe("session management", () => {
    it("shows tab bar when a session is auto-created from props", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => {
        expect(screen.getByRole("tab")).toBeInTheDocument();
      });
    });

    it("tab label contains pod/container name", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));
      expect(screen.getByRole("tab").textContent).toContain("nginx-abc");
    });

    it("clicking '+' button adds a new session tab", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      expect(screen.getAllByRole("tab")).toHaveLength(1);

      const addButton = screen.getByRole("button", { name: /add session/i });
      await userEvent.click(addButton);

      await waitFor(() => {
        expect(screen.getAllByRole("tab")).toHaveLength(2);
      });
    });

    it("clicking the X on a tab removes that session", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const closeBtn = screen.getByRole("button", { name: /close/i });
      await userEvent.click(closeBtn);

      await waitFor(() => {
        expect(screen.queryByRole("tab")).not.toBeInTheDocument();
      });
    });

    it("removing the last session goes back to the empty state", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const closeBtn = screen.getByRole("button", { name: /close/i });
      await userEvent.click(closeBtn);

      await waitFor(() => {
        expect(screen.getByText(/select a pod to connect/i)).toBeInTheDocument();
      });
    });
  });

  describe("IPC integration", () => {
    it("calls execPodCmd with correct arguments when a command is entered", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      // onData must have been registered by now
      expect(mockTerminalInstance.onData).toHaveBeenCalled();
      const onDataCallback = getOnDataCallback();

      await act(async () => {
        onDataCallback("l");
        onDataCallback("s");
        onDataCallback("\r");
      });

      await waitFor(() => {
        expect(execPodCmdMock).toHaveBeenCalledWith(
          "cluster-1",
          "default",
          "nginx-abc",
          "nginx",
          "ls",
          expect.any(String)
        );
      });
    });

    it("writes command output to the terminal after execution", async () => {
      execPodCmdMock.mockResolvedValue({ stdout: "file1.txt\nfile2.txt", stderr: "", exit_code: 0 });

      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const onDataCallback = getOnDataCallback();

      await act(async () => {
        onDataCallback("l");
        onDataCallback("s");
        onDataCallback("\r");
      });

      await waitFor(() => {
        const writeCalls = mockTerminalInstance.write.mock.calls.map((c: unknown[]) => c[0] as string);
        expect(writeCalls.some((s) => s.includes("file1.txt"))).toBe(true);
      });
    });

    it("handles IPC errors gracefully by writing an error message to the terminal", async () => {
      execPodCmdMock.mockRejectedValue(new Error("connection refused"));

      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const onDataCallback = getOnDataCallback();

      await act(async () => {
        onDataCallback("e");
        onDataCallback("c");
        onDataCallback("h");
        onDataCallback("o");
        onDataCallback("\r");
      });

      await waitFor(() => {
        const writeCalls = mockTerminalInstance.write.mock.calls.map((c: unknown[]) => c[0] as string);
        expect(
          writeCalls.some((s) => s.toLowerCase().includes("error") || s.includes("connection refused"))
        ).toBe(true);
      });
    });

    it("writes stderr output to the terminal when exit_code is non-zero", async () => {
      execPodCmdMock.mockResolvedValue({ stdout: "", stderr: "command not found", exit_code: 127 });

      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const onDataCallback = getOnDataCallback();

      await act(async () => {
        onDataCallback("b");
        onDataCallback("a");
        onDataCallback("d");
        onDataCallback("\r");
      });

      await waitFor(() => {
        const writeCalls = mockTerminalInstance.write.mock.calls.map((c: unknown[]) => c[0] as string);
        expect(writeCalls.some((s) => s.includes("command not found"))).toBe(true);
      });
    });
  });

  describe("shell selector", () => {
    it("renders a shell selector dropdown", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const shellSelector = screen.getByRole("combobox");
      expect(shellSelector).toBeInTheDocument();
    });

    it("passes selected shell to execPodCmd", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const shellSelector = screen.getByRole("combobox");
      await userEvent.selectOptions(shellSelector, "sh");

      const onDataCallback = getOnDataCallback();

      await act(async () => {
        onDataCallback("p");
        onDataCallback("w");
        onDataCallback("d");
        onDataCallback("\r");
      });

      await waitFor(() => {
        expect(execPodCmdMock).toHaveBeenCalledWith(
          expect.any(String),
          expect.any(String),
          expect.any(String),
          expect.any(String),
          "pwd",
          "sh"
        );
      });
    });
  });

  describe("cleanup", () => {
    it("calls terminal.dispose() on unmount", async () => {
      const { unmount } = render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      unmount();

      expect(mockTerminalInstance.dispose).toHaveBeenCalled();
    });
  });

  describe("terminal configuration", () => {
    beforeEach(() => {
      localStorage.clear();
    });

    it("renders settings button in tab bar", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const settingsBtn = screen.queryByRole("button", { name: /settings/i });
      expect(settingsBtn).toBeInTheDocument();
    });

    it("opens settings dialog when settings button is clicked", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const settingsBtn = screen.getByRole("button", { name: /settings/i });
      await userEvent.click(settingsBtn);

      await waitFor(() => {
        expect(screen.getByText(/terminal settings/i)).toBeInTheDocument();
      });
    });

    it("shows copy-on-select toggle in settings dialog", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const settingsBtn = screen.getByRole("button", { name: /settings/i });
      await userEvent.click(settingsBtn);

      await waitFor(() => {
        expect(screen.getByLabelText(/copy on select/i)).toBeInTheDocument();
      });
    });

    it("shows font family input in settings dialog", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const settingsBtn = screen.getByRole("button", { name: /settings/i });
      await userEvent.click(settingsBtn);

      await waitFor(() => {
        expect(screen.getByLabelText(/font family/i)).toBeInTheDocument();
      });
    });

    it("shows font size input in settings dialog", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const settingsBtn = screen.getByRole("button", { name: /settings/i });
      await userEvent.click(settingsBtn);

      await waitFor(() => {
        expect(screen.getByLabelText(/font size/i)).toBeInTheDocument();
      });
    });

    it("persists settings to localStorage when changed", async () => {
      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const settingsBtn = screen.getByRole("button", { name: /settings/i });
      await userEvent.click(settingsBtn);

      await waitFor(() => screen.getByLabelText(/copy on select/i));

      const copyOnSelectCheckbox = screen.getByLabelText(/copy on select/i) as HTMLInputElement;
      await userEvent.click(copyOnSelectCheckbox);

      await waitFor(() => {
        const stored = localStorage.getItem("terminal-settings");
        expect(stored).toBeTruthy();
        const parsed = JSON.parse(stored || "{}");
        expect(parsed.copyOnSelect).toBeDefined();
      });
    });

    it("loads settings from localStorage on mount", async () => {
      localStorage.setItem(
        "terminal-settings",
        JSON.stringify({
          copyOnSelect: true,
          fontFamily: "Courier New",
          fontSize: 16,
        })
      );

      render(<Terminal {...withPodProps} />);
      await waitFor(() => screen.getByRole("tab"));

      const settingsBtn = screen.getByRole("button", { name: /settings/i });
      await userEvent.click(settingsBtn);

      await waitFor(() => {
        const copyOnSelectCheckbox = screen.getByLabelText(/copy on select/i) as HTMLInputElement;
        expect(copyOnSelectCheckbox.checked).toBe(true);

        const fontFamilyInput = screen.getByLabelText(/font family/i) as HTMLInputElement;
        expect(fontFamilyInput.value).toBe("Courier New");

        const fontSizeInput = screen.getByLabelText(/font size/i) as HTMLInputElement;
        expect(fontSizeInput.value).toBe("16");
      });
    });
  });
});
