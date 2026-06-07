import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import * as tauriCommands from "@/lib/tauriCommands";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core");

type MockedFunction<T = (...args: unknown[]) => unknown> = T & {
  mockResolvedValue: (value: unknown) => void;
  mockRejectedValue: (error: Error) => void;
};

describe("Kubernetes Management Commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("addClusterCmd", () => {
    it("should call invoke with correct parameters", async () => {
      (invoke as MockedFunction).mockResolvedValue({
        id: "cluster-1",
        name: "production",
        context: "prod-context",
        cluster_url: "https://prod.example.com",
      });

      const result = await tauriCommands.addClusterCmd(
        "cluster-1",
        "production",
        "kubeconfig-content"
      );

      expect(invoke).toHaveBeenCalledWith("add_cluster", {
        id: "cluster-1",
        name: "production",
        kubeconfig_content: "kubeconfig-content",
      });
      expect(result).toEqual({
        id: "cluster-1",
        name: "production",
        context: "prod-context",
        cluster_url: "https://prod.example.com",
      });
    });
  });

  describe("removeClusterCmd", () => {
    it("should call invoke with cluster id", async () => {
      (invoke as MockedFunction).mockResolvedValue(undefined);

      await tauriCommands.removeClusterCmd("cluster-1");

      expect(invoke).toHaveBeenCalledWith("remove_cluster", { id: "cluster-1" });
    });
  });

  describe("listClustersCmd", () => {
    it("should call invoke and return cluster list", async () => {
      (invoke as MockedFunction).mockResolvedValue([
        {
          id: "cluster-1",
          name: "production",
          context: "prod-context",
          cluster_url: "https://prod.example.com",
        },
      ]);

      const result = await tauriCommands.listClustersCmd();

      expect(invoke).toHaveBeenCalledWith("list_clusters");
      expect(result).toHaveLength(1);
      expect(result[0].name).toBe("production");
    });
  });

  describe("startPortForwardCmd", () => {
    it("should call invoke with port forward request", async () => {
      (invoke as MockedFunction).mockResolvedValue({
        id: "pf-1",
        cluster_id: "cluster-1",
        namespace: "default",
        pod: "nginx-abc123",
        container_port: 80,
        local_port: 8080,
        status: "Active",
      });

      const request = {
        cluster_id: "cluster-1",
        namespace: "default",
        pod: "nginx-abc123",
        container_port: 80,
      };

      const result = await tauriCommands.startPortForwardCmd(request);

      expect(invoke).toHaveBeenCalledWith("start_port_forward", { request });
      expect(result).toEqual({
        id: "pf-1",
        cluster_id: "cluster-1",
        namespace: "default",
        pod: "nginx-abc123",
        container_port: 80,
        local_port: 8080,
        status: "Active",
      });
    });
  });

  describe("stopPortForwardCmd", () => {
    it("should call invoke with session id", async () => {
      (invoke as MockedFunction).mockResolvedValue(undefined);

      await tauriCommands.stopPortForwardCmd("pf-1");

      expect(invoke).toHaveBeenCalledWith("stop_port_forward", { id: "pf-1" });
    });
  });

  describe("listPortForwardsCmd", () => {
    it("should call invoke and return port forwards list", async () => {
      (invoke as MockedFunction).mockResolvedValue([
        {
          id: "pf-1",
          cluster_id: "cluster-1",
          namespace: "default",
          pod: "nginx-abc123",
          container_port: 80,
          local_port: 8080,
          status: "Active",
        },
      ]);

      const result = await tauriCommands.listPortForwardsCmd();

      expect(invoke).toHaveBeenCalledWith("list_port_forwards");
      expect(result).toHaveLength(1);
      expect(result[0].pod).toBe("nginx-abc123");
    });
  });
});
