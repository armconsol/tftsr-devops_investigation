import React from "react";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Button } from "@/components/ui";
import { X, Loader2, AlertCircle, CheckCircle } from "lucide-react";
import { Input } from "@/components/ui";
import { YamlEditor } from "./YamlEditor";
import { createResourceCmd } from "@/lib/tauriCommands";

interface RbacEditorProps {
  clusterId: string;
  namespace: string;
  onClose?: () => void;
}

type TabKey = "roles" | "clusterroles" | "rolebindings" | "clusterrolebindings";

interface TabState {
  name: string;
  yaml: string;
}

function buildRoleYaml(name: string, namespace: string): string {
  return `apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: ${name || "role-name"}
  namespace: ${namespace}
rules:
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list", "watch"]`;
}

function buildClusterRoleYaml(name: string): string {
  return `apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: ${name || "clusterrole-name"}
rules:
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list", "watch"]`;
}

function buildRoleBindingYaml(name: string, namespace: string): string {
  return `apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: ${name || "rolebinding-name"}
  namespace: ${namespace}
subjects:
- kind: User
  name: example-user
  apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: Role
  name: pod-reader
  apiGroup: rbac.authorization.k8s.io`;
}

function buildClusterRoleBindingYaml(name: string): string {
  return `apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: ${name || "clusterrolebinding-name"}
subjects:
- kind: User
  name: example-user
  apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: ClusterRole
  name: view
  apiGroup: rbac.authorization.k8s.io`;
}

export function RbacEditor({ clusterId, namespace, onClose }: RbacEditorProps) {
  const [activeTab, setActiveTab] = React.useState<TabKey>("roles");
  const [tabState, setTabState] = React.useState<Record<TabKey, TabState>>({
    roles: { name: "", yaml: buildRoleYaml("", namespace) },
    clusterroles: { name: "", yaml: buildClusterRoleYaml("") },
    rolebindings: { name: "", yaml: buildRoleBindingYaml("", namespace) },
    clusterrolebindings: { name: "", yaml: buildClusterRoleBindingYaml("") },
  });
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [success, setSuccess] = React.useState(false);

  const setName = (tab: TabKey, name: string) => {
    setTabState((prev) => {
      let yaml = prev[tab].yaml;
      if (tab === "roles") yaml = buildRoleYaml(name, namespace);
      else if (tab === "clusterroles") yaml = buildClusterRoleYaml(name);
      else if (tab === "rolebindings") yaml = buildRoleBindingYaml(name, namespace);
      else if (tab === "clusterrolebindings") yaml = buildClusterRoleBindingYaml(name);
      return { ...prev, [tab]: { name, yaml } };
    });
  };

  const setYaml = (tab: TabKey, yaml: string) => {
    setTabState((prev) => ({ ...prev, [tab]: { ...prev[tab], yaml } }));
  };

  const handleCreate = async () => {
    const { name, yaml } = tabState[activeTab];
    if (!name.trim()) return;

    setLoading(true);
    setError(null);
    setSuccess(false);

    const ns = activeTab === "clusterroles" || activeTab === "clusterrolebindings"
      ? ""
      : namespace;

    try {
      await createResourceCmd(clusterId, ns, activeTab, yaml);
      setSuccess(true);
      // Reset form for this tab
      setTabState((prev) => ({
        ...prev,
        [activeTab]: {
          name: "",
          yaml: activeTab === "roles"
            ? buildRoleYaml("", namespace)
            : activeTab === "clusterroles"
            ? buildClusterRoleYaml("")
            : activeTab === "rolebindings"
            ? buildRoleBindingYaml("", namespace)
            : buildClusterRoleBindingYaml(""),
        },
      }));
      setTimeout(() => {
        setSuccess(false);
        onClose?.();
      }, 1200);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const tabMeta: { id: TabKey; label: string }[] = [
    { id: "roles", label: "Roles" },
    { id: "clusterroles", label: "ClusterRoles" },
    { id: "rolebindings", label: "RoleBindings" },
    { id: "clusterrolebindings", label: "ClusterRoleBindings" },
  ];

  return (
    <div className="h-full flex flex-col">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-2xl font-semibold">RBAC Editor</h2>
        <Button variant="outline" onClick={onClose}>
          <X className="w-4 h-4 mr-2" />
          Close
        </Button>
      </div>

      {error && (
        <div className="mb-4 flex items-center gap-2 p-3 rounded-md bg-destructive/10 text-destructive text-sm">
          <AlertCircle className="w-4 h-4 flex-shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {success && (
        <div className="mb-4 flex items-center gap-2 p-3 rounded-md bg-green-500/10 text-green-600 text-sm">
          <CheckCircle className="w-4 h-4 flex-shrink-0" />
          <span>Resource created successfully.</span>
        </div>
      )}

      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as TabKey)}>
        <TabsList className="grid grid-cols-4 mb-4">
          {tabMeta.map((tab) => (
            <TabsTrigger key={tab.id} value={tab.id}>
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>

        <div className="flex-1 overflow-hidden">
          {tabMeta.map((tab) => (
            <TabsContent key={tab.id} value={tab.id} className="h-full flex flex-col gap-4">
              <div className="flex items-center gap-2">
                <Input
                  placeholder={`${tab.label.replace(/s$/, "")} name`}
                  value={tabState[tab.id].name}
                  onChange={(e) => setName(tab.id, e.target.value)}
                />
                <Button
                  disabled={!tabState[tab.id].name.trim() || loading}
                  onClick={handleCreate}
                >
                  {loading ? (
                    <Loader2 className="w-4 h-4 animate-spin mr-2" />
                  ) : null}
                  Create
                </Button>
              </div>

              <div className="flex-1 overflow-hidden">
                <YamlEditor
                  content={tabState[tab.id].yaml}
                  onChange={(yaml) => setYaml(tab.id, yaml)}
                  showControls={false}
                  height="100%"
                />
              </div>
            </TabsContent>
          ))}
        </div>
      </Tabs>
    </div>
  );
}
