export interface VmTableRow {
  id: string;
  vmid: number;
  name: string;
  node: string;
  status: string;
  cpu: number;
  memory: number;
  memoryTotal: number;
}

export interface VmTableFilters {
  /** Exact node match; empty/undefined means "all nodes". */
  node?: string;
  /** Exact status match; empty/undefined means "all statuses". */
  status?: string;
  /** Only include VMs whose CPU usage is at least this percent (0-100). */
  minCpuPercent?: number;
  /** Only include VMs whose memory usage is at least this percent (0-100). */
  minMemoryPercent?: number;
}

export type VmSortKey = "name" | "vmid" | "node" | "status" | "cpu" | "memory";
export type SortDirection = "asc" | "desc";

export interface FilterAndSortVmsOptions {
  /** Case-insensitive substring match against Name OR VM ID. */
  search?: string;
  filters?: VmTableFilters;
  sortKey?: VmSortKey | null;
  sortDirection?: SortDirection;
}

function cpuPercent(vm: VmTableRow): number {
  return vm.cpu > 0 ? Math.min(vm.cpu * 100, 100) : 0;
}

function memoryPercent(vm: VmTableRow): number {
  return vm.memoryTotal > 0 ? (vm.memory / vm.memoryTotal) * 100 : 0;
}

/**
 * Filters VMs by a global search term (Name or VM ID) and per-column
 * filters, then sorts the result. Never mutates the input array.
 */
export function filterAndSortVms<T extends VmTableRow>(
  vms: T[],
  options: FilterAndSortVmsOptions
): T[] {
  const { search, filters, sortKey, sortDirection = "asc" } = options;

  let result = vms.slice();

  if (search && search.trim()) {
    const needle = search.trim().toLowerCase();
    result = result.filter(
      (vm) =>
        vm.name.toLowerCase().includes(needle) || String(vm.vmid).includes(needle)
    );
  }

  if (filters?.node) {
    result = result.filter((vm) => vm.node === filters.node);
  }
  if (filters?.status) {
    result = result.filter((vm) => vm.status === filters.status);
  }
  if (filters?.minCpuPercent !== undefined) {
    const min = filters.minCpuPercent;
    result = result.filter((vm) => cpuPercent(vm) >= min);
  }
  if (filters?.minMemoryPercent !== undefined) {
    const min = filters.minMemoryPercent;
    result = result.filter((vm) => memoryPercent(vm) >= min);
  }

  if (sortKey) {
    const dir = sortDirection === "desc" ? -1 : 1;
    result = result.sort((a, b) => {
      switch (sortKey) {
        case "vmid":
          return (a.vmid - b.vmid) * dir;
        case "cpu":
          return (cpuPercent(a) - cpuPercent(b)) * dir;
        case "memory":
          return (memoryPercent(a) - memoryPercent(b)) * dir;
        case "name":
          return a.name.localeCompare(b.name) * dir;
        case "node":
          return a.node.localeCompare(b.node) * dir;
        case "status":
          return a.status.localeCompare(b.status) * dir;
        default:
          return 0;
      }
    });
  }

  return result;
}
