import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import { X } from "lucide-react";
import { YamlEditor } from "./YamlEditor";

interface ServiceDetailProps {
  serviceName: string;
  namespace: string;
  _clusterId: string;
  onClose: () => void;
}

export function ServiceDetail({ serviceName, namespace, _clusterId, onClose }: ServiceDetailProps) {
  const [activeTab, setActiveTab] = React.useState("overview");

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">Service: {serviceName}</h2>
          <Badge variant="outline">{namespace}</Badge>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="w-4 h-4" />
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-4 mb-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="endpoints">Endpoints</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
          <TabsTrigger value="events">Events</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="overview" className="h-full overflow-y-auto">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              <Card>
                <CardHeader>
                  <CardTitle>Service Information</CardTitle>
                </CardHeader>
                <CardContent className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Name</span>
                    <span className="font-mono">{serviceName}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Type</span>
                    <Badge variant="secondary">ClusterIP</Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Cluster IP</span>
                    <span className="font-mono">10.96.0.1</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">External IP</span>
                    <span className="text-muted-foreground">none</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Port</span>
                    <span>80/TCP</span>
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
                  </div>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="endpoints" className="h-full overflow-y-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>IP</TableHead>
                  <TableHead>Port</TableHead>
                  <TableHead>Node</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>10.0.0.1</TableCell>
                  <TableCell>80</TableCell>
                  <TableCell>node-1</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>10.0.0.2</TableCell>
                  <TableCell>80</TableCell>
                  <TableCell>node-2</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>10.0.0.3</TableCell>
                  <TableCell>80</TableCell>
                  <TableCell>node-3</TableCell>
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
                  <TableCell>SettingClusterIP</TableCell>
                  <TableCell>Normal</TableCell>
                  <TableCell>Assigned cluster IP 10.96.0.1</TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
