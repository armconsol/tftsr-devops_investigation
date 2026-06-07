import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Button } from "@/components/ui";
import { X } from "lucide-react";
import { YamlEditor } from "./YamlEditor";

interface ConfigMapDetailProps {
  configMapName: string;
  namespace: string;
  _clusterId: string;
  onClose: () => void;
}

export function ConfigMapDetail({ configMapName, namespace, _clusterId, onClose }: ConfigMapDetailProps) {
  const [activeTab, setActiveTab] = React.useState("data");

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">ConfigMap: {configMapName}</h2>
          <Badge variant="outline">{namespace}</Badge>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="w-4 h-4" />
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-3 mb-4">
          <TabsTrigger value="data">Data</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
          <TabsTrigger value="metadata">Metadata</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="data" className="h-full overflow-y-auto">
            <Card className="h-full flex flex-col">
              <CardHeader>
                <CardTitle>ConfigMap Data</CardTitle>
              </CardHeader>
              <CardContent className="flex-1 bg-slate-900 rounded-md p-4 overflow-auto font-mono text-sm">
                <div className="space-y-2">
                  <div>
                    <span className="text-blue-400">config.json:</span>
                    <pre className="mt-1 text-green-400">{`{
  "debug": true,
  "logLevel": "info"
}`}</pre>
                  </div>
                  <div>
                    <span className="text-blue-400">app.properties:</span>
                    <pre className="mt-1 text-green-400">{`app.name=MyApp
app.version=1.0.0
app.port=8080`}</pre>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="yaml" className="h-full">
            <YamlEditor onChange={() => {}} />
          </TabsContent>

          <TabsContent value="metadata" className="h-full overflow-y-auto">
            <div className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Metadata</CardTitle>
                </CardHeader>
                <CardContent className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Name</span>
                    <span className="font-mono">{configMapName}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">UID</span>
                    <span className="font-mono text-xs">abc123-def456</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Created</span>
                    <span className="text-sm">2 hours ago</span>
                  </div>
                </CardContent>
              </Card>

              <Card>
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
        </div>
      </Tabs>
    </div>
  );
}
