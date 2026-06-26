import { describe, it, expect, beforeEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  listCephOsd,
  listCephMonitors,
  listCephManagers,
  listCephfs,
  getCephFlags,
  type CephOsd,
  type CephMonitor,
  type CephFs,
  type CephFlag,
} from "@/lib/proxmoxClient";

const mockInvoke = vi.mocked(invoke);

describe("proxmoxClient Ceph wrappers", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("listCephOsd invokes list_ceph_osd and returns the flat OSD rows", async () => {
    const rows: CephOsd[] = [
      {
        id: 19,
        host: "vmhost4",
        status: "up",
        weight: 1.74,
        size: 1920378863616,
        used: 66875367424,
        avail: 1853503496192,
        usedPercent: 3.48,
      },
    ];
    mockInvoke.mockResolvedValue(rows);

    const result = await listCephOsd("cluster-1", "vmhost1");
    expect(mockInvoke).toHaveBeenCalledWith("list_ceph_osd", {
      clusterId: "cluster-1",
      node: "vmhost1",
    });
    expect(result[0].id).toBe(19);
    expect(result[0].status).toBe("up");
  });

  it("listCephMonitors maps numeric-quorum-derived bool rows", async () => {
    const rows: CephMonitor[] = [
      { name: "vmhost1", quorum: true, address: "172.19.111.161:6789/0", version: "19.2.3" },
    ];
    mockInvoke.mockResolvedValue(rows);

    const result = await listCephMonitors("cluster-1", "vmhost1");
    expect(mockInvoke).toHaveBeenCalledWith("list_ceph_monitors", {
      clusterId: "cluster-1",
      node: "vmhost1",
    });
    expect(result[0].quorum).toBe(true);
  });

  it("listCephManagers invokes list_ceph_managers", async () => {
    mockInvoke.mockResolvedValue([]);
    await listCephManagers("cluster-1", "vmhost1");
    expect(mockInvoke).toHaveBeenCalledWith("list_ceph_managers", {
      clusterId: "cluster-1",
      node: "vmhost1",
    });
  });

  it("listCephfs surfaces dataPool as a string", async () => {
    const rows: CephFs[] = [
      { name: "cephfs", metadataPool: "cephfs_metadata", dataPool: "cephfs_data" },
    ];
    mockInvoke.mockResolvedValue(rows);

    const result = await listCephfs("cluster-1", "vmhost1");
    expect(result[0].dataPool).toBe("cephfs_data");
  });

  it("getCephFlags returns the {name,value,description} array", async () => {
    const flags: CephFlag[] = [
      { name: "noout", value: 0, description: "OSDs will not automatically be marked out." },
      { name: "nobackfill", value: 1, description: "Backfilling of PGs is suspended." },
    ];
    mockInvoke.mockResolvedValue(flags);

    const result = await getCephFlags("cluster-1", "vmhost1");
    expect(mockInvoke).toHaveBeenCalledWith("get_ceph_flags", {
      clusterId: "cluster-1",
      node: "vmhost1",
    });
    expect(Array.isArray(result)).toBe(true);
    expect(result[1].name).toBe("nobackfill");
    expect(result[1].value).toBe(1);
  });
});
