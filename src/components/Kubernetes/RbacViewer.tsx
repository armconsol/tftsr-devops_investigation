import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import { Plus, Loader2, AlertCircle } from "lucide-react";
import {
  listRolesCmd,
  listClusterrolesCmd,
  listRolebindingsCmd,
  listClusterrolebindingsCmd,
  deleteResourceCmd,
  type RoleInfo,
  type ClusterRoleInfo,
  type RoleBindingInfo,
  type ClusterRoleBindingInfo,
} from "@/lib/tauriCommands";

interface RbacViewerProps {
  clusterId: string;
  namespace: string;
  onCreateRole?: () => void;
}

type ActiveTab = "roles" | "clusterroles" | "rolebindings" | "clusterrolebindings";

interface RbacData {
  roles: RoleInfo[];
  clusterRoles: ClusterRoleInfo[];
  roleBindings: RoleBindingInfo[];
  clusterRoleBindings: ClusterRoleBindingInfo[];
}

export function RbacViewer({ clusterId, namespace, onCreateRole }: RbacViewerProps) {
  const [activeTab, setActiveTab] = React.useState<ActiveTab>("roles");
  const [data, setData] = React.useState<RbacData>({
    roles: [],
    clusterRoles: [],
    roleBindings: [],
    clusterRoleBindings: [],
  });
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);
  const [deletingName, setDeletingName] = React.useState<string | null>(null);

  const fetchAll = React.useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [roles, clusterRoles, roleBindings, clusterRoleBindings] = await Promise.all([
        listRolesCmd(clusterId, namespace),
        listClusterrolesCmd(clusterId),
        listRolebindingsCmd(clusterId, namespace),
        listClusterrolebindingsCmd(clusterId),
      ]);
      setData({ roles, clusterRoles, roleBindings, clusterRoleBindings });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [clusterId, namespace]);

  React.useEffect(() => {
    fetchAll();
  }, [fetchAll]);

  const handleDelete = async (resourceType: string, ns: string, name: string) => {
    setDeletingName(name);
    try {
      await deleteResourceCmd(clusterId, resourceType, ns, name);
      await fetchAll();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setDeletingName(null);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64" data-testid="rbac-loading">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-64 gap-4" data-testid="rbac-error">
        <AlertCircle className="w-8 h-8 text-destructive" />
        <p className="text-sm text-muted-foreground">{error}</p>
        <Button variant="outline" onClick={fetchAll}>Retry</Button>
      </div>
    );
  }

  const tabs: { id: ActiveTab; label: string }[] = [
    { id: "roles", label: "Roles" },
    { id: "clusterroles", label: "ClusterRoles" },
    { id: "rolebindings", label: "RoleBindings" },
    { id: "clusterrolebindings", label: "ClusterRoleBindings" },
  ];

  return (
    <div className="h-full overflow-y-auto">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">RBAC Management</h2>
          <p className="text-muted-foreground">Cluster ID: {clusterId} | Namespace: {namespace}</p>
        </div>
        <Button onClick={onCreateRole}>
          <Plus className="w-4 h-4 mr-2" />
          Create Role
        </Button>
      </div>

      <div className="flex gap-1 mb-4 border-b">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-2 text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? "border-b-2 border-primary text-foreground"
                : "text-muted-foreground hover:text-foreground"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {activeTab === "roles" && (
        <div className="bg-card rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Namespace</TableHead>
                <TableHead>Age</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.roles.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={4} className="text-center text-muted-foreground py-8">
                    No roles found
                  </TableCell>
                </TableRow>
              ) : (
                data.roles.map((role) => (
                  <TableRow key={role.name}>
                    <TableCell className="font-medium">{role.name}</TableCell>
                    <TableCell className="font-mono text-sm">{role.namespace}</TableCell>
                    <TableCell className="text-muted-foreground">{role.age}</TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="sm"
                        disabled={deletingName === role.name}
                        onClick={() => handleDelete("roles", namespace, role.name)}
                      >
                        {deletingName === role.name ? (
                          <Loader2 className="w-3 h-3 animate-spin" />
                        ) : (
                          "Delete"
                        )}
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      )}

      {activeTab === "clusterroles" && (
        <div className="bg-card rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Age</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.clusterRoles.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={3} className="text-center text-muted-foreground py-8">
                    No cluster roles found
                  </TableCell>
                </TableRow>
              ) : (
                data.clusterRoles.map((cr) => (
                  <TableRow key={cr.name}>
                    <TableCell className="font-medium">{cr.name}</TableCell>
                    <TableCell className="text-muted-foreground">{cr.age}</TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="sm"
                        disabled={deletingName === cr.name}
                        onClick={() => handleDelete("clusterroles", "", cr.name)}
                      >
                        {deletingName === cr.name ? (
                          <Loader2 className="w-3 h-3 animate-spin" />
                        ) : (
                          "Delete"
                        )}
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      )}

      {activeTab === "rolebindings" && (
        <div className="bg-card rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Namespace</TableHead>
                <TableHead>Role</TableHead>
                <TableHead>Age</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.roleBindings.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                    No role bindings found
                  </TableCell>
                </TableRow>
              ) : (
                data.roleBindings.map((rb) => (
                  <TableRow key={rb.name}>
                    <TableCell className="font-medium">{rb.name}</TableCell>
                    <TableCell className="font-mono text-sm">{rb.namespace}</TableCell>
                    <TableCell>{rb.role}</TableCell>
                    <TableCell className="text-muted-foreground">{rb.age}</TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="sm"
                        disabled={deletingName === rb.name}
                        onClick={() => handleDelete("rolebindings", namespace, rb.name)}
                      >
                        {deletingName === rb.name ? (
                          <Loader2 className="w-3 h-3 animate-spin" />
                        ) : (
                          "Delete"
                        )}
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      )}

      {activeTab === "clusterrolebindings" && (
        <div className="bg-card rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>ClusterRole</TableHead>
                <TableHead>Age</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.clusterRoleBindings.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={4} className="text-center text-muted-foreground py-8">
                    No cluster role bindings found
                  </TableCell>
                </TableRow>
              ) : (
                data.clusterRoleBindings.map((crb) => (
                  <TableRow key={crb.name}>
                    <TableCell className="font-medium">{crb.name}</TableCell>
                    <TableCell>{crb.cluster_role}</TableCell>
                    <TableCell className="text-muted-foreground">{crb.age}</TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="sm"
                        disabled={deletingName === crb.name}
                        onClick={() => handleDelete("clusterrolebindings", "", crb.name)}
                      >
                        {deletingName === crb.name ? (
                          <Loader2 className="w-3 h-3 animate-spin" />
                        ) : (
                          "Delete"
                        )}
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}
