// Proxmox domain types
// Defines TypeScript types for Proxmox entities

export type ClusterType = "ve" | "pbs";

export interface ClusterInfo {
  id: string;
  name: string;
  clusterType: ClusterType;
  url: string;
  port: number;
  username: string;
  createdAt: string;
  updatedAt: string;
}

export interface ClusterConnection {
  url: string;
  port: number;
}

export interface VmInfo {
  id: number;
  name?: string;
  status: string;
  cpu: number;
  memory: number;
  disk: number;
  uptime: number;
  node: string;
  template?: boolean;
  agent?: string;
  mem?: number;
  maxMem?: number;
  maxDisk?: number;
  netIn?: number;
  netOut?: number;
  diskRead?: number;
  diskWrite?: number;
}

export interface BackupJob {
  jobId: number;
  name: string;
  schedule: string;
  enabled: boolean;
  datastore: string;
  source: string;
  retention: string;
}

export interface DatastoreInfo {
  datastore: string;
  node: string;
  size: number;
  used: number;
  available: number;
  status: string;
}

export interface CephPool {
  pool: string;
  poolId: number;
  size: number;
  minSize: number;
  pgNum: number;
  used: number;
  avail: number;
  status: string;
}

export interface CephOsd {
  osd: number;
  up: boolean;
  in: boolean;
  weight: number;
  pgNum: number;
  usage: number;
}

export interface FirewallRule {
  ruleNum: number;
  action: string;
  protocol: string;
  source: string;
  destination: string;
  port?: string;
  enabled: boolean;
}

export interface HaGroup {
  group: string;
  nodes: string[];
  maxFailures: number;
  maxRelocate: number;
  state: string;
}
