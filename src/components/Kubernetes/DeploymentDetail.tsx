import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import { X } from "lucide-react";
import { YamlEditor } from "./YamlEditor";

interface DeploymentDetailProps {
  deploymentName: string;
  namespace: string;
  _clusterId: string;
  onClose: () => void;
}

export function DeploymentDetail({ deploymentName, namespace, _clusterId, onClose }: DeploymentDetailProps) {
  const [activeTab, setActiveTab] = React.useState("overview");

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">Deployment: {deploymentName}</h2>
          <Badge variant="outline">{namespace}</Badge>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="w-4 h-4" />
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-4 mb-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="replicas">Replicas</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
          <TabsTrigger value="events">Events</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="overview" className="h-full overflow-y-auto">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              <Card>
                <CardHeader>
                  <CardTitle>Deployment Information</CardTitle>
                </CardHeader>
                <CardContent className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Name</span>
                    <span className="font-mono">{deploymentName}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Replicas</span>
                    <span>3/3 Ready</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Strategy</span>
                    <span>RollingUpdate</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Image</span>
                    <span className="font-mono">nginx:latest</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Created</span>
                    <span className="text-sm">2 hours ago</span>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Selector</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="flex flex-wrap gap-2">
                    <Badge variant="secondary">app=web</Badge>
                    <Badge variant="secondary">tier=frontend</Badge>
                  </div>
                </CardContent>
              </Card>

              <Card className="lg:col-span-2">
                <CardHeader>
                  <CardTitle>Labels</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="flex flex-wrap gap-2">
                    <Badge variant="secondary">app=web</Badge>
                    <Badge variant="secondary">tier=frontend</Badge>
                    <Badge variant="secondary">version=v1</Badge>
                  </div>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="replicas" className="h-full overflow-y-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Ready</TableHead>
                  <TableHead>Age</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>{deploymentName}-abc123</TableCell>
                  <TableCell>Running</TableCell>
                  <TableCell>1/1</TableCell>
                  <TableCell>2h</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>{deploymentName}-def456</TableCell>
                  <TableCell>Running</TableCell>
                  <TableCell>1/1</TableCell>
                  <TableCell>2h</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>{deploymentName}-ghi789</TableCell>
                  <TableCell>Running</TableCell>
                  <TableCell>1/1</TableCell>
                  <TableCell>2h</TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </TabsContent>

          <TabsContent value="yaml" className="h-full">
            <YamlEditor onChange={() => {}} />
          </TabsContent>

          <TabsContent value="events" className="h-full overflow-y-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Time</TableHead>
                  <TableHead>Reason</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Message</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>2 hours ago</TableCell>
                  <TableCell>ScalingReplicaSet</TableCell>
                  <TableCell>Normal</TableCell>
                  <TableCell>Scaled up replica set {deploymentName}-abc123 to 3</TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
