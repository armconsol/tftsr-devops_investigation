import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import {
  addRemoteConnectionCmd,
  updateRemoteConnectionCmd,
  deleteRemoteConnectionCmd,
  listRemoteConnectionsCmd,
  getRemoteConnectionCmd,
  startRdpSession,
  stopRdpSession,
  getRemoteConnections,
  type RemoteConnection,
  type RemoteConnectionSummary,
  type NewRemoteConnection,
  type RemoteConnectionUpdate,
  type RemoteProtocol,
  type RdpSession,
} from "@/lib/tauriCommands";

describe("Remote Desktop Types", () => {
  it("should create a valid NewRemoteConnection", () => {
    const connection: NewRemoteConnection = {
      name: "Test Server",
      protocol: "rdp" as RemoteProtocol,
      hostname: "192.168.1.100",
      port: 3389,
      username: "admin",
      password: "secure-password",
      domain: "WORKGROUP",
      ssh_enabled: false,
      resolution: "1920x1080",
      color_depth: 32,
      clipboard_sync: true,
      drive_redirect: false,
      multi_monitor: false,
      compression: true,
      quality: 80,
      auto_resize: true,
      stretch_to_fill: false,
    };

    expect(connection.name).toBe("Test Server");
    expect(connection.protocol).toBe("rdp");
    expect(connection.hostname).toBe("192.168.1.100");
    expect(connection.port).toBe(3389);
    expect(connection.ssh_enabled).toBe(false);
  });

  it("should create a valid RemoteConnectionUpdate", () => {
    const updates: RemoteConnectionUpdate = {
      name: "Updated Server Name",
      resolution: "2560x1440",
      quality: 90,
    };

    expect(updates.name).toBe("Updated Server Name");
    expect(updates.resolution).toBe("2560x1440");
    expect(updates.quality).toBe(90);
  });

  it("should create a valid RemoteConnectionSummary", () => {
    const summary: RemoteConnectionSummary = {
      id: "123e4567-e89b-12d3-a456-426614174000",
      name: "Production Server",
      protocol: "vnc" as RemoteProtocol,
      hostname: "10.0.0.50",
      port: 5900,
      username: "root",
      status: "disconnected",
      ssh_enabled: false,
      created_at: "2024-01-15T10:30:00Z",
      updated_at: "2024-01-20T14:45:00Z",
      last_connected_at: "2024-01-19T09:00:00Z",
    };

    expect(summary.id).toBe("123e4567-e89b-12d3-a456-426614174000");
    expect(summary.status).toBe("disconnected");
    expect(summary.ssh_enabled).toBe(false);
  });
});

describe("Remote Desktop Commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("addRemoteConnectionCmd", () => {
    it("should call invoke with create_remote_connection", async () => {
      const mockConnection: NewRemoteConnection = {
        name: "Test Server",
        protocol: "rdp" as RemoteProtocol,
        hostname: "192.168.1.100",
        port: 3389,
        username: "admin",
        password: "password123",
        domain: undefined,
        ssh_enabled: false,
        auto_resize: true,
        stretch_to_fill: false,
      };

      vi.mocked(invoke).mockResolvedValue({} as RemoteConnection);

      await addRemoteConnectionCmd(mockConnection);

      expect(invoke).toHaveBeenCalledWith("create_remote_connection", {
        newConn: mockConnection,
      });
    });
  });

  describe("listRemoteConnectionsCmd", () => {
    it("should call invoke without filter", async () => {
      vi.mocked(invoke).mockResolvedValue([]);

      await listRemoteConnectionsCmd();

      expect(invoke).toHaveBeenCalledWith("list_remote_connections", {
        filter: null,
      });
    });

    it("should pass filter when provided", async () => {
      const filter = { protocol: "rdp" as RemoteProtocol, name: "test" };
      vi.mocked(invoke).mockResolvedValue([]);

      await listRemoteConnectionsCmd(filter);

      expect(invoke).toHaveBeenCalledWith("list_remote_connections", {
        filter,
      });
    });
  });

  describe("getRemoteConnectionCmd", () => {
    it("should call invoke with correct id parameter", async () => {
      const connectionId = "123e4567-e89b-12d3-a456-426614174000";
      vi.mocked(invoke).mockResolvedValue(null);

      await getRemoteConnectionCmd(connectionId);

      expect(invoke).toHaveBeenCalledWith("get_remote_connection", {
        id: connectionId,
      });
    });
  });

  describe("updateRemoteConnectionCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const connectionId = "123e4567-e89b-12d3-a456-426614174000";
      const updates: RemoteConnectionUpdate = { name: "Updated Name" };
      vi.mocked(invoke).mockResolvedValue({} as RemoteConnection);

      await updateRemoteConnectionCmd(connectionId, updates);

      expect(invoke).toHaveBeenCalledWith("update_remote_connection", {
        id: connectionId,
        update: updates,
      });
    });
  });

  describe("deleteRemoteConnectionCmd", () => {
    it("should call invoke with correct id", async () => {
      const connectionId = "123e4567-e89b-12d3-a456-426614174000";
      vi.mocked(invoke).mockResolvedValue(undefined);

      await deleteRemoteConnectionCmd(connectionId);

      expect(invoke).toHaveBeenCalledWith("delete_remote_connection", {
        id: connectionId,
      });
    });
  });

  describe("getRemoteConnections", () => {
    it("should delegate to listRemoteConnectionsCmd", async () => {
      vi.mocked(invoke).mockResolvedValue([]);

      await getRemoteConnections();

      expect(invoke).toHaveBeenCalledWith("list_remote_connections", {
        filter: null,
      });
    });
  });
});

describe("RDP Session Commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("startRdpSession", () => {
    it("should call invoke with connectionId and password", async () => {
      const mockSession: RdpSession = {
        id: "session-123",
        connection_id: "conn-456",
        hostname: "192.168.1.100",
        port: 3389,
        username: "admin",
        resolution: "1920x1080",
        color_depth: 32,
        websocket_port: 8765,
        websocket_url: "ws://127.0.0.1:8765/rdp/session-123",
        connected: true,
        ssh_enabled: false,
      };

      vi.mocked(invoke).mockResolvedValue(mockSession);

      const result = await startRdpSession("conn-456", "password123");

      expect(invoke).toHaveBeenCalledWith("start_rdp_session", {
        connectionId: "conn-456",
        password: "password123",
      });
      expect(result).toEqual(mockSession);
    });

    it("should omit the password so the stored credential is used", async () => {
      const mockSession: RdpSession = {
        id: "session-789",
        connection_id: "conn-456",
        hostname: "192.168.1.100",
        port: 3389,
        username: "admin",
        resolution: "1920x1080",
        color_depth: 32,
        websocket_port: 8765,
        websocket_url: "ws://127.0.0.1:8765/rdp/session-789",
        connected: true,
        ssh_enabled: false,
      };

      vi.mocked(invoke).mockResolvedValue(mockSession);

      const result = await startRdpSession("conn-456");

      expect(invoke).toHaveBeenCalledWith("start_rdp_session", {
        connectionId: "conn-456",
        password: undefined,
      });
      expect(result).toEqual(mockSession);
    });
  });

  describe("stopRdpSession", () => {
    it("should call invoke with sessionId", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await stopRdpSession("session-123");

      expect(invoke).toHaveBeenCalledWith("stop_rdp_session", {
        sessionId: "session-123",
      });
    });
  });
});
