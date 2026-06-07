import React from "react";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Button } from "@/components/ui";
import { Plus, X, Check } from "lucide-react";
import { Input } from "@/components/ui";

interface RbacEditorProps {
  _clusterId: string;
  namespace: string;
  onClose: () => void;
}

export function RbacEditor({ _clusterId, namespace, onClose }: RbacEditorProps) {
  const [activeTab, setActiveTab] = React.useState("roles");
  const [newRoleName, setNewRoleName] = React.useState("");

  return (
    <div className="h-full flex flex-col">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-2xl font-semibold">RBAC Editor</h2>
        <div className="flex items-center gap-2">
          <Button variant="outline" onClick={onClose}>
            <X className="w-4 h-4 mr-2" />
            Cancel
          </Button>
          <Button>
            <Check className="w-4 h-4 mr-2" />
            Save Changes
          </Button>
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-4 mb-4">
          <TabsTrigger value="roles">Roles</TabsTrigger>
          <TabsTrigger value="clusterroles">ClusterRoles</TabsTrigger>
          <TabsTrigger value="rolebindings">RoleBindings</TabsTrigger>
          <TabsTrigger value="clusterrolebindings">ClusterRoleBindings</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="roles" className="h-full flex flex-col">
            <div className="mb-4 flex items-center gap-2">
              <Input
                placeholder="New role name"
                value={newRoleName}
                onChange={(e) => setNewRoleName(e.target.value)}
              />
              <Button disabled={!newRoleName}>
                <Plus className="w-4 h-4 mr-2" />
                Create Role
              </Button>
            </div>

            <div className="flex-1 overflow-hidden">
              <div className="bg-card rounded-lg border flex flex-col h-full">
                <div className="border-b px-6 py-4">
                  <h3 className="font-semibold">Role YAML Editor</h3>
                </div>
                <div className="flex-1 bg-slate-900 p-4 font-mono text-sm text-green-400 overflow-auto">
                  <div className="space-y-1">
                    <div>
                      <span className="text-blue-400">apiVersion:</span> rbac.authorization.k8s.io/v1
                    </div>
                    <div>
                      <span className="text-blue-400">kind:</span> Role
                    </div>
                    <div>
                      <span className="text-blue-400">metadata:</span>
                    </div>
                    <div className="pl-4">
                      <span className="text-blue-400">name:</span> {newRoleName || "role-name"}
                    </div>
                    <div className="pl-4">
                      <span className="text-blue-400">namespace:</span> {namespace}
                    </div>
                    <div>
                      <span className="text-blue-400">rules:</span>
                    </div>
                    <div className="pl-4">
                      <span className="text-blue-400">-</span> <span className="text-blue-400">apiGroups:</span> [""]
                    </div>
                    <div className="pl-6">
                      <span className="text-blue-400">resources:</span> ["pods"]
                    </div>
                    <div className="pl-6">
                      <span className="text-blue-400">verbs:</span> ["get", "list", "watch"]
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="clusterroles" className="h-full flex flex-col">
            <div className="text-center py-12 text-muted-foreground">
              <p>ClusterRole editing would be displayed here</p>
            </div>
          </TabsContent>

          <TabsContent value="rolebindings" className="h-full flex flex-col">
            <div className="text-center py-12 text-muted-foreground">
              <p>RoleBinding editing would be displayed here</p>
            </div>
          </TabsContent>

          <TabsContent value="clusterrolebindings" className="h-full flex flex-col">
            <div className="text-center py-12 text-muted-foreground">
              <p>ClusterRoleBinding editing would be displayed here</p>
            </div>
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
