import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import { X } from "lucide-react";
import { YamlEditor } from "./YamlEditor";
import type { ServiceInfo } from "@/lib/tauriCommands";

interface ServiceDetailProps {
  clusterId: string;
  namespace: string;
  service: ServiceInfo;
  onClose?: () => void;
}

export function ServiceDetail({ namespace, service, onClose }: ServiceDetailProps) {
  const [activeTab, setActiveTab] = React.useState("overview");

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">Service: {service.name}</h2>
          <Badge variant="outline">{namespace}</Badge>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="w-4 h-4" />
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-2 mb-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
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
                    <span className="font-mono">{service.name}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{service.namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Type</span>
                    <Badge variant="secondary">{service.type}</Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Cluster IP</span>
                    <span className="font-mono">{service.cluster_ip}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">External IP</span>
                    <span className="font-mono text-muted-foreground">
                      {service.external_ip ?? "none"}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Age</span>
                    <span className="text-sm">{service.age}</span>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Ports</CardTitle>
                </CardHeader>
                <CardContent>
                  {service.ports.length === 0 ? (
                    <span className="text-sm text-muted-foreground">No ports defined.</span>
                  ) : (
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>Name</TableHead>
                          <TableHead>Port</TableHead>
                          <TableHead>Protocol</TableHead>
                          <TableHead>Target Port</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {service.ports.map((p) => (
                          <TableRow key={`${p.port}-${p.protocol}`}>
                            <TableCell>{p.name ?? "—"}</TableCell>
                            <TableCell>{p.port}</TableCell>
                            <TableCell>{p.protocol}</TableCell>
                            <TableCell>{p.target_port ?? "—"}</TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  )}
                </CardContent>
              </Card>

              {Object.keys(service.selector).length > 0 && (
                <Card className="lg:col-span-2">
                  <CardHeader>
                    <CardTitle>Selector</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="flex flex-wrap gap-2">
                      {Object.entries(service.selector).map(([k, v]) => (
                        <Badge key={k} variant="secondary">
                          {k}={v}
                        </Badge>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}
            </div>
          </TabsContent>

          <TabsContent value="yaml" className="h-full">
            <YamlEditor
              readOnly
              showControls={false}
              content={JSON.stringify(service, null, 2)}
            />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
