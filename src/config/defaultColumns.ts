import type { ColumnConfig } from "@/hooks/useColumnConfig";

/**
 * Default column visibility configuration for each resource type
 * Based on FreeLens patterns: commonly used columns visible by default,
 * detailed/technical columns hidden by default
 */

export const DEFAULT_COLUMNS: Record<string, ColumnConfig> = {
  // Workloads
  pods: {
    name: true,
    namespace: true,
    ready: true,
    status: true,
    restarts: true,
    age: true,
    ip: false, // Hidden by default - too detailed
    node: false, // Hidden by default - too detailed
    qos: false, // Hidden by default - rarely needed
    cpu: false, // Hidden by default - metrics optional
    memory: false, // Hidden by default - metrics optional
    actions: true,
  },

  deployments: {
    name: true,
    namespace: true,
    ready: true,
    upToDate: true,
    available: true,
    age: true,
    conditions: false, // Hidden by default - verbose
    images: false, // Hidden by default - too detailed
    actions: true,
  },

  statefulsets: {
    name: true,
    namespace: true,
    ready: true,
    replicas: true,
    age: true,
    actions: true,
  },

  daemonsets: {
    name: true,
    namespace: true,
    desired: true,
    current: true,
    ready: true,
    upToDate: true,
    available: true,
    age: true,
    actions: true,
  },

  jobs: {
    name: true,
    namespace: true,
    completions: true,
    duration: true,
    age: true,
    labels: false, // Hidden by default - verbose
    actions: true,
  },

  cronjobs: {
    name: true,
    namespace: true,
    schedule: true,
    active: true,
    lastSchedule: true,
    age: true,
    timezone: false, // Hidden by default - rarely set
    labels: false, // Hidden by default - verbose
    actions: true,
  },

  replicasets: {
    name: true,
    namespace: true,
    desired: true,
    current: true,
    ready: true,
    age: true,
    labels: false, // Hidden by default - verbose
    actions: true,
  },

  replicationcontrollers: {
    name: true,
    namespace: true,
    desired: true,
    current: true,
    ready: true,
    age: true,
    actions: true,
  },

  // Network
  services: {
    name: true,
    namespace: true,
    type: true,
    clusterIP: true,
    externalIP: true,
    ports: true,
    age: true,
    selector: false, // Hidden by default - too detailed
    actions: true,
  },

  ingresses: {
    name: true,
    namespace: true,
    hosts: true,
    addresses: true,
    ports: true,
    age: true,
    rules: false, // Hidden by default - verbose
    tls: false, // Hidden by default - technical
    actions: true,
  },

  networkpolicies: {
    name: true,
    namespace: true,
    podSelector: true,
    age: true,
    policyTypes: false, // Hidden by default - technical
    actions: true,
  },

  endpoints: {
    name: true,
    namespace: true,
    endpoints: true,
    age: true,
    actions: true,
  },

  endpointslices: {
    name: true,
    namespace: true,
    addressType: true,
    endpoints: true,
    age: true,
    ports: false, // Hidden by default - verbose
    actions: true,
  },

  ingressclasses: {
    name: true,
    controller: true,
    age: true,
    parameters: false, // Hidden by default - rarely used
    actions: true,
  },

  // Config
  configmaps: {
    name: true,
    namespace: true,
    data: true,
    age: true,
    actions: true,
  },

  secrets: {
    name: true,
    namespace: true,
    type: true,
    data: true,
    age: true,
    actions: true,
  },

  resourcequotas: {
    name: true,
    namespace: true,
    age: true,
    scopes: false, // Hidden by default - technical
    actions: true,
  },

  limitranges: {
    name: true,
    namespace: true,
    age: true,
    actions: true,
  },

  horizontalpodautoscalers: {
    name: true,
    namespace: true,
    reference: true,
    minPods: true,
    maxPods: true,
    replicas: true,
    age: true,
    targets: false, // Hidden by default - verbose
    actions: true,
  },

  poddisruptionbudgets: {
    name: true,
    namespace: true,
    minAvailable: true,
    maxUnavailable: true,
    age: true,
    allowedDisruptions: false, // Hidden by default - calculated
    actions: true,
  },

  priorityclasses: {
    name: true,
    value: true,
    globalDefault: true,
    age: true,
    description: false, // Hidden by default - verbose
    actions: true,
  },

  runtimeclasses: {
    name: true,
    handler: true,
    age: true,
    actions: true,
  },

  leases: {
    name: true,
    namespace: true,
    holder: true,
    age: true,
    actions: true,
  },

  mutatingwebhookconfigurations: {
    name: true,
    webhooks: true,
    age: true,
    actions: true,
  },

  validatingwebhookconfigurations: {
    name: true,
    webhooks: true,
    age: true,
    actions: true,
  },

  // Storage
  persistentvolumes: {
    name: true,
    capacity: true,
    accessModes: true,
    reclaimPolicy: true,
    status: true,
    claim: true,
    storageClass: true,
    age: true,
    volumeMode: false, // Hidden by default - rarely changed
    actions: true,
  },

  persistentvolumeclaims: {
    name: true,
    namespace: true,
    status: true,
    volume: true,
    capacity: true,
    accessModes: true,
    storageClass: true,
    age: true,
    volumeMode: false, // Hidden by default - rarely changed
    actions: true,
  },

  storageclasses: {
    name: true,
    provisioner: true,
    reclaimPolicy: true,
    volumeBindingMode: true,
    age: true,
    allowVolumeExpansion: false, // Hidden by default - technical
    parameters: false, // Hidden by default - verbose
    actions: true,
  },

  // RBAC
  serviceaccounts: {
    name: true,
    namespace: true,
    secrets: true,
    age: true,
    actions: true,
  },

  roles: {
    name: true,
    namespace: true,
    age: true,
    actions: true,
  },

  clusterroles: {
    name: true,
    age: true,
    aggregationRule: false, // Hidden by default - technical
    actions: true,
  },

  rolebindings: {
    name: true,
    namespace: true,
    role: true,
    age: true,
    subjects: false, // Hidden by default - verbose
    actions: true,
  },

  clusterrolebindings: {
    name: true,
    role: true,
    age: true,
    subjects: false, // Hidden by default - verbose
    actions: true,
  },

  // Cluster
  nodes: {
    name: true,
    status: true,
    roles: true,
    age: true,
    version: true,
    internalIP: false, // Hidden by default - technical
    externalIP: false, // Hidden by default - technical
    osImage: false, // Hidden by default - verbose
    kernelVersion: false, // Hidden by default - verbose
    containerRuntime: false, // Hidden by default - technical
    cpu: false, // Hidden by default - metrics optional
    memory: false, // Hidden by default - metrics optional
    actions: true,
  },

  namespaces: {
    name: true,
    status: true,
    age: true,
    labels: false, // Hidden by default - verbose
    actions: true,
  },

  events: {
    namespace: true,
    lastSeen: true,
    type: true,
    reason: true,
    object: true,
    message: true,
    source: false, // Hidden by default - verbose
    count: false, // Hidden by default - technical
  },
};
