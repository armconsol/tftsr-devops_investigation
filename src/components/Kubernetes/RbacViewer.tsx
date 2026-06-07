import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import { Plus, Shield, User } from "lucide-react";

interface RbacViewerProps {
  clusterId: string;
  namespace: string;
}

export function RbacViewer({ clusterId, namespace }: RbacViewerProps) {
  return (
    <div className="h-full overflow-y-auto">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">RBAC Management</h2>
          <p className="text-muted-foreground">Cluster ID: {clusterId} | Namespace: {namespace}</p>
        </div>
        <Button>
          <Plus className="w-4 h-4 mr-2" />
          Create Role
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-card rounded-lg border">
          <div className="border-b px-6 py-4">
            <h3 className="font-semibold flex items-center gap-2">
              <Shield className="w-5 h-5" />
              Roles
            </h3>
          </div>
          <div className="p-6">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Namespace</TableHead>
                  <TableHead>Rules</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>pod-reader</TableCell>
                  <TableCell className="font-mono">{namespace}</TableCell>
                  <TableCell>get, list, watch pods</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>secret-viewer</TableCell>
                  <TableCell className="font-mono">{namespace}</TableCell>
                  <TableCell>get, list secrets</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>deployment-manager</TableCell>
                  <TableCell className="font-mono">{namespace}</TableCell>
                  <TableCell>get, list, create, update deployments</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </div>

        <div className="bg-card rounded-lg border">
          <div className="border-b px-6 py-4">
            <h3 className="font-semibold flex items-center gap-2">
              <Shield className="w-5 h-5" />
              ClusterRoles
            </h3>
          </div>
          <div className="p-6">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Rules</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>admin</TableCell>
                  <TableCell>Full access to all resources</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>edit</TableCell>
                  <TableCell>Modify resources in namespace</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>view</TableCell>
                  <TableCell>Read-only access to resources</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </div>

        <div className="bg-card rounded-lg border">
          <div className="border-b px-6 py-4">
            <h3 className="font-semibold flex items-center gap-2">
              <User className="w-5 h-5" />
              RoleBindings
            </h3>
          </div>
          <div className="p-6">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Role</TableHead>
                  <TableHead>Subjects</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>pod-reader-binding</TableCell>
                  <TableCell>pod-reader</TableCell>
                  <TableCell>user:alice</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>deployment-manager-binding</TableCell>
                  <TableCell>deployment-manager</TableCell>
                  <TableCell>group:devs</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </div>

        <div className="bg-card rounded-lg border">
          <div className="border-b px-6 py-4">
            <h3 className="font-semibold flex items-center gap-2">
              <User className="w-5 h-5" />
              ClusterRoleBindings
            </h3>
          </div>
          <div className="p-6">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>ClusterRole</TableHead>
                  <TableHead>Subjects</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>admin-binding</TableCell>
                  <TableCell>admin</TableCell>
                  <TableCell>group:admins</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>view-binding</TableCell>
                  <TableCell>view</TableCell>
                  <TableCell>group:auditors</TableCell>
                  <TableCell>
                    <Button variant="ghost" size="sm">Edit</Button>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </div>
      </div>
    </div>
  );
}
