/* eslint-disable @typescript-eslint/no-explicit-any */
// Proxmox client module
// Provides TypeScript client wrapper for Proxmox API

import { invoke } from "@tauri-apps/api/core";
import { ClusterInfo, ClusterType } from "./domain";

/**
 * Add a Proxmox cluster
 * @param id - Unique cluster identifier
 * @param name - Display name for the cluster
 * @param clusterType - Type of cluster (ve or pbs)
 * @param connection - Connection details (url and port)
 * @param username - Root username for authentication
 * @param password - Root password for authentication
 */
export async function addProxmoxCluster(
  id: string,
  name: string,
  clusterType: ClusterType,
  connection: { url: string; port: number },
  username: string,
  password: string
): Promise<ClusterInfo> {
  return await invoke<ClusterInfo>("add_proxmox_cluster", {
    id,
    name,
    clusterType,
    connection,
    username,
    password,
  });
}

/**
 * Remove a Proxmox cluster
 * @param id - Cluster identifier to remove
 */
export async function removeProxmoxCluster(id: string): Promise<void> {
  await invoke("remove_proxmox_cluster", { id });
}

/**
 * Update an existing Proxmox cluster's metadata and credentials atomically.
 * Uses a single SQL UPDATE so there is no window where the record is missing.
 */
export async function updateProxmoxCluster(
  id: string,
  name: string,
  clusterType: ClusterType,
  connection: { url: string; port: number },
  username: string,
  password: string
): Promise<ClusterInfo> {
  return await invoke<ClusterInfo>("update_proxmox_cluster", {
    id,
    name,
    clusterType,
    connection,
    username,
    password,
  });
}

/**
 * Ping a Proxmox cluster — authenticates and calls the version endpoint to verify
 * the API is reachable and credentials are valid.
 */
export async function pingProxmoxCluster(clusterId: string): Promise<unknown> {
  return await invoke("ping_proxmox_cluster", { clusterId });
}

/**
 * Connect (or re-connect) to a cluster stored in the DB.
 * Authenticates against the Proxmox API and populates the in-memory pool.
 * Use after app restart or after an explicit disconnect.
 */
export async function connectProxmoxCluster(clusterId: string): Promise<boolean> {
  return await invoke<boolean>("connect_proxmox_cluster", { clusterId });
}

/**
 * Disconnect from a cluster by removing its authenticated session from the
 * in-memory pool. Credentials are retained in the DB for later reconnection.
 */
export async function disconnectProxmoxCluster(clusterId: string): Promise<void> {
  await invoke("disconnect_proxmox_cluster", { clusterId });
}

/**
 * List all Proxmox clusters
 */
export async function listProxmoxClusters(): Promise<ClusterInfo[]> {
  return await invoke<ClusterInfo[]>("list_proxmox_clusters");
}

/**
 * Get a specific Proxmox cluster
 * @param id - Cluster identifier
 */
export async function getProxmoxCluster(id: string): Promise<ClusterInfo | null> {
  return await invoke<ClusterInfo | null>("get_proxmox_cluster", { id });
}

/**
 * List all Proxmox VMs
 * @param clusterId - Cluster identifier
 */
export async function listProxmoxVms(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_proxmox_vms", { clusterId });
}

/**
 * List all Proxmox LXC containers
 * @param clusterId - Cluster identifier
 */
export async function listProxmoxContainers(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_proxmox_containers", { clusterId });
}

/**
 * Get Proxmox VM details
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param vmId - VM identifier
 */
export async function getProxmoxVm(
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<any> {
  return await invoke<any>("get_proxmox_vm", { clusterId, nodeId, vmId });
}

/**
 * Start a Proxmox VM
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param vmId - VM identifier
 */
export async function startProxmoxVm(
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> {
  await invoke("start_proxmox_vm", { clusterId, nodeId, vmId });
}

/**
 * Stop a Proxmox VM
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param vmId - VM identifier
 */
export async function stopProxmoxVm(
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> {
  await invoke("stop_proxmox_vm", { clusterId, nodeId, vmId });
}

/**
 * Reboot a Proxmox VM
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param vmId - VM identifier
 */
export async function rebootProxmoxVm(
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> {
  await invoke("reboot_proxmox_vm", { clusterId, nodeId, vmId });
}

/**
 * Shutdown a Proxmox VM
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param vmId - VM identifier
 */
export async function shutdownProxmoxVm(
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> {
  await invoke("shutdown_proxmox_vm", { clusterId, nodeId, vmId });
}

export async function suspendProxmoxVm(
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> {
  await invoke("suspend_proxmox_vm", { clusterId, nodeId, vmId });
}

export async function resumeProxmoxVm(
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> {
  await invoke("resume_proxmox_vm", { clusterId, nodeId, vmId });
}

export interface ProxmoxNodeSummary {
  node: string;
  status?: string;
  cpu?: number;
  maxcpu?: number;
  mem?: number;
  maxmem?: number;
  uptime?: number;
  [key: string]: unknown;
}

export async function listProxmoxNodes(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_proxmox_nodes", { clusterId });
}

/**
 * Returns the sorted list of node names for a cluster. Centralises the parsing
 * of the raw PVE `/nodes` payload (where the node name lives in the `node`
 * field) so node dropdowns across the UI stay consistent.
 */
export async function listProxmoxNodeNames(clusterId: string): Promise<string[]> {
  const nodes = await listProxmoxNodes(clusterId);
  return nodes
    .map((n) => (n && typeof n.node === "string" ? (n.node as string) : ""))
    .filter((name): name is string => name.length > 0)
    .sort((a, b) => a.localeCompare(b));
}

export interface CreateVmParams {
  nodeId: string;
  vmid: number;
  name: string;
  memory: number;
  cores: number;
  sockets: number;
  osType: string;
  storage: string;
  diskSize: number;
  netBridge: string;
  iso?: string;
}

export async function createProxmoxVm(
  clusterId: string,
  params: CreateVmParams
): Promise<void> {
  await invoke("create_proxmox_vm", {
    clusterId,
    nodeId: params.nodeId,
    vmid: params.vmid,
    name: params.name,
    memory: params.memory,
    cores: params.cores,
    sockets: params.sockets,
    osType: params.osType,
    storage: params.storage,
    diskSize: params.diskSize,
    netBridge: params.netBridge,
    iso: params.iso ?? null,
  });
}

/**
 * List Proxmox Backup Jobs
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export async function listProxmoxBackupJobs(
  clusterId: string
): Promise<any[]> {
  return await invoke<any[]>("list_proxmox_backup_jobs", { clusterId });
}

/**
 * List Proxmox Datastores
 * @param clusterId - Cluster identifier
 */
export async function listProxmoxDatastores(
  clusterId: string
): Promise<any[]> {
  return await invoke<any[]>("list_proxmox_datastores", { clusterId });
}

/**
 * Get the configuration of a single datacenter-level storage.
 */
export async function getProxmoxStorageConfig(
  clusterId: string,
  storage: string
): Promise<Record<string, any>> {
  return await invoke<Record<string, any>>("get_proxmox_storage_config", {
    clusterId,
    storage,
  });
}

/**
 * Update a datacenter-level storage configuration.
 * Only provided fields are changed.
 */
export async function updateProxmoxStorage(
  clusterId: string,
  storage: string,
  config: { content?: string; nodes?: string; disable?: boolean }
): Promise<void> {
  return await invoke<void>("update_proxmox_storage", {
    clusterId,
    storage,
    content: config.content,
    nodes: config.nodes,
    disable: config.disable,
  });
}

/**
 * Delete a datacenter-level storage configuration.
 */
export async function deleteProxmoxStorage(
  clusterId: string,
  storage: string
): Promise<void> {
  return await invoke<void>("delete_proxmox_storage", { clusterId, storage });
}

/**
 * Trigger Proxmox Backup Job ("Run now").
 * Runs the job's vzdump configuration on its node (or the first cluster node).
 * @param clusterId - Cluster identifier
 * @param jobId - Backup job identifier (string id from cluster/backup)
 */
export async function triggerProxmoxBackupJob(
  clusterId: string,
  jobId: string
): Promise<void> {
  await invoke("trigger_proxmox_backup_job", { clusterId, jobId });
}

/**
 * List Ceph Pools
 * @param clusterId - Cluster identifier
 */
export async function listCephPools(clusterId: string, node: string): Promise<CephPool[]> {
  return await invoke<CephPool[]>("list_ceph_pools", { clusterId, node });
}

/**
 * List Ceph OSDs
 * @param clusterId - Cluster identifier
 * @param node - Node name
 */
export async function listCephOsd(clusterId: string, node: string): Promise<CephOsd[]> {
  return await invoke<CephOsd[]>("list_ceph_osd", { clusterId, node });
}

/**
 * Get Ceph Health
 * @param clusterId - Cluster identifier
 * @param node - Node name
 */
export async function getCephHealth(clusterId: string, node: string): Promise<CephHealth> {
  return await invoke<CephHealth>("get_ceph_health", { clusterId, node });
}

// ─── User Management (LDAP/AD/OpenID) ─────────────────────────────────────────

/**
 * List authentication realms
 * @param clusterId - Cluster identifier
 */
export async function listAuthRealms(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_auth_realms", { clusterId });
}

/**
 * Add LDAP authentication realm
 * @param clusterId - Cluster identifier
 * @param realm - Realm configuration
 */
export async function addLdapRealm(
  clusterId: string,
  realm: any
): Promise<void> {
  await invoke("add_ldap_realm", { clusterId, realm });
}

/**
 * Add Active Directory authentication realm
 * @param clusterId - Cluster identifier
 * @param realm - Realm configuration
 */
export async function addAdRealm(
  clusterId: string,
  realm: any
): Promise<void> {
  await invoke("add_ad_realm", { clusterId, realm });
}

/**
 * Add OpenID Connect authentication realm
 * @param clusterId - Cluster identifier
 * @param realm - Realm configuration
 */
export async function addOpenidRealm(
  clusterId: string,
  realm: any
): Promise<void> {
  await invoke("add_openid_realm", { clusterId, realm });
}

// ─── ACME/Let's Encrypt ───────────────────────────────────────────────────────

/**
 * List ACME accounts
 * @param clusterId - Cluster identifier
 */
export async function listAcmeAccounts(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_acme_accounts", { clusterId });
}

/**
 * Register ACME account
 * @param clusterId - Cluster identifier
 * @param account - Account configuration
 */
export async function registerAcmeAccount(
  clusterId: string,
  account: any
): Promise<void> {
  await invoke("register_acme_account", { clusterId, account });
}

/**
 * Get ACME challenges
 * @param clusterId - Cluster identifier
 */
export async function getAcmeChallenges(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("get_acme_challenges", { clusterId });
}

// ─── APT Repository Management ────────────────────────────────────────────────

/**
 * List APT updates
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export async function listAptUpdates(
  clusterId: string,
  nodeId: string
): Promise<any[]> {
  return await invoke<any[]>("list_apt_updates", { clusterId, node: nodeId });
}

/**
 * Update APT repositories
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export async function updateAptRepos(
  clusterId: string,
  nodeId: string
): Promise<void> {
  await invoke("update_apt_repos", { clusterId, node: nodeId });
}

/**
 * List APT repositories
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export async function listAptRepositories(
  clusterId: string,
  nodeId: string
): Promise<any[]> {
  return await invoke<any[]>("list_apt_repositories", { clusterId, node: nodeId });
}

// ─── Remote Shell ─────────────────────────────────────────────────────────────

/**
 * Get shell ticket for remote terminal access
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export async function getShellTicket(
  clusterId: string,
  nodeId: string
): Promise<any> {
  return await invoke<any>("get_shell_ticket", { clusterId, nodeId });
}

// ─── Certificate Management ───────────────────────────────────────────────────

/**
 * List certificates
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export async function listCertificates(
  clusterId: string,
  nodeId: string
): Promise<any[]> {
  return await invoke<any[]>("list_certificates", { clusterId, nodeId });
}

/**
 * Upload a certificate
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param cert - Certificate data
 */
export async function uploadCertificate(
  clusterId: string,
  nodeId: string,
  cert: any
): Promise<void> {
  await invoke("upload_certificate", { clusterId, nodeId, cert });
}

/**
 * Get certificate details
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param certId - Certificate identifier
 */
export async function getCertificate(
  clusterId: string,
  nodeId: string,
  certId: string
): Promise<any> {
  return await invoke<any>("get_certificate", { clusterId, nodeId, certId });
}

// ─── Firewall Management ──────────────────────────────────────────────────────

/**
 * List firewall rules
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export async function listFirewallRules(
  clusterId: string,
  nodeId: string
): Promise<any[]> {
  return await invoke<any[]>("list_firewall_rules", { clusterId, nodeId });
}

/**
 * Add a firewall rule
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param rule - Rule configuration
 */
export async function addFirewallRule(
  clusterId: string,
  nodeId: string,
  rule: any
): Promise<void> {
  await invoke("add_firewall_rule", { clusterId, nodeId, rule });
}

/**
 * Delete a firewall rule
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param ruleId - Rule identifier
 */
export async function deleteFirewallRule(
  clusterId: string,
  nodeId: string,
  ruleId: number
): Promise<void> {
  await invoke("delete_firewall_rule", { clusterId, nodeId, ruleId });
}

// ─── SDN Management ───────────────────────────────────────────────────────────

/**
 * List SDN controllers
 * @param clusterId - Cluster identifier
 */
export async function listSdnControllers(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_sdn_controllers", { clusterId });
}

/**
 * List SDN virtual networks
 * @param clusterId - Cluster identifier
 */
export async function listSdnVnets(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_sdn_vnets", { clusterId });
}

/**
 * List SDN zones
 * @param clusterId - Cluster identifier
 */
export async function listSdnZones(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_sdn_zones", { clusterId });
}

// ─── Ceph Cluster Management ──────────────────────────────────────────────────

/**
 * List Ceph clusters
 * @param clusterId - Cluster identifier
 */
export async function listCephClusters(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_ceph_clusters", { clusterId });
}

/**
 * Get Ceph cluster status
 * @param clusterId - Cluster identifier
 */
export async function getCephClusterStatus(clusterId: string): Promise<any> {
  return await invoke<any>("get_ceph_cluster_status", { clusterId });
}

// ─── Remote Migration ─────────────────────────────────────────────────────────

/**
 * Migrate a VM
 * @param clusterId - Source cluster identifier
 * @param nodeId - Node identifier
 * @param vmId - VM identifier
 * @param targetClusterId - Target cluster identifier
 * @param online - Whether to migrate online
 */
export async function migrateVm(
  clusterId: string,
  nodeId: string,
  vmId: number,
  targetNode: string,
  targetCluster: string
): Promise<MigrationTaskResult> {
  return await invoke<MigrationTaskResult>("migrate_vm", {
    clusterId,
    nodeId,
    vmId,
    targetNode,
    targetCluster,
  });
}

/** Result of an intra-cluster migration start (carries the task UPID). */
export interface MigrationTaskResult {
  task_id: string;
  source_node: string;
  [key: string]: unknown;
}

/** Result of starting a cross-datacenter (remote) migration. */
export interface RemoteMigrationStart {
  upid: string;
  source_node: string;
  dest_cluster_id: string;
  dest_userid: string;
  dest_tokenname: string;
}

/**
 * Start a cross-datacenter (remote) VM migration. Returns the task UPID plus
 * the temporary destination token details so the caller can clean it up once
 * the migration task completes.
 */
export async function startRemoteMigration(
  clusterId: string,
  node: string,
  vmId: number,
  destClusterId: string,
  targetNode: string,
  targetStorage: string,
  targetBridge: string,
  online: boolean
): Promise<RemoteMigrationStart> {
  return await invoke<RemoteMigrationStart>("start_remote_migration", {
    clusterId,
    node,
    vmId,
    destClusterId,
    targetNode,
    targetStorage,
    targetBridge,
    online,
  });
}

/**
 * List migration status
 * @param clusterId - Cluster identifier
 */
export async function listMigrationStatus(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_migration_status", { clusterId });
}

// ─── System Updates ───────────────────────────────────────────────────────────

/**
 * List updates
 * @param clusterId - Cluster identifier
 */
export async function listUpdates(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_updates", { clusterId });
}

/**
 * Refresh updates
 * @param clusterId - Cluster identifier
 */
export async function refreshUpdates(clusterId: string): Promise<void> {
  await invoke("refresh_updates", { clusterId });
}

/**
 * Install updates
 * @param clusterId - Cluster identifier
 * @param updates - Updates to install
 */
export async function installUpdates(
  clusterId: string,
  updates: any[]
): Promise<void> {
  await invoke("install_updates", { clusterId, updates });
}

// ─── Task Management ──────────────────────────────────────────────────────────

/**
 * List tasks
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export async function listTasks(
  clusterId: string,
  nodeId: string
): Promise<any[]> {
  return await invoke<any[]>("list_tasks", { clusterId, nodeId });
}

/**
 * Get task status
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param taskId - Task identifier
 */
export async function getTaskStatus(
  clusterId: string,
  nodeId: string,
  taskId: string
): Promise<TaskStatusInfo> {
  return await invoke<TaskStatusInfo>("get_task_status", {
    clusterId,
    node: nodeId,
    taskId,
  });
}

/** Task status as returned by the backend `get_task_status` command. */
export interface TaskStatusInfo {
  task_id: string;
  node: string;
  status: string;
  exit_status?: string | null;
  progress?: number;
  [key: string]: unknown;
}

/**
 * Stop a task
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param taskId - Task identifier
 */
export async function stopTask(
  clusterId: string,
  nodeId: string,
  taskId: string
): Promise<void> {
  await invoke("stop_task", { clusterId, nodeId, taskId });
}

// ─── Metric Collection ────────────────────────────────────────────────────────

/**
 * Get metrics summary
 * @param clusterId - Cluster identifier
 */
export async function getMetricsSummary(clusterId: string): Promise<any> {
  return await invoke<any>("get_metrics_summary", { clusterId });
}

/**
 * List metric collections
 * @param clusterId - Cluster identifier
 */
export async function listMetricCollections(
  clusterId: string
): Promise<any[]> {
  return await invoke<any[]>("list_metric_collections", { clusterId });
}

// ─── HA (High Availability) ───────────────────────────────────────────────────

export interface HaGroup {
  id: string;
  nodes: string;
  comment?: string;
  restricted?: boolean;
  noQuorumPolicy?: string;
}

export interface HaResource {
  sid: string;
  group?: string;
  state: string;
  maxRestart?: number;
  maxRelocate?: number;
  comment?: string;
}

/**
 * List HA groups
 * @param clusterId - Cluster identifier
 */
export const listHaGroups = async (clusterId: string): Promise<HaGroup[]> =>
  invoke<HaGroup[]>("list_ha_groups", { clusterId });

/**
 * Create an HA group
 * @param clusterId - Cluster identifier
 * @param config - HA group configuration (group id + node list)
 */
export const createHaGroup = async (
  clusterId: string,
  config: { id: string; nodes: string[] }
): Promise<void> =>
  invoke<void>("create_ha_group", {
    clusterId,
    group: config.id,
    nodes: config.nodes,
  });

/**
 * Update an HA group
 * @param clusterId - Cluster identifier
 * @param id - HA group identifier
 * @param config - HA group fields to update
 */
export const updateHaGroup = async (
  clusterId: string,
  id: string,
  config: {
    nodes: string[];
    comment?: string;
    restricted?: boolean;
    nofailback?: boolean;
  }
): Promise<void> =>
  invoke<void>("update_ha_group", {
    clusterId,
    group: id,
    nodes: config.nodes,
    comment: config.comment,
    restricted: config.restricted,
    nofailback: config.nofailback,
  });

/**
 * Delete an HA group
 * @param clusterId - Cluster identifier
 * @param id - HA group identifier
 */
export const deleteHaGroup = async (
  clusterId: string,
  id: string
): Promise<void> => invoke<void>("delete_ha_group", { clusterId, group: id });

/**
 * List HA resources
 * @param clusterId - Cluster identifier
 */
export const listHaResources = async (
  clusterId: string
): Promise<HaResource[]> =>
  invoke<HaResource[]>("list_ha_resources", { clusterId });

/**
 * Update (edit) an HA resource
 * @param clusterId - Cluster identifier
 * @param sid - HA resource identifier (e.g. "vm:100")
 * @param config - fields to update
 */
export const updateHaResource = async (
  clusterId: string,
  sid: string,
  config: {
    group?: string;
    state?: string;
    maxRestart?: number;
    maxRelocate?: number;
    comment?: string;
  }
): Promise<void> =>
  invoke<void>("update_ha_resource", {
    clusterId,
    resource: sid,
    group: config.group,
    stateValue: config.state,
    maxRestart: config.maxRestart,
    maxRelocate: config.maxRelocate,
    comment: config.comment,
  });

/**
 * Enable an HA resource
 * @param clusterId - Cluster identifier
 * @param id - HA resource identifier
 */
export const enableHaResource = async (
  clusterId: string,
  id: string
): Promise<void> => invoke<void>("enable_ha_resource", { clusterId, resource: id });

export const disableHaResource = async (
  clusterId: string,
  id: string
): Promise<void> => invoke<void>("disable_ha_resource", { clusterId, resource: id });

export const deleteHaResource = async (
  clusterId: string,
  id: string
): Promise<void> => invoke<void>("delete_ha_resource", { clusterId, resource: id });

// ─── ACL / User Management ────────────────────────────────────────────────────

export interface AclEntry {
  path: string;
  type: "user" | "group" | "token";
  ugid: string;
  roleid: string;
  propagate?: boolean;
}

export interface ProxmoxUser {
  userid: string;
  comment?: string;
  email?: string;
  enabled: boolean;
  expire?: number;
  firstname?: string;
  lastname?: string;
  groups?: string[];
}

export interface AuthRealm {
  realm: string;
  type: string;
  comment?: string;
}

/**
 * List ACL entries
 * @param clusterId - Cluster identifier
 */
export const listAcls = async (clusterId: string): Promise<AclEntry[]> =>
  invoke<AclEntry[]>("list_acls", { clusterId });

/**
 * List users
 * @param clusterId - Cluster identifier
 */
export const listUsers = async (clusterId: string): Promise<ProxmoxUser[]> =>
  invoke<ProxmoxUser[]>("list_users", { clusterId });

/**
 * List authentication realms (typed)
 * @param clusterId - Cluster identifier
 */
export const listRealms = async (clusterId: string): Promise<AuthRealm[]> =>
  invoke<AuthRealm[]>("list_realms", { clusterId });

// ─── Cluster Notes ────────────────────────────────────────────────────────────

/**
 * Get cluster notes
 * @param clusterId - Cluster identifier
 */
export const getClusterNotes = async (clusterId: string): Promise<string> =>
  invoke<string>("get_cluster_notes", { clusterId });

/**
 * Update cluster notes
 * @param clusterId - Cluster identifier
 * @param notes - Notes content
 */
export const updateClusterNotes = async (
  clusterId: string,
  notes: string
): Promise<void> => invoke<void>("update_cluster_notes", { clusterId, notes });

// ─── Resource Search ──────────────────────────────────────────────────────────

export interface SearchResult {
  id: string;
  type: "vm" | "container" | "node" | "storage" | "pool";
  name: string;
  node?: string;
  description?: string;
}

/**
 * Search Proxmox resources
 * @param clusterId - Cluster identifier
 * @param query - Search query string
 */
export const searchResources = async (
  clusterId: string,
  query: string
): Promise<SearchResult[]> =>
  invoke<SearchResult[]>("search_proxmox_resources", { clusterId, query });

// ─── Node Status ──────────────────────────────────────────────────────────────

export interface NodeStatus {
  uptime: number;
  memory: { used: number; total: number };
  cpu: number;
  swap: { used: number; total: number };
  disk: { used: number; total: number };
  loadAvg: number[];
  version: string;
}

/**
 * Get node status
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export const getNodeStatus = async (
  clusterId: string,
  nodeId: string
): Promise<NodeStatus> =>
  invoke<NodeStatus>("get_node_status", { clusterId, nodeId });

// ─── APT (typed) ──────────────────────────────────────────────────────────────

export interface AptPackage {
  package: string;
  version: string;
  newVersion?: string;
  priority: string;
  description?: string;
}

export interface AptRepository {
  types: string[];
  uris: string[];
  suites: string[];
  components: string[];
  enabled: boolean;
  comment?: string;
}

// ─── Syslog ───────────────────────────────────────────────────────────────────

export interface SyslogEntry {
  n: number;
  t: string;
  msg: string;
}

/**
 * Get node syslog
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param limit - Maximum number of entries (default 500)
 */
export const getSyslog = async (
  clusterId: string,
  nodeId: string,
  limit?: number
): Promise<SyslogEntry[]> =>
  invoke<SyslogEntry[]>("get_syslog", {
    clusterId,
    nodeId,
    limit: limit ?? 500,
  });

// ─── Network Interfaces ───────────────────────────────────────────────────────

export interface NetworkInterface {
  iface: string;
  type: string;
  address?: string;
  netmask?: string;
  gateway?: string;
  active: boolean;
  autostart: boolean;
  comments?: string;
}

/**
 * List network interfaces on a node
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 */
export const listNetworkInterfaces = async (
  clusterId: string,
  nodeId: string
): Promise<NetworkInterface[]> =>
  invoke<NetworkInterface[]>("list_network_interfaces", { clusterId, nodeId });

/**
 * Network interface configuration for creation/update
 */
export interface NetworkInterfaceConfig {
  iface: string;
  type: string;
  address?: string;
  netmask?: string;
  gateway?: string;
  active?: boolean;
  autostart?: boolean;
  comments?: string;
}

/**
 * Create a network interface
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param config - Network interface configuration
 */
export const createNetworkInterface = async (
  clusterId: string,
  nodeId: string,
  config: NetworkInterfaceConfig
): Promise<void> =>
  invoke<void>("create_network_interface", { clusterId, nodeId, config });

/**
 * Update a network interface
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param iface - Network interface identifier
 * @param config - Updated network interface configuration
 */
export const updateNetworkInterface = async (
  clusterId: string,
  nodeId: string,
  iface: string,
  config: NetworkInterfaceConfig
): Promise<void> =>
  invoke<void>("update_network_interface", { clusterId, nodeId, iface, config });

/**
 * Delete a network interface
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param iface - Network interface identifier
 */
export const deleteNetworkInterface = async (
  clusterId: string,
  nodeId: string,
  iface: string
): Promise<void> =>
  invoke<void>("delete_network_interface", { clusterId, nodeId, iface });

// ─── VM Snapshots ─────────────────────────────────────────────────────────────

export interface ProxmoxSnapshot {
  snapname: string;
  vmid: number;
  name?: string;
  ctime: number;
  parent?: string;
  description?: string;
}

/**
 * List snapshots for a VM
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param vmid - VM identifier
 */
export const listProxmoxSnapshots = async (
  clusterId: string,
  nodeId: string,
  vmid: number
): Promise<ProxmoxSnapshot[]> =>
  invoke<ProxmoxSnapshot[]>("list_proxmox_snapshots", { clusterId, nodeId, vmid });

/**
 * Create a snapshot for a VM
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param vmid - VM identifier
 * @param snapshotName - Snapshot name
 */
export const createProxmoxSnapshot = async (
  clusterId: string,
  nodeId: string,
  vmid: number,
  snapshotName: string
): Promise<void> =>
  invoke<void>("create_proxmox_snapshot", { clusterId, nodeId, vmid, snapshotName });

/**
 * Delete a snapshot for a VM
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param vmid - VM identifier
 * @param snapshotName - Snapshot name
 */
export const deleteProxmoxSnapshot = async (
  clusterId: string,
  nodeId: string,
  vmid: number,
  snapshotName: string
): Promise<void> =>
  invoke<void>("delete_proxmox_snapshot", { clusterId, nodeId, vmid, snapshotName });

/**
 * Rollback a VM to a snapshot
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param vmid - VM identifier
 * @param snapshotName - Snapshot name
 */
export const rollbackProxmoxSnapshot = async (
  clusterId: string,
  nodeId: string,
  vmid: number,
  snapshotName: string
): Promise<void> =>
  invoke<void>("rollback_proxmox_snapshot", { clusterId, nodeId, vmid, snapshotName });

// ─── Subscription ─────────────────────────────────────────────────────────────

export interface SubscriptionStatus {
  status: "active" | "expired" | "none";
  productname?: string;
  regdate?: string;
  nextduedate?: string;
  key?: string;
  serverid?: string;
}

/**
 * Get subscription status
 * @param clusterId - Cluster identifier
 */
export const getSubscriptionStatus = async (
  clusterId: string
): Promise<SubscriptionStatus> =>
  invoke<SubscriptionStatus>("get_subscription_status", { clusterId });

// ─── Cluster Task Log ─────────────────────────────────────────────────────────

export interface ClusterTask {
  upid: string;
  node: string;
  pid: number;
  starttime: number;
  type: string;
  user: string;
  status?: string;
  exitstatus?: string;
}

/**
 * List cluster-level tasks
 * @param clusterId - Cluster identifier
 * @param limit - Maximum number of tasks to return (default 50)
 */
export const listClusterTasks = async (
  clusterId: string,
  limit?: number
): Promise<ClusterTask[]> =>
  invoke<ClusterTask[]>("list_cluster_tasks", {
    clusterId,
    limit: limit ?? 50,
  });

// ─── Storage Per-Node ─────────────────────────────────────────────────────────

/**
 * List storage pools visible on a specific node (filtered from cluster resources)
 */
export const listProxmoxStorages = async (
  clusterId: string,
  nodeId: string
): Promise<{ storage: string; type: string; content?: string }[]> => {
  const all = await listProxmoxDatastores(clusterId);
  return (all as Array<{ storage?: string; node?: string; type?: string; content?: string }>)
    .filter((s) => s.node === nodeId || !s.node)
    .map((s) => ({
      storage: s.storage ?? '',
      type: s.type ?? '',
      content: s.content,
    }))
    .filter((s) => s.storage !== '');
};

// ─── ISO Images ───────────────────────────────────────────────────────────────

/**
 * List ISO images available in a Proxmox storage
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param storageId - Storage pool identifier
 */
export const listIsoImages = async (
  clusterId: string,
  nodeId: string,
  storageId: string
): Promise<{ volid: string; name?: string; size?: number }[]> =>
  invoke<{ volid: string; name?: string; size?: number }[]>("list_iso_images", {
    clusterId,
    nodeId,
    storageId,
  });

/**
 * Upload an ISO file to a Proxmox storage pool.
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param storageId - Storage pool identifier
 * @param filePath - Absolute local path to the .iso file (from file dialog)
 * @returns Proxmox task UPID
 */
export const uploadIsoImage = async (
  clusterId: string,
  nodeId: string,
  storageId: string,
  filePath: string
): Promise<string> =>
  invoke<string>("upload_iso_image", {
    clusterId,
    nodeId,
    storageId,
    filePath,
  });

// ─── VM clone / delete ────────────────────────────────────────────────────────

export interface CloneVmParams extends Record<string, unknown> {
  clusterId: string;
  nodeId: string;
  vmId: number;
  newVmId: number;
  name?: string;
  full?: boolean;
}

export const cloneVm = (params: CloneVmParams): Promise<void> =>
  invoke("clone_vm", params);

export const deleteVm = (
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> => invoke("delete_vm", { clusterId, nodeId, vmId });

// ─── LXC Container Power ──────────────────────────────────────────────────────

export const startProxmoxContainer = (
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> =>
  invoke("start_proxmox_container", { clusterId, nodeId, vmId });

export const stopProxmoxContainer = (
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> =>
  invoke("stop_proxmox_container", { clusterId, nodeId, vmId });

export const rebootProxmoxContainer = (
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> =>
  invoke("reboot_proxmox_container", { clusterId, nodeId, vmId });

export const shutdownProxmoxContainer = (
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> =>
  invoke("shutdown_proxmox_container", { clusterId, nodeId, vmId });

export const suspendProxmoxContainer = (
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> =>
  invoke("suspend_proxmox_container", { clusterId, nodeId, vmId });

export const resumeProxmoxContainer = (
  clusterId: string,
  nodeId: string,
  vmId: number
): Promise<void> =>
  invoke("resume_proxmox_container", { clusterId, nodeId, vmId });

// ─── SDN CRUD ─────────────────────────────────────────────────────────────────

export const createSdnZone = (
  clusterId: string,
  zone: string,
  asn: number,
  vni: number
): Promise<void> => invoke("create_sdn_zone", { clusterId, zone, asn, vni });

export const updateSdnZone = (
  clusterId: string,
  zone: string,
  asn: number,
  vni: number
): Promise<void> => invoke("update_sdn_zone", { clusterId, zone, asn, vni });

export const deleteSdnZone = (
  clusterId: string,
  zone: string
): Promise<void> => invoke("delete_sdn_zone", { clusterId, zone });

export const createSdnVnet = (
  clusterId: string,
  vnet: string,
  zone: string,
  l2vni: number
): Promise<void> =>
  invoke("create_sdn_vnet", { clusterId, vnet, zone, l2vni });

export const updateSdnVnet = (
  clusterId: string,
  vnet: string,
  zone: string,
  l2vni: number
): Promise<void> =>
  invoke("update_sdn_vnet", { clusterId, vnet, zone, l2vni });

export const deleteSdnVnet = (
  clusterId: string,
  vnet: string
): Promise<void> => invoke("delete_sdn_vnet", { clusterId, vnet });

// ─── Backup Job CRUD ──────────────────────────────────────────────────────────

export interface BackupJobParams extends Record<string, unknown> {
  clusterId: string;
  storage: string;
  vmid?: string;
  mode?: string;
  schedule?: string;
  enabled?: boolean;
}

export const createProxmoxBackupJob = (
  params: BackupJobParams
): Promise<void> => invoke("create_proxmox_backup_job", params);

export const updateProxmoxBackupJob = (
  clusterId: string,
  jobId: string,
  updates: Partial<Omit<BackupJobParams, "clusterId">>
): Promise<void> =>
  invoke("update_proxmox_backup_job", { clusterId, jobId, ...updates });

export const deleteProxmoxBackupJob = (
  clusterId: string,
  jobId: string
): Promise<void> =>
  invoke("delete_proxmox_backup_job", { clusterId, jobId });

// ─── ACL CRUD ─────────────────────────────────────────────────────────────────

export const createProxmoxAcl = (
  clusterId: string,
  path: string,
  roles: string,
  users?: string,
  groups?: string,
  propagate?: boolean
): Promise<void> =>
  invoke("create_proxmox_acl", { clusterId, path, roles, users, groups, propagate });

export const deleteProxmoxAcl = (
  clusterId: string,
  path: string,
  roles: string,
  users?: string,
  groups?: string
): Promise<void> =>
  invoke("delete_proxmox_acl", { clusterId, path, roles, users, groups });

// ─── User CRUD ────────────────────────────────────────────────────────────────

export const createProxmoxUser = (
  clusterId: string,
  userid: string,
  password: string,
  comment?: string,
  email?: string,
  enabled?: boolean
): Promise<void> =>
  invoke("create_proxmox_user", {
    clusterId,
    userid,
    password,
    comment,
    email,
    enabled,
  });

export const updateProxmoxUser = (
  clusterId: string,
  userid: string,
  comment?: string,
  email?: string,
  enabled?: boolean
): Promise<void> =>
  invoke("update_proxmox_user", { clusterId, userid, comment, email, enabled });

export const deleteProxmoxUser = (
  clusterId: string,
  userid: string
): Promise<void> => invoke("delete_proxmox_user", { clusterId, userid });

// ─── Realm CRUD ───────────────────────────────────────────────────────────────

export const createProxmoxRealm = (
  clusterId: string,
  realm: string,
  realmType: string,
  comment?: string
): Promise<void> =>
  invoke("create_proxmox_realm", { clusterId, realm, realmType, comment });

export const updateProxmoxRealm = (
  clusterId: string,
  realm: string,
  comment?: string
): Promise<void> =>
  invoke("update_proxmox_realm", { clusterId, realm, comment });

export const deleteProxmoxRealm = (
  clusterId: string,
  realm: string
): Promise<void> => invoke("delete_proxmox_realm", { clusterId, realm });

// ─── Firewall rule update ─────────────────────────────────────────────────────

export const updateFirewallRule = (
  clusterId: string,
  nodeId: string,
  ruleNum: number,
  rule: {
    action: string;
    protocol?: string;
    source?: string;
    destination?: string;
    port?: string;
    enabled: boolean;
    comment?: string;
  }
): Promise<void> =>
  invoke("update_proxmox_firewall_rule", { clusterId, nodeId, ruleNum, rule });

// ─── Node DNS ─────────────────────────────────────────────────────────────────

export interface NodeDns {
  search: string;
  dns1?: string;
  dns2?: string;
  dns3?: string;
}

// ─── Node Time ────────────────────────────────────────────────────────────────

export interface NodeTime {
  localtime: number;
  time: number;
  timezone: string;
}

// ─── VM Pending Config Entry ──────────────────────────────────────────────────

export interface VmPendingEntry {
  key: string;
  value?: unknown;
  pending?: unknown;
  delete?: number;
}

// ─── Ceph ─────────────────────────────────────────────────────────────────────

export interface CephMonitor {
  name: string;
  quorum: boolean;
  address: string;
  version?: string;
}

export interface CephMgr {
  name: string;
  addr?: string;
  state?: string;
}

export interface CephFs {
  name: string;
  metadataPool?: string;
  dataPool?: string;
}

export interface CephHealth {
  status: 'HEALTH_OK' | 'HEALTH_WARN' | 'HEALTH_ERR';
  summary: string;
  details: string[];
}

export interface CephPool {
  id: string;
  name: string;
  type: string;
  size: number;
  minSize: number;
  used: number;
  available: number;
  total: number;
  usedPercent: number;
}

export interface CephOsd {
  id: number;
  host: string;
  status: 'up' | 'down';
  weight: number;
  size: number;
  used: number;
  avail: number;
  usedPercent: number;
}

// ─── Cluster Firewall Status ──────────────────────────────────────────────────

export interface ClusterFirewallStatus {
  enable?: number;
  policyIn?: string;
  policyOut?: string;
}

// ─── TFA ──────────────────────────────────────────────────────────────────────

export interface TfaEntry {
  id: string;
  userid: string;
  tfaType: string;
  description?: string;
  enable?: boolean;
  created?: number;
}

// ─── User Tokens ──────────────────────────────────────────────────────────────

export interface UserToken {
  tokenid: string;
  comment?: string;
  privsep?: number;
  expire?: number;
}

export interface UserTokenCreateResult {
  fullTokenid?: string;
  info?: unknown;
  value?: string;
}

// ─── PBS ──────────────────────────────────────────────────────────────────────

export interface PbsDatastore {
  store: string;
  path?: string;
  total?: number;
  used?: number;
  avail?: number;
  storeType?: string;
}

export interface PbsNamespace {
  ns: string;
  comment?: string;
}

export interface PbsSnapshot {
  backupId: string;
  backupTime: number;
  backupType: string;
  size?: number;
  files?: unknown[];
  verifyState?: string;
  notes?: string;
}

export interface PbsTask {
  upid: string;
  node: string;
  taskType: string;
  status?: string;
  starttime: number;
  endtime?: number;
}

// ─── Node Administration ──────────────────────────────────────────────────────

export const getNodeDns = (
  clusterId: string,
  node: string
): Promise<NodeDns> => invoke("get_node_dns", { clusterId, node });

export const updateNodeDns = (
  clusterId: string,
  node: string,
  search: string,
  dns1?: string,
  dns2?: string,
  dns3?: string
): Promise<void> =>
  invoke("update_node_dns", { clusterId, node, search, dns1, dns2, dns3 });

export const getNodeTime = (
  clusterId: string,
  node: string
): Promise<NodeTime> => invoke("get_node_time", { clusterId, node });

export const updateNodeTime = (
  clusterId: string,
  node: string,
  timezone: string
): Promise<void> => invoke("update_node_time", { clusterId, node, timezone });

export const rebootNode = (
  clusterId: string,
  node: string
): Promise<string> => invoke("reboot_node", { clusterId, node });

export const shutdownNode = (
  clusterId: string,
  node: string
): Promise<string> => invoke("shutdown_node", { clusterId, node });

export const getNodeJournal = (
  clusterId: string,
  node: string,
  lastentries?: number
): Promise<string[]> =>
  invoke("get_node_journal", { clusterId, node, lastentries });

export const getNodeReport = (
  clusterId: string,
  node: string
): Promise<string> => invoke("get_node_report", { clusterId, node });

// ─── Network ──────────────────────────────────────────────────────────────────

export const reloadNetworkConfig = (
  clusterId: string,
  node: string
): Promise<string> => invoke("reload_network_config", { clusterId, node });

// ─── VM Config ────────────────────────────────────────────────────────────────

export const getVmConfig = (
  clusterId: string,
  node: string,
  vmId: number
): Promise<Record<string, unknown>> =>
  invoke("get_vm_config", { clusterId, node, vmId });

export const getVmPendingConfig = (
  clusterId: string,
  node: string,
  vmId: number
): Promise<VmPendingEntry[]> =>
  invoke("get_vm_pending_config", { clusterId, node, vmId });

export const remoteMigrateVm = (
  clusterId: string,
  node: string,
  vmId: number,
  targetNode: string,
  targetStorage: string,
  online: boolean
): Promise<string> =>
  invoke("remote_migrate_vm", {
    clusterId,
    node,
    vmId,
    targetNode,
    targetStorage,
    online,
  });

// ─── VM Console (noVNC) ───────────────────────────────────────────────────

/** Session details for connecting an in-app noVNC client to the local proxy. */
export interface VncConsoleSession {
  local_url: string;
  ticket: string;
  local_port: number;
}

/** Open an in-app noVNC console for a QEMU VM. */
export const openVncConsole = (
  clusterId: string,
  node: string,
  vmId: number
): Promise<VncConsoleSession> =>
  invoke("open_vnc_console", { clusterId, node, vmId });

/** Open an in-app noVNC console for an LXC container. */
export const openLxcConsole = (
  clusterId: string,
  node: string,
  vmId: number
): Promise<VncConsoleSession> =>
  invoke("open_lxc_console", { clusterId, node, vmId });

/** Tagged host-shell session for the Remotes "Console (Shell)" action. */
export interface NodeShellSession {
  /** "novnc" (PVE graphical shell) or "xterm" (PBS terminal). */
  kind: "novnc" | "xterm";
  localUrl: string;
  ticket: string;
  localPort: number;
  /** RFB password for noVNC shells (PVE vncshell only). */
  password: string | null;
  /** Session user (needed for the xterm termproxy login line). */
  user: string;
}

/** Open a host (node) shell for a stored remote (PVE=noVNC, PBS=xterm). */
export const openNodeShell = (
  clusterId: string,
  node: string
): Promise<NodeShellSession> =>
  invoke("open_node_shell", { clusterId, node });

// ─── Container (LXC) ──────────────────────────────────────────────────────────

export const getContainerConfig = (
  clusterId: string,
  node: string,
  vmId: number
): Promise<Record<string, unknown>> =>
  invoke("get_container_config", { clusterId, node, vmId });

export interface ContainerCreateParams {
  vmid: number;
  ostemplate: string;
  hostname?: string;
  memory?: number;
  cores?: number;
  rootfs?: string;
  net0?: string;
  password?: string;
  unprivileged?: boolean;
  start?: boolean;
}

export const createProxmoxContainer = (
  clusterId: string,
  node: string,
  params: ContainerCreateParams
): Promise<string> =>
  invoke("create_proxmox_container", { clusterId, node, ...params });

// ─── RRD Metrics ──────────────────────────────────────────────────────────────

export type RrdTimeframe = "hour" | "day" | "week" | "month" | "year";

export const getNodeRrdData = (
  clusterId: string,
  node: string,
  timeframe: RrdTimeframe
): Promise<Record<string, unknown>[]> =>
  invoke("get_node_rrd_data", { clusterId, node, timeframe });

export const getVmRrdData = (
  clusterId: string,
  node: string,
  vmId: number,
  timeframe: RrdTimeframe
): Promise<Record<string, unknown>[]> =>
  invoke("get_vm_rrd_data", { clusterId, node, vmId, timeframe });

export const getStorageRrdData = (
  clusterId: string,
  node: string,
  storage: string,
  timeframe: RrdTimeframe
): Promise<Record<string, unknown>[]> =>
  invoke("get_storage_rrd_data", { clusterId, node, storage, timeframe });

// ─── Ceph Advanced ────────────────────────────────────────────────────────────

export const listCephMonitors = (clusterId: string, node: string): Promise<CephMonitor[]> =>
  invoke("list_ceph_monitors", { clusterId, node });

export const listCephManagers = (
  clusterId: string,
  node: string
): Promise<CephMgr[]> => invoke("list_ceph_managers", { clusterId, node });

export const listCephfs = (
  clusterId: string,
  node: string
): Promise<CephFs[]> => invoke("list_cephfs", { clusterId, node });

export interface CephFlag {
  name: string;
  value: number;
  description?: string;
}

export const getCephFlags = (
  clusterId: string,
  node: string
): Promise<CephFlag[]> =>
  invoke("get_ceph_flags", { clusterId, node });

// ─── Firewall (cluster + guest) ───────────────────────────────────────────────

export const listClusterFirewallRules = (
  clusterId: string
): Promise<any[]> => invoke("list_cluster_firewall_rules", { clusterId });

export const getClusterFirewallStatus = (
  clusterId: string
): Promise<ClusterFirewallStatus> =>
  invoke("get_cluster_firewall_status", { clusterId });

export const listGuestFirewallRules = (
  clusterId: string,
  node: string,
  vmId: number
): Promise<any[]> =>
  invoke("list_guest_firewall_rules", { clusterId, node, vmId });

export const addGuestFirewallRule = (
  clusterId: string,
  node: string,
  vmId: number,
  action: string,
  proto?: string,
  source?: string,
  dest?: string,
  dport?: string,
  enable?: boolean
): Promise<void> =>
  invoke("add_guest_firewall_rule", {
    clusterId,
    node,
    vmId,
    action,
    proto,
    source,
    dest,
    dport,
    enable,
  });

export const deleteGuestFirewallRule = (
  clusterId: string,
  node: string,
  vmId: number,
  pos: number
): Promise<void> =>
  invoke("delete_guest_firewall_rule", { clusterId, node, vmId, pos });

// ─── TFA Management ───────────────────────────────────────────────────────────

export const listTfaEntries = (clusterId: string): Promise<TfaEntry[]> =>
  invoke("list_tfa_entries", { clusterId });

export const addTfaEntry = (
  clusterId: string,
  userid: string,
  tfaType: string,
  description?: string,
  totp?: string,
  value?: string,
  key?: string
): Promise<unknown> =>
  invoke("add_tfa_entry", {
    clusterId,
    userid,
    tfaType,
    description,
    totp,
    value,
    key,
  });

export const deleteTfaEntry = (
  clusterId: string,
  userid: string,
  id: string
): Promise<void> => invoke("delete_tfa_entry", { clusterId, userid, id });

// ─── User API Tokens ──────────────────────────────────────────────────────────

export const listUserTokens = (
  clusterId: string,
  userid: string
): Promise<UserToken[]> =>
  invoke("list_user_tokens", { clusterId, userid });

export const createUserToken = (
  clusterId: string,
  userid: string,
  tokenname: string,
  comment?: string,
  privsep?: boolean,
  expire?: number
): Promise<UserTokenCreateResult> =>
  invoke("create_user_token", {
    clusterId,
    userid,
    tokenname,
    comment,
    privsep,
    expire,
  });

export const deleteUserToken = (
  clusterId: string,
  userid: string,
  tokenname: string
): Promise<void> =>
  invoke("delete_user_token", { clusterId, userid, tokenname });

// ─── PBS Management ───────────────────────────────────────────────────────────

export const listPbsDatastores = (
  clusterId: string
): Promise<PbsDatastore[]> => invoke("list_pbs_datastores", { clusterId });

export const getPbsDatastoreStatus = (
  clusterId: string,
  store: string
): Promise<Record<string, unknown>> =>
  invoke("get_pbs_datastore_status", { clusterId, store });

export const listPbsNamespaces = (
  clusterId: string,
  store: string
): Promise<PbsNamespace[]> =>
  invoke("list_pbs_namespaces", { clusterId, store });

export const listPbsSnapshots = (
  clusterId: string,
  store: string,
  ns?: string
): Promise<PbsSnapshot[]> =>
  invoke("list_pbs_snapshots", { clusterId, store, ns });

export const listPbsTasks = (
  clusterId: string,
  node: string
): Promise<PbsTask[]> => invoke("list_pbs_tasks", { clusterId, node });

export const getPbsNodeStatus = (
  clusterId: string,
  node: string
): Promise<Record<string, unknown>> =>
  invoke("get_pbs_node_status", { clusterId, node });

// ─── Subscription ─────────────────────────────────────────────────────────────

export const updateSubscription = (
  clusterId: string,
  node: string,
  key: string
): Promise<void> =>
  invoke("update_subscription", { clusterId, node, key });
