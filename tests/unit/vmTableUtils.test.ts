import { describe, it, expect } from "vitest";
import { filterAndSortVms, type VmTableRow } from "@/components/Proxmox/vmTableUtils";

const vms: VmTableRow[] = [
  { id: "100", vmid: 100, name: "web-server", node: "vmhost2", status: "running", cpu: 0.5, memory: 512, memoryTotal: 1024 },
  { id: "101", vmid: 101, name: "db-server", node: "vmhost1", status: "stopped", cpu: 0, memory: 0, memoryTotal: 2048 },
  { id: "202", vmid: 202, name: "cache-node", node: "vmhost1", status: "paused", cpu: 0.1, memory: 100, memoryTotal: 512 },
];

describe("filterAndSortVms", () => {
  it("returns all rows unmodified when no options are given", () => {
    expect(filterAndSortVms(vms, {})).toEqual(vms);
  });

  it("searches by name substring (case-insensitive)", () => {
    const result = filterAndSortVms(vms, { search: "SERVER" });
    expect(result.map((v) => v.vmid)).toEqual([100, 101]);
  });

  it("searches by VM ID substring", () => {
    const result = filterAndSortVms(vms, { search: "20" });
    expect(result.map((v) => v.vmid)).toEqual([202]);
  });

  it("filters by exact node", () => {
    const result = filterAndSortVms(vms, { filters: { node: "vmhost1" } });
    expect(result.map((v) => v.vmid).sort()).toEqual([101, 202]);
  });

  it("filters by status", () => {
    const result = filterAndSortVms(vms, { filters: { status: "running" } });
    expect(result.map((v) => v.vmid)).toEqual([100]);
  });

  it("filters by minimum CPU percent", () => {
    const result = filterAndSortVms(vms, { filters: { minCpuPercent: 20 } });
    expect(result.map((v) => v.vmid)).toEqual([100]);
  });

  it("filters by minimum memory percent", () => {
    // web-server: 512/1024=50%, cache-node: 100/512=~19.5%, db-server: 0/2048=0%
    const result = filterAndSortVms(vms, { filters: { minMemoryPercent: 40 } });
    expect(result.map((v) => v.vmid)).toEqual([100]);
  });

  it("combines search and filters", () => {
    const result = filterAndSortVms(vms, {
      search: "server",
      filters: { node: "vmhost1" },
    });
    expect(result.map((v) => v.vmid)).toEqual([101]);
  });

  it("sorts by name ascending", () => {
    const result = filterAndSortVms(vms, { sortKey: "name", sortDirection: "asc" });
    expect(result.map((v) => v.name)).toEqual(["cache-node", "db-server", "web-server"]);
  });

  it("sorts by name descending", () => {
    const result = filterAndSortVms(vms, { sortKey: "name", sortDirection: "desc" });
    expect(result.map((v) => v.name)).toEqual(["web-server", "db-server", "cache-node"]);
  });

  it("sorts by vmid numerically ascending", () => {
    const result = filterAndSortVms(vms, { sortKey: "vmid", sortDirection: "asc" });
    expect(result.map((v) => v.vmid)).toEqual([100, 101, 202]);
  });

  it("sorts by cpu descending", () => {
    const result = filterAndSortVms(vms, { sortKey: "cpu", sortDirection: "desc" });
    expect(result.map((v) => v.vmid)).toEqual([100, 202, 101]);
  });

  it("sorts by memory percent descending", () => {
    const result = filterAndSortVms(vms, { sortKey: "memory", sortDirection: "desc" });
    // 50%, 19.5%, 0%
    expect(result.map((v) => v.vmid)).toEqual([100, 202, 101]);
  });

  it("sorts by node then does not mutate the input array", () => {
    const original = [...vms];
    filterAndSortVms(vms, { sortKey: "node", sortDirection: "asc" });
    expect(vms).toEqual(original);
  });

  it("sorts by status alphabetically", () => {
    const result = filterAndSortVms(vms, { sortKey: "status", sortDirection: "asc" });
    expect(result.map((v) => v.status)).toEqual(["paused", "running", "stopped"]);
  });
});
