import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Button } from "@/components/ui";
import { X } from "lucide-react";
import { YamlEditor } from "./YamlEditor";
import type { ConfigMapInfo } from "@/lib/tauriCommands";

interface ConfigMapDetailProps {
  clusterId: string;
  namespace: string;
  configMap: ConfigMapInfo;
  onClose?: () => void;
}

export function ConfigMapDetail({ namespace, configMap, onClose }: ConfigMapDetailProps) {
  const [activeTab, setActiveTab] = React.useState("data");

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">ConfigMap: {configMap.name}</h2>
          <Badge variant="outline">{namespace}</Badge>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="w-4 h-4" />
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-3 mb-4">
          <TabsTrigger value="data">Data</TabsTrigger>
          <TabsTrigger value="metadata">Metadata</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="data" className="h-full overflow-y-auto">
            <Card>
              <CardHeader>
                <CardTitle>ConfigMap Data</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <span>Keys:</span>
                  <Badge variant="secondary">{configMap.data_keys}</Badge>
                </div>
                <p className="mt-3 text-sm text-muted-foreground">
                  The backend returns a key count only. Full data values are available via{" "}
                  <code className="font-mono text-xs">kubectl get configmap</code>.
                </p>
              </CardContent>
            </Card>
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
                    <span className="font-mono">{configMap.name}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{configMap.namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Data Keys</span>
                    <span>{configMap.data_keys}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Age</span>
                    <span className="text-sm">{configMap.age}</span>
                  </div>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="yaml" className="h-full">
            <YamlEditor
              readOnly
              showControls={false}
              content={JSON.stringify(configMap, null, 2)}
            />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
