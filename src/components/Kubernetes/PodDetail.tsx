import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import { Copy, Terminal, X } from "lucide-react";
import { YamlEditor } from "./YamlEditor";

interface PodDetailProps {
  podName: string;
  namespace: string;
  _clusterId: string;
  onClose: () => void;
}

export function PodDetail({ podName, namespace, _clusterId, onClose }: PodDetailProps) {
  const [activeTab, setActiveTab] = React.useState("overview");

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">Pod: {podName}</h2>
          <Badge variant="outline">{namespace}</Badge>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="w-4 h-4" />
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-4 mb-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="logs">Logs</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
          <TabsTrigger value="events">Events</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="overview" className="h-full overflow-y-auto">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              <Card>
                <CardHeader>
                  <CardTitle>Pod Information</CardTitle>
                </CardHeader>
                <CardContent className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Name</span>
                    <span className="font-mono">{podName}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Status</span>
                    <Badge variant="default">Running</Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">IP</span>
                    <span className="font-mono">10.0.0.1</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Node</span>
                    <span className="font-mono">node-1</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Restart Count</span>
                    <span>0</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Created</span>
                    <span className="text-sm">2 hours ago</span>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Containers</CardTitle>
                </CardHeader>
                <CardContent>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Name</TableHead>
                        <TableHead>Image</TableHead>
                        <TableHead>State</TableHead>
                        <TableHead>Ready</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      <TableRow>
                        <TableCell>example</TableCell>
                        <TableCell className="font-mono">nginx:latest</TableCell>
                        <TableCell>Running</TableCell>
                        <TableCell>True</TableCell>
                      </TableRow>
                    </TableBody>
                  </Table>
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

          <TabsContent value="logs" className="h-full">
            <Card className="h-full flex flex-col">
              <CardHeader className="flex flex-row items-center justify-between">
                <CardTitle>Container Logs</CardTitle>
                <div className="flex items-center gap-2">
                  <Button variant="outline" size="sm">
                    <Terminal className="w-4 h-4 mr-2" />
                    Execute
                  </Button>
                  <Button variant="outline" size="sm">
                    <Copy className="w-4 h-4 mr-2" />
                    Copy
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="flex-1 bg-slate-900 rounded-md p-4 overflow-auto font-mono text-sm">
                <div className="text-green-400">[INFO] Starting nginx server...</div>
                <div className="text-green-400">[INFO] Listening on port 80</div>
                <div className="text-blue-400">[ACCESS] GET / - 200 OK</div>
                <div className="text-blue-400">[ACCESS] GET /css/style.css - 200 OK</div>
                <div className="text-blue-400">[ACCESS] GET /js/app.js - 200 OK</div>
                <div className="text-yellow-400">[WARN] Slow response time detected</div>
                <div className="text-blue-400">[ACCESS] POST /api/data - 201 Created</div>
              </CardContent>
            </Card>
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
                  <TableCell>Pulled</TableCell>
                  <TableCell>Normal</TableCell>
                  <TableCell>Container image "nginx:latest" already present on machine</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>2 hours ago</TableCell>
                  <TableCell>Created</TableCell>
                  <TableCell>Normal</TableCell>
                  <TableCell>Created container example</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>2 hours ago</TableCell>
                  <TableCell>Started</TableCell>
                  <TableCell>Normal</TableCell>
                  <TableCell>Started container example</TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
