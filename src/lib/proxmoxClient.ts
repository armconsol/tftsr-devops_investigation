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

export async function listProxmoxNodes(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_proxmox_nodes", { clusterId });
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
  clusterId: string,
  nodeId: string
): Promise<any[]> {
  return await invoke<any[]>("list_proxmox_backup_jobs", { clusterId, nodeId });
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
 * Trigger Proxmox Backup Job
 * @param clusterId - Cluster identifier
 * @param nodeId - Node identifier
 * @param jobId - Backup job identifier
 */
export async function triggerProxmoxBackupJob(
  clusterId: string,
  nodeId: string,
  jobId: number
): Promise<void> {
  await invoke("trigger_proxmox_backup_job", { clusterId, nodeId, jobId });
}

/**
 * List Ceph Pools
 * @param clusterId - Cluster identifier
 */
export async function listCephPools(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_ceph_pools", { clusterId });
}

/**
 * List Ceph OSDs
 * @param clusterId - Cluster identifier
 */
export async function listCephOsd(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_ceph_osd", { clusterId });
}

/**
 * Get Ceph Health
 * @param clusterId - Cluster identifier
 */
export async function getCephHealth(clusterId: string): Promise<any> {
  return await invoke<any>("get_ceph_health", { clusterId });
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
  return await invoke<any[]>("list_apt_updates", { clusterId, nodeId });
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
  await invoke("update_apt_repos", { clusterId, nodeId });
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
  return await invoke<any[]>("list_apt_repositories", { clusterId, nodeId });
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

// ─── Dashboard Views ──────────────────────────────────────────────────────────

/**
 * List dashboard views
 * @param clusterId - Cluster identifier
 */
export async function listViews(clusterId: string): Promise<any[]> {
  return await invoke<any[]>("list_views", { clusterId });
}

/**
 * Add a dashboard view
 * @param clusterId - Cluster identifier
 * @param view - View configuration
 */
export async function addView(clusterId: string, view: any): Promise<void> {
  await invoke("add_view", { clusterId, view });
}

/**
 * Update a dashboard view
 * @param clusterId - Cluster identifier
 * @param viewId - View identifier
 * @param view - View configuration
 */
export async function updateView(
  clusterId: string,
  viewId: string,
  view: any
): Promise<void> {
  await invoke("update_view", { clusterId, viewId, view });
}

/**
 * Delete a dashboard view
 * @param clusterId - Cluster identifier
 * @param viewId - View identifier
 */
export async function deleteView(
  clusterId: string,
  viewId: string
): Promise<void> {
  await invoke("delete_view", { clusterId, viewId });
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
  targetClusterId: string,
  online: boolean
): Promise<void> {
  await invoke("migrate_vm", {
    clusterId,
    nodeId,
    vmId,
    targetClusterId,
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
): Promise<any> {
  return await invoke<any>("get_task_status", { clusterId, nodeId, taskId });
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
 * @param config - HA group configuration
 */
export const createHaGroup = async (
  clusterId: string,
  config: Partial<HaGroup>
): Promise<void> => invoke<void>("create_ha_group", { clusterId, config });

/**
 * Update an HA group
 * @param clusterId - Cluster identifier
 * @param id - HA group identifier
 * @param config - HA group configuration
 */
export const updateHaGroup = async (
  clusterId: string,
  id: string,
  config: Partial<HaGroup>
): Promise<void> => invoke<void>("update_ha_group", { clusterId, id, config });

/**
 * Delete an HA group
 * @param clusterId - Cluster identifier
 * @param id - HA group identifier
 */
export const deleteHaGroup = async (
  clusterId: string,
  id: string
): Promise<void> => invoke<void>("delete_ha_group", { clusterId, id });

/**
 * List HA resources
 * @param clusterId - Cluster identifier
 */
export const listHaResources = async (
  clusterId: string
): Promise<HaResource[]> =>
  invoke<HaResource[]>("list_ha_resources", { clusterId });

/**
 * Enable an HA resource
 * @param clusterId - Cluster identifier
 * @param id - HA resource identifier
 */
export const enableHaResource = async (
  clusterId: string,
  id: string
): Promise<void> => invoke<void>("enable_ha_resource", { clusterId, id });

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

// ─── Cluster Views (typed) ────────────────────────────────────────────────────

export interface ClusterView {
  view_id: string;
  name: string;
  description?: string;
  layout?: string;
  enabled?: boolean;
}

/**
 * List cluster views
 * @param clusterId - Cluster identifier
 */
export const listClusterViews = async (
  clusterId: string
): Promise<ClusterView[]> =>
  invoke<ClusterView[]>("list_cluster_views", { clusterId });

/**
 * Create a cluster view
 * @param clusterId - Cluster identifier
 * @param viewId - View identifier
 * @param name - View display name
 */
export const createClusterView = async (
  clusterId: string,
  viewId: string,
  name: string
): Promise<void> =>
  invoke<void>("create_cluster_view", { clusterId, viewId, name });

/**
 * Delete a cluster view
 * @param clusterId - Cluster identifier
 * @param viewId - View identifier
 */
export const deleteClusterView = async (
  clusterId: string,
  viewId: string
): Promise<void> => invoke<void>("delete_cluster_view", { clusterId, viewId });

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
