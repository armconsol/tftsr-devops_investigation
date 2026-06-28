import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";

// Mock the invoke function
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Import the remote desktop types and commands
import {
  addRemoteConnectionCmd,
  updateRemoteConnectionCmd,
  removeRemoteConnectionCmd,
  listRemoteConnectionsCmd,
  getRemoteConnectionCmd,
  testRemoteConnectionCmd,
  connectRemoteCmd,
  disconnectRemoteCmd,
  testRdpConnectionCmd,
  connectRdpCmd,
  testVncConnectionCmd,
  connectVncCmd,
  type RemoteConnection,
  type RemoteConnectionSummary,
  type NewRemoteConnection,
  type RemoteConnectionUpdate,
  type Protocol,
} from "@/lib/tauriCommands";

describe("Remote Desktop Types", () => {
  it("should have valid Protocol type", () => {
    const rdp: Protocol = "rdp";
    const vnc: Protocol = "vnc";
    expect(rdp).toBe("rdp");
    expect(vnc).toBe("vnc");
  });

  it("should create a valid NewRemoteConnection", () => {
    const connection: NewRemoteConnection = {
      name: "Test Server",
      protocol: "rdp",
      hostname: "192.168.1.100",
      port: 3389,
      username: "admin",
      password: "secure-password",
      domain: "WORKGROUP",
      resolution: "1920x1080",
      color_depth: 32,
      clipboard_sync: true,
      drive_redirect: false,
      multi_monitor: false,
      compression: true,
      quality: 80,
    };

    expect(connection.name).toBe("Test Server");
    expect(connection.protocol).toBe("rdp");
    expect(connection.hostname).toBe("192.168.1.100");
    expect(connection.port).toBe(3389);
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
      protocol: "vnc",
      hostname: "10.0.0.50",
      port: 5900,
      username: "root",
      status: "disconnected",
      created_at: "2024-01-15T10:30:00Z",
      updated_at: "2024-01-20T14:45:00Z",
      last_connected_at: "2024-01-19T09:00:00Z",
    };

    expect(summary.id).toBe("123e4567-e89b-12d3-a456-426614174000");
    expect(summary.status).toBe("disconnected");
  });
});

describe("Remote Desktop Commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("addRemoteConnectionCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const mockConnection: NewRemoteConnection = {
        name: "Test Server",
        protocol: "rdp",
        hostname: "192.168.1.100",
        port: 3389,
        username: "admin",
        password: "password123",
      };

      const mockResult: RemoteConnection = {
        id: "123e4567-e89b-12d3-a456-426614174000",
        name: "Test Server",
        protocol: "rdp",
        hostname: "192.168.1.100",
        port: 3389,
        username: "admin",
        password_encrypted: "encrypted-password",
        resolution: "1280x800",
        color_depth: 32,
        clipboard_sync: true,
        drive_redirect: false,
        multi_monitor: false,
        compression: true,
        quality: 80,
        created_at: "2024-01-15T10:30:00Z",
        updated_at: "2024-01-15T10:30:00Z",
      };

      vi.mocked(invoke).mockResolvedValue(mockResult);

      const result = await addRemoteConnectionCmd(mockConnection);

      expect(invoke).toHaveBeenCalledWith("add_remote_connection_cmd", mockConnection);
      expect(result).toEqual(mockResult);
    });
  });

  describe("updateRemoteConnectionCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const connectionId = "123e4567-e89b-12d3-a456-426614174000";
      const updates: RemoteConnectionUpdate = {
        name: "Updated Name",
        resolution: "1920x1080",
      };

      vi.mocked(invoke).mockResolvedValue(undefined);

      await updateRemoteConnectionCmd(connectionId, updates);

      expect(invoke).toHaveBeenCalledWith("update_remote_connection_cmd", {
        id: connectionId,
        ...updates,
      });
    });
  });

  describe("removeRemoteConnectionCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const connectionId = "123e4567-e89b-12d3-a456-426614174000";

      vi.mocked(invoke).mockResolvedValue(undefined);

      await removeRemoteConnectionCmd(connectionId);

      expect(invoke).toHaveBeenCalledWith("remove_remote_connection_cmd", {
        id: connectionId,
      });
    });
  });

  describe("listRemoteConnectionsCmd", () => {
    it("should call invoke without filter when no filter provided", async () => {
      vi.mocked(invoke).mockResolvedValue([]);

      await listRemoteConnectionsCmd();

      expect(invoke).toHaveBeenCalledWith("list_remote_connections_cmd", {
        protocol: undefined,
        name: undefined,
        limit: undefined,
        offset: undefined,
      });
    });

    it("should call invoke with filter parameters", async () => {
      const filter = {
        protocol: "rdp" as Protocol,
        name: "test",
        limit: 10,
        offset: 0,
      };

      vi.mocked(invoke).mockResolvedValue([]);

      await listRemoteConnectionsCmd(filter);

      expect(invoke).toHaveBeenCalledWith("list_remote_connections_cmd", filter);
    });
  });

  describe("getRemoteConnectionCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const connectionId = "123e4567-e89b-12d3-a456-426614174000";

      vi.mocked(invoke).mockResolvedValue(undefined);

      await getRemoteConnectionCmd(connectionId);

      expect(invoke).toHaveBeenCalledWith("get_remote_connection_cmd", {
        id: connectionId,
      });
    });
  });

  describe("testRemoteConnectionCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const connectionId = "123e4567-e89b-12d3-a456-426614174000";

      vi.mocked(invoke).mockResolvedValue(true);

      const result = await testRemoteConnectionCmd(connectionId);

      expect(invoke).toHaveBeenCalledWith("test_remote_connection_cmd", {
        id: connectionId,
      });
      expect(result).toBe(true);
    });
  });

  describe("connectRemoteCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const connectionId = "123e4567-e89b-12d3-a456-426614174000";
      const mockWsUrl = "ws://127.0.0.1:8765/rdp/session-id";

      vi.mocked(invoke).mockResolvedValue(mockWsUrl);

      const result = await connectRemoteCmd(connectionId);

      expect(invoke).toHaveBeenCalledWith("connect_remote_cmd", {
        id: connectionId,
      });
      expect(result).toBe(mockWsUrl);
    });
  });

  describe("disconnectRemoteCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const sessionId = "session-123";

      vi.mocked(invoke).mockResolvedValue(undefined);

      await disconnectRemoteCmd(sessionId);

      expect(invoke).toHaveBeenCalledWith("disconnect_remote_cmd", {
        sessionId,
      });
    });
  });
});

describe("RDP-specific Commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("testRdpConnectionCmd", () => {
    it("should call invoke with correct parameters", async () => {
      vi.mocked(invoke).mockResolvedValue(true);

      const result = await testRdpConnectionCmd(
        "192.168.1.100",
        3389,
        "admin",
        "WORKGROUP",
        "password123"
      );

      expect(invoke).toHaveBeenCalledWith("test_rdp_connection_cmd", {
        hostname: "192.168.1.100",
        port: 3389,
        username: "admin",
        domain: "WORKGROUP",
        password: "password123",
      });
      expect(result).toBe(true);
    });

    it("should call invoke with minimal parameters", async () => {
      vi.mocked(invoke).mockResolvedValue(true);

      const result = await testRdpConnectionCmd("192.168.1.100", 3389);

      expect(invoke).toHaveBeenCalledWith("test_rdp_connection_cmd", {
        hostname: "192.168.1.100",
        port: 3389,
        username: undefined,
        domain: undefined,
        password: undefined,
      });
      expect(result).toBe(true);
    });
  });

  describe("connectRdpCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const mockWsUrl = "ws://127.0.0.1:8765/rdp/session-id";

      vi.mocked(invoke).mockResolvedValue(mockWsUrl);

      const result = await connectRdpCmd(
        "192.168.1.100",
        3389,
        "admin",
        "WORKGROUP",
        "password123",
        "1920x1080",
        32,
        true
      );

      expect(invoke).toHaveBeenCalledWith("connect_rdp_cmd", {
        hostname: "192.168.1.100",
        port: 3389,
        username: "admin",
        domain: "WORKGROUP",
        password: "password123",
        resolution: "1920x1080",
        color_depth: 32,
        clipboard_sync: true,
      });
      expect(result).toBe(mockWsUrl);
    });
  });
});

describe("VNC-specific Commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("testVncConnectionCmd", () => {
    it("should call invoke with correct parameters", async () => {
      vi.mocked(invoke).mockResolvedValue(true);

      const result = await testVncConnectionCmd("192.168.1.100", 5900, "vnc-password");

      expect(invoke).toHaveBeenCalledWith("test_vnc_connection_cmd", {
        hostname: "192.168.1.100",
        port: 5900,
        password: "vnc-password",
      });
      expect(result).toBe(true);
    });
  });

  describe("connectVncCmd", () => {
    it("should call invoke with correct parameters", async () => {
      const mockWsUrl = "ws://127.0.0.1:8765/vnc/session-id";

      vi.mocked(invoke).mockResolvedValue(mockWsUrl);

      const result = await connectVncCmd(
        "192.168.1.100",
        5900,
        "vnc-password",
        "1920x1080",
        true
      );

      expect(invoke).toHaveBeenCalledWith("connect_vnc_cmd", {
        hostname: "192.168.1.100",
        port: 5900,
        password: "vnc-password",
        resolution: "1920x1080",
        clipboard_sync: true,
      });
      expect(result).toBe(mockWsUrl);
    });
  });
});
