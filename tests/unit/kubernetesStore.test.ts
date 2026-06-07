import { describe, it, expect, beforeEach } from "vitest";
import { useKubernetesStore } from "@/stores/kubernetesStore";
import type { ResourceInfo } from "@/lib/tauriCommands";

describe("Kubernetes Store", () => {
  beforeEach(() => {
    useKubernetesStore.getState().clusters.forEach((c) => 
      useKubernetesStore.getState().removeCluster(c.id)
    );
  });

  describe("Cluster Management", () => {
    it("should add a cluster", () => {
      const cluster = {
        id: "cluster-1",
        name: "Production",
        context: "prod-context",
        cluster_url: "https://k8s.example.com",
      };
      
      useKubernetesStore.getState().addCluster(cluster);
      
      expect(useKubernetesStore.getState().clusters).toHaveLength(1);
      expect(useKubernetesStore.getState().clusters[0].name).toBe("Production");
    });

    it("should remove a cluster", () => {
      const cluster = {
        id: "cluster-1",
        name: "Production",
        context: "prod-context",
        cluster_url: "https://k8s.example.com",
      };
      
      useKubernetesStore.getState().addCluster(cluster);
      useKubernetesStore.getState().removeCluster("cluster-1");
      
      expect(useKubernetesStore.getState().clusters).toHaveLength(0);
    });

    it("should update a cluster", () => {
      const cluster = {
        id: "cluster-1",
        name: "Production",
        context: "prod-context",
        cluster_url: "https://k8s.example.com",
      };
      
      useKubernetesStore.getState().addCluster(cluster);
      useKubernetesStore.getState().updateCluster("cluster-1", { name: "Production New" });
      
      expect(useKubernetesStore.getState().clusters[0].name).toBe("Production New");
    });

    it("should set selected cluster", () => {
      const cluster = {
        id: "cluster-1",
        name: "Production",
        context: "prod-context",
        cluster_url: "https://k8s.example.com",
      };
      
      useKubernetesStore.getState().addCluster(cluster);
      useKubernetesStore.getState().setSelectedCluster("cluster-1");
      
      expect(useKubernetesStore.getState().selectedClusterId).toBe("cluster-1");
    });
  });

  describe("Namespace Management", () => {
    it("should set selected namespace", () => {
      useKubernetesStore.getState().setSelectedNamespace("default");
      expect(useKubernetesStore.getState().selectedNamespace).toBe("default");
    });

    it("should set namespaces for a cluster", () => {
      useKubernetesStore.getState().setNamespaces("cluster-1", ["default", "kube-system", "production"]);
      expect(useKubernetesStore.getState().namespaces["cluster-1"]).toEqual(["default", "kube-system", "production"]);
    });
  });

  describe("Resource Loading", () => {
    it("should mark resource as loaded", () => {
      useKubernetesStore.getState().markResourceLoaded("pods");
      expect(useKubernetesStore.getState().isResourceLoaded("pods")).toBe(true);
    });

    it("should mark resource as unloaded", () => {
      useKubernetesStore.getState().markResourceLoaded("pods");
      useKubernetesStore.getState().markResourceUnloaded("pods");
      expect(useKubernetesStore.getState().isResourceLoaded("pods")).toBe(false);
    });
  });

  describe("Terminal Sessions", () => {
    it("should add a terminal session", () => {
      const sessionId = useKubernetesStore.getState().addTerminalSession({
        clusterId: "cluster-1",
        namespace: "default",
        pod: "nginx",
        container: "nginx",
        command: "bash",
      });
      
      expect(sessionId).toBe("terminal-1");
      expect(useKubernetesStore.getState().terminalSessions[sessionId]).toBeDefined();
    });

    it("should remove a terminal session", () => {
      const sessionId = useKubernetesStore.getState().addTerminalSession({
        clusterId: "cluster-1",
        namespace: "default",
        pod: "nginx",
        container: "nginx",
        command: "bash",
      });
      
      useKubernetesStore.getState().removeTerminalSession(sessionId);
      expect(useKubernetesStore.getState().terminalSessions[sessionId]).toBeUndefined();
    });
  });

  describe("Search", () => {
    it("should set global search query", () => {
      useKubernetesStore.getState().setGlobalSearchQuery("nginx");
      expect(useKubernetesStore.getState().globalSearchQuery).toBe("nginx");
    });

    it("should set search results", () => {
      const results = [{ name: "nginx-1", namespace: "default" }];
      useKubernetesStore.getState().setSearchResults("pods", results as ResourceInfo[]);
      
      expect(useKubernetesStore.getState().searchResults.pods).toEqual(results);
    });
  });

  describe("Bulk Selection", () => {
    it("should add to bulk selection", () => {
      useKubernetesStore.getState().addToBulkSelection("pods", "nginx-1");
      expect(useKubernetesStore.getState().bulkSelection.pods).toContain("nginx-1");
    });

    it("should remove from bulk selection", () => {
      useKubernetesStore.getState().addToBulkSelection("pods", "nginx-1");
      useKubernetesStore.getState().removeFromBulkSelection("pods", "nginx-1");
      expect(useKubernetesStore.getState().bulkSelection.pods).not.toContain("nginx-1");
    });

    it("should clear bulk selection", () => {
      useKubernetesStore.getState().addToBulkSelection("pods", "nginx-1");
      useKubernetesStore.getState().clearBulkSelection("pods");
      expect(useKubernetesStore.getState().bulkSelection.pods).toEqual([]);
    });

    it("should get bulk selection count", () => {
      useKubernetesStore.getState().addToBulkSelection("pods", "nginx-1");
      useKubernetesStore.getState().addToBulkSelection("pods", "nginx-2");
      expect(useKubernetesStore.getState().getBulkSelectionCount("pods")).toBe(2);
    });
  });
});
