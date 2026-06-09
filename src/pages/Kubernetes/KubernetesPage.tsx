import React, { useState, useEffect, useCallback, useRef } from "react";
import {
  Layers,
  Network,
  Database,
  Shield,
  Server,
  ChevronDown,
  ChevronRight,
  RefreshCw,
  Plus,
  Package,
  Settings2,
  Box,
  Bell,
  Puzzle,
} from "lucide-react";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { useKubernetesStore } from "@/stores/kubernetesStore";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui";
import {
  PodList,
  DeploymentList,
  DaemonSetList,
  StatefulSetList,
  ReplicaSetList,
  JobList,
  CronJobList,
  ServiceList,
  IngressList,
  ConfigMapList,
  SecretList,
  HPAList,
  PVCList,
  PVList,
  ServiceAccountList,
  RoleList,
  ClusterRoleList,
  RoleBindingList,
  ClusterRoleBindingList,
  NodeList,
  EventList,
  ClusterOverview,
  PortForwardList,
  PortForwardForm,
  CommandPalette,
  Hotbar,
  StorageClassList,
  NetworkPolicyList,
  ResourceQuotaList,
  LimitRangeList,
  ReplicationControllerList,
  PodDisruptionBudgetList,
  PriorityClassList,
  RuntimeClassList,
  LeaseList,
  MutatingWebhookList,
  ValidatingWebhookList,
  EndpointList,
  EndpointSliceList,
  IngressClassList,
  NamespaceList,
  WorkloadOverview,
  CrdList,
} from "@/components/Kubernetes";
import type {
  KubeconfigInfo,
  NamespaceInfo,
  PortForwardResponse,
  PodInfo,
  ServiceInfo,
  DeploymentInfo,
  StatefulSetInfo,
  DaemonSetInfo,
  ReplicaSetInfo,
  JobInfo,
  CronJobInfo,
  ConfigMapInfo,
  SecretInfo,
  NodeInfo,
  EventInfo,
  IngressInfo,
  PersistentVolumeClaimInfo,
  PersistentVolumeInfo,
  ServiceAccountInfo,
  RoleInfo,
  ClusterRoleInfo,
  RoleBindingInfo,
  ClusterRoleBindingInfo,
  HorizontalPodAutoscalerInfo,
  StorageClassInfo,
  NetworkPolicyInfo,
  ResourceQuotaInfo,
  LimitRangeInfo,
  ReplicationControllerInfo,
  PodDisruptionBudgetInfo,
  PriorityClassInfo,
  RuntimeClassInfo,
  LeaseInfo,
  WebhookConfigInfo,
  EndpointInfo,
  EndpointSliceInfo,
  IngressClassInfo,
  NamespaceResourceInfo,
  HelmChart,
  HelmRelease,
  CrdInfo,
} from "@/lib/tauriCommands";
import {
  listKubeconfigsCmd,
  activateKubeconfigCmd,
  connectClusterFromKubeconfigCmd,
  listNamespacesCmd,
  listPortForwardsCmd,
  startPortForwardCmd,
  stopPortForwardCmd,
  deletePortForwardCmd,
  listPodsCmd,
  listServicesCmd,
  listDeploymentsCmd,
  listStatefulsetsCmd,
  listDaemonsetsCmd,
  listReplicasetsCmd,
  listJobsCmd,
  listCronjobsCmd,
  listConfigmapsCmd,
  listSecretsCmd,
  listNodesCmd,
  listEventsCmd,
  listIngressesCmd,
  listPersistentvolumeclaimsCmd,
  listPersistentvolumesCmd,
  listServiceaccountsCmd,
  listRolesCmd,
  listClusterrolesCmd,
  listRolebindingsCmd,
  listClusterrolebindingsCmd,
  listHorizontalpodautoscalersCmd,
  listStorageclassesCmd,
  listNetworkpoliciesCmd,
  listResourcequotasCmd,
  listLimitrangesCmd,
  listReplicationcontrollersCmd,
  listPoddisruptionbudgetsCmd,
  listPriorityclassesCmd,
  listRuntimeclassesCmd,
  listLeasesCmd,
  listMutatingwebhookconfigurationsCmd,
  listValidatingwebhookconfigurationsCmd,
  listEndpointsCmd,
  listEndpointslicesCmd,
  listIngressclassesCmd,
  listNamespacesResourceCmd,
  helmSearchRepoCmd,
  helmListReleasesCmd,
  listCrdsCmd,
} from "@/lib/tauriCommands";

// ─── Types ────────────────────────────────────────────────────────────────────

type ActiveSection =
  | "cluster_overview"
  | "nodes"
  | "workloads_overview"
  | "pods"
  | "deployments"
  | "daemonsets"
  | "statefulsets"
  | "replicasets"
  | "replicationcontrollers"
  | "jobs"
  | "cronjobs"
  | "configmaps"
  | "secrets"
  | "resourcequotas"
  | "limitranges"
  | "hpas"
  | "poddisruptionbudgets"
  | "priorityclasses"
  | "runtimeclasses"
  | "leases"
  | "mutatingwebhooks"
  | "validatingwebhooks"
  | "services"
  | "endpointslices"
  | "endpoints"
  | "ingresses"
  | "ingressclasses"
  | "networkpolicies"
  | "portforwarding"
  | "pvcs"
  | "pvs"
  | "storageclasses"
  | "namespaces"
  | "events"
  | "helm_charts"
  | "helm_releases"
  | "serviceaccounts"
  | "clusterroles"
  | "roles"
  | "clusterrolebindings"
  | "rolebindings"
  | "crds";

interface NavItem {
  id: ActiveSection;
  label: string;
}

interface NavGroup {
  type: "group";
  label: string;
  icon: React.ElementType;
  items: NavItem[];
}

interface NavTopLevel {
  type: "toplevel";
  id: ActiveSection;
  label: string;
  icon: React.ElementType;
}

type NavEntry = NavGroup | NavTopLevel;

// ─── Nav structure ────────────────────────────────────────────────────────────

const NAV_ENTRIES: NavEntry[] = [
  { type: "toplevel", id: "cluster_overview", label: "Cluster", icon: Server },
  { type: "toplevel", id: "nodes", label: "Nodes", icon: Server },
  {
    type: "group",
    label: "Workloads",
    icon: Layers,
    items: [
      { id: "workloads_overview", label: "Overview" },
      { id: "pods", label: "Pods" },
      { id: "deployments", label: "Deployments" },
      { id: "daemonsets", label: "Daemon Sets" },
      { id: "statefulsets", label: "Stateful Sets" },
      { id: "replicasets", label: "Replica Sets" },
      { id: "replicationcontrollers", label: "Replication Controllers" },
      { id: "jobs", label: "Jobs" },
      { id: "cronjobs", label: "Cron Jobs" },
    ],
  },
  {
    type: "group",
    label: "Config",
    icon: Settings2,
    items: [
      { id: "configmaps", label: "Config Maps" },
      { id: "secrets", label: "Secrets" },
      { id: "resourcequotas", label: "Resource Quotas" },
      { id: "limitranges", label: "Limit Ranges" },
      { id: "hpas", label: "Horizontal Pod Autoscalers" },
      { id: "poddisruptionbudgets", label: "Pod Disruption Budgets" },
      { id: "priorityclasses", label: "Priority Classes" },
      { id: "runtimeclasses", label: "Runtime Classes" },
      { id: "leases", label: "Leases" },
      { id: "mutatingwebhooks", label: "Mutating Webhook Configs" },
      { id: "validatingwebhooks", label: "Validating Webhook Configs" },
    ],
  },
  {
    type: "group",
    label: "Network",
    icon: Network,
    items: [
      { id: "services", label: "Services" },
      { id: "endpointslices", label: "Endpoint Slices" },
      { id: "endpoints", label: "Endpoints" },
      { id: "ingresses", label: "Ingresses" },
      { id: "ingressclasses", label: "Ingress Classes" },
      { id: "networkpolicies", label: "Network Policies" },
      { id: "portforwarding", label: "Port Forwarding" },
    ],
  },
  {
    type: "group",
    label: "Storage",
    icon: Database,
    items: [
      { id: "pvcs", label: "Persistent Volume Claims" },
      { id: "pvs", label: "Persistent Volumes" },
      { id: "storageclasses", label: "Storage Classes" },
    ],
  },
  { type: "toplevel", id: "namespaces", label: "Namespaces", icon: Box },
  { type: "toplevel", id: "events", label: "Events", icon: Bell },
  {
    type: "group",
    label: "Helm",
    icon: Package,
    items: [
      { id: "helm_charts", label: "Charts" },
      { id: "helm_releases", label: "Releases" },
    ],
  },
  {
    type: "group",
    label: "Access Control",
    icon: Shield,
    items: [
      { id: "serviceaccounts", label: "Service Accounts" },
      { id: "clusterroles", label: "Cluster Roles" },
      { id: "roles", label: "Roles" },
      { id: "clusterrolebindings", label: "Cluster Role Bindings" },
      { id: "rolebindings", label: "Role Bindings" },
    ],
  },
  {
    type: "group",
    label: "Custom Resources",
    icon: Puzzle,
    items: [
      { id: "crds", label: "Definitions" },
    ],
  },
];

// ─── Resource data union ──────────────────────────────────────────────────────

interface ResourceData {
  pods: PodInfo[];
  services: ServiceInfo[];
  deployments: DeploymentInfo[];
  statefulsets: StatefulSetInfo[];
  daemonsets: DaemonSetInfo[];
  replicasets: ReplicaSetInfo[];
  jobs: JobInfo[];
  cronjobs: CronJobInfo[];
  configmaps: ConfigMapInfo[];
  secrets: SecretInfo[];
  nodes: NodeInfo[];
  events: EventInfo[];
  ingresses: IngressInfo[];
  pvcs: PersistentVolumeClaimInfo[];
  pvs: PersistentVolumeInfo[];
  serviceaccounts: ServiceAccountInfo[];
  roles: RoleInfo[];
  clusterroles: ClusterRoleInfo[];
  rolebindings: RoleBindingInfo[];
  clusterrolebindings: ClusterRoleBindingInfo[];
  hpas: HorizontalPodAutoscalerInfo[];
  storageclasses: StorageClassInfo[];
  networkpolicies: NetworkPolicyInfo[];
  resourcequotas: ResourceQuotaInfo[];
  limitranges: LimitRangeInfo[];
  replicationcontrollers: ReplicationControllerInfo[];
  poddisruptionbudgets: PodDisruptionBudgetInfo[];
  priorityclasses: PriorityClassInfo[];
  runtimeclasses: RuntimeClassInfo[];
  leases: LeaseInfo[];
  mutatingwebhooks: WebhookConfigInfo[];
  validatingwebhooks: WebhookConfigInfo[];
  endpoints: EndpointInfo[];
  endpointslices: EndpointSliceInfo[];
  ingressclasses: IngressClassInfo[];
  namespaces_resource: NamespaceResourceInfo[];
  helm_charts: HelmChart[];
  helm_releases: HelmRelease[];
  crds: CrdInfo[];
}

const EMPTY_RESOURCES: ResourceData = {
  pods: [],
  services: [],
  deployments: [],
  statefulsets: [],
  daemonsets: [],
  replicasets: [],
  jobs: [],
  cronjobs: [],
  configmaps: [],
  secrets: [],
  nodes: [],
  events: [],
  ingresses: [],
  pvcs: [],
  pvs: [],
  serviceaccounts: [],
  roles: [],
  clusterroles: [],
  rolebindings: [],
  clusterrolebindings: [],
  hpas: [],
  storageclasses: [],
  networkpolicies: [],
  resourcequotas: [],
  limitranges: [],
  replicationcontrollers: [],
  poddisruptionbudgets: [],
  priorityclasses: [],
  runtimeclasses: [],
  leases: [],
  mutatingwebhooks: [],
  validatingwebhooks: [],
  endpoints: [],
  endpointslices: [],
  ingressclasses: [],
  namespaces_resource: [],
  helm_charts: [],
  helm_releases: [],
  crds: [],
};

// ─── Component ───────────────────────────────────────────────────────────────

export function KubernetesPage() {
  const { selectedClusterId, selectedNamespace, setSelectedCluster, setSelectedNamespace } =
    useKubernetesStore();

  const [kubeconfigs, setKubeconfigs] = useState<KubeconfigInfo[]>([]);
  const [namespaces, setNamespaces] = useState<NamespaceInfo[]>([]);
  const [portForwards, setPortForwards] = useState<PortForwardResponse[]>([]);
  const [resources, setResources] = useState<ResourceData>(EMPTY_RESOURCES);
  const [activeSection, setActiveSection] = useState<ActiveSection>("cluster_overview");
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    Workloads: true,
    Config: true,
    Network: true,
    Storage: true,
    Helm: false,
    "Access Control": true,
    "Custom Resources": false,
  });
  const [isLoadingResources, setIsLoadingResources] = useState(false);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [isPortForwardFormOpen, setIsPortForwardFormOpen] = useState(false);
  const [isNotificationsOpen, setIsNotificationsOpen] = useState(false);
  const [connectionError, setConnectionError] = useState<string | null>(null);

  const lastLoadedRef = useRef<{ section: ActiveSection; clusterId: string; namespace: string } | null>(null);
  const initializedRef = useRef(false);

  // ── Initial data load ──────────────────────────────────────────────────────

  const loadInitialData = useCallback(async () => {
    try {
      const [kubeconfigsData, portForwardsData] = await Promise.all([
        listKubeconfigsCmd(),
        listPortForwardsCmd(),
      ]);
      setKubeconfigs(kubeconfigsData);
      setPortForwards(portForwardsData);

      if (!initializedRef.current) {
        initializedRef.current = true;
        const activeConfig = kubeconfigsData.find((c) => c.is_active);
        const targetId = selectedClusterId ?? activeConfig?.id;
        if (targetId) {
          const err = await connectClusterFromKubeconfigCmd(targetId)
            .then(() => null)
            .catch((e: unknown) => e);
          if (err) {
            setConnectionError(err instanceof Error ? err.message : String(err));
          } else {
            setSelectedCluster(targetId);
          }
        }
      }
    } catch (err) {
      console.error("Failed to load initial Kubernetes data:", err);
    }
  }, [selectedClusterId, setSelectedCluster]);

  useEffect(() => {
    loadInitialData();
  }, [loadInitialData]);

  // ── Load namespaces when cluster changes ──────────────────────────────────

  useEffect(() => {
    if (!selectedClusterId) return;

    listNamespacesCmd(selectedClusterId)
      .then(setNamespaces)
      .catch((err) => console.error("Failed to load namespaces:", err));
  }, [selectedClusterId]);

  // ── Load resource data when section, cluster, or namespace changes ─────────

  const loadResourceData = useCallback(
    async (section: ActiveSection, clusterId: string, namespace: string) => {
      if (section === "cluster_overview" || section === "portforwarding") {
        return;
      }

      const ns = namespace === "all" ? "" : namespace;

      setIsLoadingResources(true);
      try {
        switch (section) {
          case "workloads_overview": {
            const [pods, deployments, statefulsets, daemonsets, jobs, cronjobs] =
              await Promise.allSettled([
                listPodsCmd(clusterId, ns),
                listDeploymentsCmd(clusterId, ns),
                listStatefulsetsCmd(clusterId, ns),
                listDaemonsetsCmd(clusterId, ns),
                listJobsCmd(clusterId, ns),
                listCronjobsCmd(clusterId, ns),
              ]).then((results) =>
                results.map((r) => (r.status === "fulfilled" ? r.value : []))
              );
            setResources((r) => ({
              ...r,
              pods: pods as PodInfo[],
              deployments: deployments as DeploymentInfo[],
              statefulsets: statefulsets as StatefulSetInfo[],
              daemonsets: daemonsets as DaemonSetInfo[],
              jobs: jobs as JobInfo[],
              cronjobs: cronjobs as CronJobInfo[],
            }));
            break;
          }
          case "pods":
            await listPodsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, pods: data }))
            );
            break;
          case "deployments":
            await listDeploymentsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, deployments: data }))
            );
            break;
          case "daemonsets":
            await listDaemonsetsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, daemonsets: data }))
            );
            break;
          case "statefulsets":
            await listStatefulsetsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, statefulsets: data }))
            );
            break;
          case "replicasets":
            await listReplicasetsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, replicasets: data }))
            );
            break;
          case "replicationcontrollers":
            await listReplicationcontrollersCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, replicationcontrollers: data }))
            );
            break;
          case "jobs":
            await listJobsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, jobs: data }))
            );
            break;
          case "cronjobs":
            await listCronjobsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, cronjobs: data }))
            );
            break;
          case "services":
            await listServicesCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, services: data }))
            );
            break;
          case "ingresses":
            await listIngressesCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, ingresses: data }))
            );
            break;
          case "configmaps":
            await listConfigmapsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, configmaps: data }))
            );
            break;
          case "secrets":
            await listSecretsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, secrets: data }))
            );
            break;
          case "hpas":
            await listHorizontalpodautoscalersCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, hpas: data }))
            );
            break;
          case "pvcs":
            await listPersistentvolumeclaimsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, pvcs: data }))
            );
            break;
          case "pvs":
            await listPersistentvolumesCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, pvs: data }))
            );
            break;
          case "serviceaccounts":
            await listServiceaccountsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, serviceaccounts: data }))
            );
            break;
          case "roles":
            await listRolesCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, roles: data }))
            );
            break;
          case "clusterroles":
            await listClusterrolesCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, clusterroles: data }))
            );
            break;
          case "rolebindings":
            await listRolebindingsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, rolebindings: data }))
            );
            break;
          case "clusterrolebindings":
            await listClusterrolebindingsCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, clusterrolebindings: data }))
            );
            break;
          case "nodes":
            await listNodesCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, nodes: data }))
            );
            break;
          case "events":
            await listEventsCmd(clusterId, ns || undefined).then((data) =>
              setResources((r) => ({ ...r, events: data }))
            );
            break;
          case "storageclasses":
            await listStorageclassesCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, storageclasses: data }))
            );
            break;
          case "networkpolicies":
            await listNetworkpoliciesCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, networkpolicies: data }))
            );
            break;
          case "resourcequotas":
            await listResourcequotasCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, resourcequotas: data }))
            );
            break;
          case "limitranges":
            await listLimitrangesCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, limitranges: data }))
            );
            break;
          case "poddisruptionbudgets":
            await listPoddisruptionbudgetsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, poddisruptionbudgets: data }))
            );
            break;
          case "priorityclasses":
            await listPriorityclassesCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, priorityclasses: data }))
            );
            break;
          case "runtimeclasses":
            await listRuntimeclassesCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, runtimeclasses: data }))
            );
            break;
          case "leases":
            await listLeasesCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, leases: data }))
            );
            break;
          case "mutatingwebhooks":
            await listMutatingwebhookconfigurationsCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, mutatingwebhooks: data }))
            );
            break;
          case "validatingwebhooks":
            await listValidatingwebhookconfigurationsCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, validatingwebhooks: data }))
            );
            break;
          case "endpoints":
            await listEndpointsCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, endpoints: data }))
            );
            break;
          case "endpointslices":
            await listEndpointslicesCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, endpointslices: data }))
            );
            break;
          case "ingressclasses":
            await listIngressclassesCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, ingressclasses: data }))
            );
            break;
          case "namespaces":
            await listNamespacesResourceCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, namespaces_resource: data }))
            );
            break;
          case "helm_charts":
            await helmSearchRepoCmd(clusterId, "").then((data) =>
              setResources((r) => ({ ...r, helm_charts: data }))
            );
            break;
          case "helm_releases":
            await helmListReleasesCmd(clusterId, ns).then((data) =>
              setResources((r) => ({ ...r, helm_releases: data }))
            );
            break;
          case "crds":
            await listCrdsCmd(clusterId).then((data) =>
              setResources((r) => ({ ...r, crds: data }))
            );
            break;
        }
        lastLoadedRef.current = { section, clusterId, namespace };
      } catch (err) {
        console.error(`Failed to load ${section}:`, err);
      } finally {
        setIsLoadingResources(false);
      }
    },
    []
  );

  // Reset resources when activeSection changes to prevent stale data accumulation
  useEffect(() => {
    setResources(EMPTY_RESOURCES);
  }, [activeSection]);

  useEffect(() => {
    if (!selectedClusterId) return;
    loadResourceData(activeSection, selectedClusterId, selectedNamespace);
  }, [activeSection, selectedClusterId, selectedNamespace, loadResourceData]);

  // ── Keyboard shortcut for CommandPalette ──────────────────────────────────

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === "k") {
        e.preventDefault();
        setIsCommandPaletteOpen((prev) => !prev);
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, []);

  // ── Handlers ─────────────────────────────────────────────────────────────

  const handleClusterChange = async (id: string) => {
    try {
      await activateKubeconfigCmd(id);
      await connectClusterFromKubeconfigCmd(id);
      const updated = await listKubeconfigsCmd();
      setKubeconfigs(updated);
      const active = updated.find((c) => c.is_active);
      if (active) {
        setSelectedCluster(active.id);
      }
    } catch (err) {
      console.error("Failed to activate kubeconfig:", err);
    }
  };

  const handleRefresh = () => {
    if (!selectedClusterId) return;
    lastLoadedRef.current = null;
    if (activeSection === "portforwarding") {
      listPortForwardsCmd()
        .then(setPortForwards)
        .catch((err) => console.error("Failed to refresh port forwards:", err));
      return;
    }
    loadResourceData(activeSection, selectedClusterId, selectedNamespace);
  };

  const handleStopPortForward = async (id: string) => {
    try {
      await stopPortForwardCmd(id);
      setPortForwards((prev) => prev.filter((pf) => pf.id !== id));
    } catch (err) {
      console.error("Failed to stop port forward:", err);
    }
  };

  const handleDeletePortForward = async (id: string) => {
    try {
      await deletePortForwardCmd(id);
      setPortForwards((prev) => prev.filter((pf) => pf.id !== id));
    } catch (err) {
      console.error("Failed to delete port forward:", err);
    }
  };

  const handleStartPortForward = async (portForward: Parameters<typeof startPortForwardCmd>[0]) => {
    try {
      const result = await startPortForwardCmd(portForward);
      setPortForwards((prev) => [...prev, result]);
    } catch (err) {
      console.error("Failed to start port forward:", err);
    }
  };

  const toggleSection = (label: string) => {
    setExpandedSections((prev) => ({ ...prev, [label]: !prev[label] }));
  };

  const handleNavigate = (section: string) => {
    setActiveSection(section as ActiveSection);
  };

  // ── Content renderer ──────────────────────────────────────────────────────

  const renderContent = () => {
    if (!selectedClusterId) {
      return (
        <div className="flex flex-col items-center justify-center h-full gap-4 text-center px-8">
          <Package className="w-16 h-16 text-muted-foreground" />
          <h2 className="text-2xl font-semibold">No cluster selected</h2>
          <p className="text-muted-foreground max-w-sm">
            Select a cluster from the dropdown above, or upload a kubeconfig file
            in Settings → Kubeconfig to get started.
          </p>
        </div>
      );
    }

    if (activeSection === "cluster_overview") {
      return (
        <ClusterOverview
          clusterId={selectedClusterId}
          clusterName={selectedConfig?.name}
        />
      );
    }

    if (activeSection === "workloads_overview") {
      return (
        <WorkloadOverview
          clusterId={selectedClusterId}
          resources={{
            pods: resources.pods,
            deployments: resources.deployments,
            statefulsets: resources.statefulsets,
            daemonsets: resources.daemonsets,
            jobs: resources.jobs,
            cronjobs: resources.cronjobs,
          }}
        />
      );
    }

    if (activeSection === "portforwarding") {
      return (
        <div className="p-6 space-y-4">
          <PortForwardList
            portForwards={portForwards}
            onStart={() => setIsPortForwardFormOpen(true)}
            onStop={handleStopPortForward}
            onDelete={handleDeletePortForward}
          />
          <PortForwardForm
            isOpen={isPortForwardFormOpen}
            onClose={() => setIsPortForwardFormOpen(false)}
            onStart={(pf) => {
              setPortForwards((prev) => [...prev, pf]);
              setIsPortForwardFormOpen(false);
            }}
          />
        </div>
      );
    }

    if (isLoadingResources) {
      return (
        <div className="flex items-center justify-center h-full">
          <div className="flex flex-col items-center gap-4">
            <RefreshCw className="w-8 h-8 animate-spin text-primary" />
            <p className="text-muted-foreground">Loading resources...</p>
          </div>
        </div>
      );
    }

    const ns = selectedNamespace;
    const cid = selectedClusterId;

    switch (activeSection) {
      case "pods":
        return <PodList pods={resources.pods} clusterId={cid} namespace={ns} onRefresh={handleRefresh} />;
      case "deployments":
        return <DeploymentList deployments={resources.deployments} clusterId={cid} namespace={ns} />;
      case "daemonsets":
        return <DaemonSetList daemonsets={resources.daemonsets} clusterId={cid} namespace={ns} />;
      case "statefulsets":
        return <StatefulSetList statefulsets={resources.statefulsets} clusterId={cid} namespace={ns} />;
      case "replicasets":
        return <ReplicaSetList replicaSets={resources.replicasets} clusterId={cid} namespace={ns} />;
      case "replicationcontrollers":
        return <ReplicationControllerList items={resources.replicationcontrollers} clusterId={cid} namespace={ns} />;
      case "jobs":
        return <JobList jobs={resources.jobs} clusterId={cid} namespace={ns} />;
      case "cronjobs":
        return <CronJobList cronJobs={resources.cronjobs} clusterId={cid} namespace={ns} />;
      case "services":
        return <ServiceList services={resources.services} clusterId={cid} namespace={ns} />;
      case "ingresses":
        return <IngressList ingresses={resources.ingresses} clusterId={cid} namespace={ns} />;
      case "configmaps":
        return <ConfigMapList configmaps={resources.configmaps} clusterId={cid} namespace={ns} onRefresh={handleRefresh} />;
      case "secrets":
        return <SecretList secrets={resources.secrets} clusterId={cid} namespace={ns} onRefresh={handleRefresh} />;
      case "hpas":
        return <HPAList hpas={resources.hpas} clusterId={cid} namespace={ns} onRefresh={handleRefresh} />;
      case "pvcs":
        return <PVCList pvcs={resources.pvcs} clusterId={cid} namespace={ns} />;
      case "pvs":
        return <PVList pvs={resources.pvs} clusterId={cid} />;
      case "serviceaccounts":
        return <ServiceAccountList serviceAccounts={resources.serviceaccounts} clusterId={cid} namespace={ns} />;
      case "roles":
        return <RoleList roles={resources.roles} clusterId={cid} namespace={ns} />;
      case "clusterroles":
        return <ClusterRoleList clusterRoles={resources.clusterroles} clusterId={cid} />;
      case "rolebindings":
        return <RoleBindingList roleBindings={resources.rolebindings} clusterId={cid} namespace={ns} />;
      case "clusterrolebindings":
        return <ClusterRoleBindingList clusterRoleBindings={resources.clusterrolebindings} clusterId={cid} />;
      case "nodes":
        return <NodeList nodes={resources.nodes} clusterId={cid} />;
      case "events":
        return <EventList events={resources.events} clusterId={cid} namespace={ns} />;
      case "storageclasses":
        return <StorageClassList storageclasses={resources.storageclasses} clusterId={cid} namespace={ns} />;
      case "networkpolicies":
        return <NetworkPolicyList networkpolicies={resources.networkpolicies} clusterId={cid} namespace={ns} />;
      case "resourcequotas":
        return <ResourceQuotaList resourcequotas={resources.resourcequotas} clusterId={cid} namespace={ns} onRefresh={handleRefresh} />;
      case "limitranges":
        return <LimitRangeList limitranges={resources.limitranges} clusterId={cid} namespace={ns} onRefresh={handleRefresh} />;
      case "poddisruptionbudgets":
        return <PodDisruptionBudgetList items={resources.poddisruptionbudgets} clusterId={cid} namespace={ns} onRefresh={handleRefresh} />;
      case "priorityclasses":
        return <PriorityClassList items={resources.priorityclasses} clusterId={cid} onRefresh={handleRefresh} />;
      case "runtimeclasses":
        return <RuntimeClassList items={resources.runtimeclasses} clusterId={cid} onRefresh={handleRefresh} />;
      case "leases":
        return <LeaseList items={resources.leases} clusterId={cid} namespace={ns} onRefresh={handleRefresh} />;
      case "mutatingwebhooks":
        return <MutatingWebhookList items={resources.mutatingwebhooks} clusterId={cid} onRefresh={handleRefresh} />;
      case "validatingwebhooks":
        return <ValidatingWebhookList items={resources.validatingwebhooks} clusterId={cid} onRefresh={handleRefresh} />;
      case "endpoints":
        return <EndpointList items={resources.endpoints} clusterId={cid} namespace={ns} />;
      case "endpointslices":
        return <EndpointSliceList items={resources.endpointslices} clusterId={cid} namespace={ns} />;
      case "ingressclasses":
        return <IngressClassList items={resources.ingressclasses} clusterId={cid} />;
      case "namespaces":
        return <NamespaceList items={resources.namespaces_resource} clusterId={cid} />;
      case "helm_charts":
        return (
          <div className="p-6">
            <h2 className="text-xl font-semibold mb-4">Helm Charts</h2>
            {resources.helm_charts.length === 0 ? (
              <p className="text-muted-foreground">No charts found. Add a Helm repository to browse charts.</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm border-collapse">
                  <thead>
                    <tr className="border-b text-muted-foreground text-left">
                      <th className="px-4 py-3 font-medium">Name</th>
                      <th className="px-4 py-3 font-medium">Repository</th>
                      <th className="px-4 py-3 font-medium">Chart Version</th>
                      <th className="px-4 py-3 font-medium">App Version</th>
                      <th className="px-4 py-3 font-medium">Description</th>
                    </tr>
                  </thead>
                  <tbody>
                    {resources.helm_charts.map((chart) => (
                      <tr key={`${chart.repository}-${chart.name}`} className="border-b hover:bg-muted/30 transition-colors">
                        <td className="px-4 py-3 font-medium">{chart.name}</td>
                        <td className="px-4 py-3 text-muted-foreground">{chart.repository}</td>
                        <td className="px-4 py-3 font-mono text-xs">{chart.chart_version}</td>
                        <td className="px-4 py-3 font-mono text-xs">{chart.app_version}</td>
                        <td className="px-4 py-3 text-muted-foreground truncate max-w-xs">{chart.description}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        );
      case "helm_releases":
        return (
          <div className="p-6">
            <h2 className="text-xl font-semibold mb-4">Helm Releases</h2>
            {resources.helm_releases.length === 0 ? (
              <p className="text-muted-foreground">No Helm releases found in this namespace.</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm border-collapse">
                  <thead>
                    <tr className="border-b text-muted-foreground text-left">
                      <th className="px-4 py-3 font-medium">Name</th>
                      <th className="px-4 py-3 font-medium">Namespace</th>
                      <th className="px-4 py-3 font-medium">Chart</th>
                      <th className="px-4 py-3 font-medium">App Version</th>
                      <th className="px-4 py-3 font-medium">Status</th>
                      <th className="px-4 py-3 font-medium">Updated</th>
                    </tr>
                  </thead>
                  <tbody>
                    {resources.helm_releases.map((rel) => (
                      <tr key={`${rel.namespace}-${rel.name}`} className="border-b hover:bg-muted/30 transition-colors">
                        <td className="px-4 py-3 font-medium">{rel.name}</td>
                        <td className="px-4 py-3 text-muted-foreground">{rel.namespace}</td>
                        <td className="px-4 py-3 font-mono text-xs">{rel.chart} {rel.chart_version}</td>
                        <td className="px-4 py-3 font-mono text-xs">{rel.app_version}</td>
                        <td className="px-4 py-3">
                          <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                            rel.status === "deployed"
                              ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                              : rel.status === "failed"
                              ? "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
                              : "bg-muted text-muted-foreground"
                          }`}>
                            {rel.status}
                          </span>
                        </td>
                        <td className="px-4 py-3 text-muted-foreground text-xs">{rel.updated}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        );
      case "crds":
        return (
          <div className="p-6">
            <CrdList clusterId={cid} />
          </div>
        );
      default:
        return null;
    }
  };

  // ── Render ────────────────────────────────────────────────────────────────

  const selectedConfig = kubeconfigs.find((c) => c.id === selectedClusterId);

  return (
    <ErrorBoundary>
    <div className="flex flex-col h-full bg-background">
      {/* Hotbar */}
      <Hotbar
        onRefresh={handleRefresh}
        onAddResource={() => setIsCommandPaletteOpen(true)}
        onSettings={() => {}}
        onNotifications={() => setIsNotificationsOpen(true)}
        clusterName={selectedConfig?.name}
      />

      {/* Top bar: cluster selector + namespace selector */}
      <div className="flex items-center gap-4 px-4 py-2 border-b bg-card">
        <div className="flex items-center gap-2">
          <Server className="w-4 h-4 text-muted-foreground shrink-0" />
          <Select
            value={selectedClusterId ?? ""}
            onValueChange={handleClusterChange}
          >
            <SelectTrigger className="w-52 h-8 text-sm">
              <SelectValue placeholder="Select cluster" />
            </SelectTrigger>
            <SelectContent>
              {kubeconfigs.length === 0 ? (
                <SelectItem value="__none__">No kubeconfigs available</SelectItem>
              ) : (
                kubeconfigs.map((kc) => (
                  <SelectItem key={kc.id} value={kc.id}>
                    {kc.name}
                  </SelectItem>
                ))
              )}
            </SelectContent>
          </Select>
        </div>

        {selectedClusterId && (
          <>
            <div className="h-4 w-px bg-border" />
            <div className="flex items-center gap-2">
              <span className="text-xs text-muted-foreground">Namespace:</span>
              <Select
                value={selectedNamespace}
                onValueChange={setSelectedNamespace}
              >
                <SelectTrigger className="w-44 h-8 text-sm">
                  <SelectValue placeholder="All namespaces" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Namespaces</SelectItem>
                  {namespaces.map((ns) => (
                    <SelectItem key={ns.name} value={ns.name}>
                      {ns.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </>
        )}

        {selectedConfig && (
          <div className="ml-auto flex items-center gap-2 text-xs text-muted-foreground">
            <span className="font-medium">Context:</span>
            <span>{selectedConfig.context}</span>
            {selectedConfig.cluster_url && (
              <>
                <span className="text-border">|</span>
                <span className="font-mono truncate max-w-48">{selectedConfig.cluster_url}</span>
              </>
            )}
          </div>
        )}
      </div>

      {connectionError && (
        <div className="flex items-center gap-2 px-4 py-2 bg-destructive/10 border-b border-destructive/20 text-destructive text-sm">
          <span>Cluster connection failed: {connectionError}</span>
          <button className="ml-auto underline" onClick={() => setConnectionError(null)}>Dismiss</button>
        </div>
      )}

      {/* Main layout: sidebar + content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <aside className="w-56 shrink-0 border-r bg-card overflow-y-auto flex flex-col">
          {NAV_ENTRIES.map((entry) => {
            if (entry.type === "toplevel") {
              const Icon = entry.icon;
              return (
                <button
                  key={entry.id}
                  onClick={() => setActiveSection(entry.id)}
                  aria-label={entry.label}
                  className={`flex items-center gap-2 w-full px-3 py-2 text-xs font-semibold uppercase tracking-wider transition-colors ${
                    activeSection === entry.id
                      ? "bg-primary/10 text-primary border-l-2 border-primary"
                      : "text-muted-foreground hover:text-foreground hover:bg-accent"
                  }`}
                >
                  <Icon className="w-3.5 h-3.5" />
                  <span>{entry.label}</span>
                </button>
              );
            }

            const isExpanded = expandedSections[entry.label] ?? true;
            const Icon = entry.icon;

            return (
              <div key={entry.label}>
                <button
                  onClick={() => toggleSection(entry.label)}
                  className="flex items-center justify-between w-full px-3 py-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                >
                  <div className="flex items-center gap-2">
                    <Icon className="w-3.5 h-3.5" />
                    <span>{entry.label}</span>
                  </div>
                  {isExpanded ? (
                    <ChevronDown className="w-3 h-3" />
                  ) : (
                    <ChevronRight className="w-3 h-3" />
                  )}
                </button>

                {isExpanded && (
                  <div className="pb-1">
                    {entry.items.map((item) => (
                      <button
                        key={item.id}
                        onClick={() => setActiveSection(item.id)}
                        aria-label={item.label}
                        className={`flex items-center w-full px-5 py-1.5 text-sm transition-colors ${
                          activeSection === item.id
                            ? "bg-primary/10 text-primary font-medium border-l-2 border-primary"
                            : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                        }`}
                      >
                        {item.label}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            );
          })}

          {/* Add resource shortcut at bottom of sidebar */}
          <div className="mt-auto border-t p-3">
            <button
              onClick={() => setIsCommandPaletteOpen(true)}
              className="flex items-center gap-2 w-full px-3 py-2 text-xs text-muted-foreground hover:text-foreground hover:bg-accent rounded-md transition-colors"
            >
              <Plus className="w-3.5 h-3.5" />
              <span>Command Palette</span>
              <kbd className="ml-auto text-[10px] bg-muted border rounded px-1 py-0.5">⌃K</kbd>
            </button>
          </div>
        </aside>

        {/* Main content */}
        <main className="flex-1 overflow-y-auto bg-background">
          {renderContent()}
        </main>
      </div>

      {/* Command Palette */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
        onNavigate={handleNavigate}
      />

      {/* Notifications panel */}
      <Dialog open={isNotificationsOpen} onOpenChange={setIsNotificationsOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Notifications</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-2">
            {selectedConfig ? (
              <div className="text-sm">
                <p className="font-medium mb-1">Active cluster</p>
                <p className="text-muted-foreground">{selectedConfig.context}</p>
                {selectedConfig.cluster_url && (
                  <p className="font-mono text-xs text-muted-foreground mt-0.5 truncate">
                    {selectedConfig.cluster_url}
                  </p>
                )}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">No cluster connected.</p>
            )}
            <p className="text-xs text-muted-foreground pt-2 border-t">
              Navigate to <strong>Events</strong> to view live cluster events.
            </p>
          </div>
        </DialogContent>
      </Dialog>

      {/* Port Forward Form (only rendered outside portforwarding section via global trigger) */}
      {activeSection !== "portforwarding" && (
        <PortForwardForm
          isOpen={isPortForwardFormOpen}
          onClose={() => setIsPortForwardFormOpen(false)}
          onStart={(pf) => {
            void handleStartPortForward({
              cluster_id: pf.cluster_id,
              namespace: pf.namespace,
              pod: pf.pod,
              container_port: pf.container_ports[0] ?? 80,
            });
            setIsPortForwardFormOpen(false);
          }}
        />
      )}
    </div>
    </ErrorBoundary>
  );
}
