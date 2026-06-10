import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import { Copy, Network, X } from "lucide-react";
import { Loader2 } from "lucide-react";
import { PortForwardDialog } from "./PortForwardDialog";
import { YamlEditor } from "./YamlEditor";
import { getPodLogsCmd } from "@/lib/tauriCommands";
import type { PodInfo } from "@/lib/tauriCommands";

interface PodDetailProps {
  clusterId: string;
  namespace: string;
  pod: PodInfo;
  onClose?: () => void;
}

export function PodDetail({ clusterId, namespace, pod, onClose }: PodDetailProps) {
  const [activeTab, setActiveTab] = React.useState("overview");
  const [selectedContainer, setSelectedContainer] = React.useState(pod.containers[0] ?? "");
  const [logs, setLogs] = React.useState<string | null>(null);
  const [logsLoading, setLogsLoading] = React.useState(false);
  const [logsError, setLogsError] = React.useState<string | null>(null);
  const [portForwardOpen, setPortForwardOpen] = React.useState(false);

  const fetchLogs = React.useCallback(
    async (containerName: string) => {
      if (!containerName) return;
      setLogsLoading(true);
      setLogsError(null);
      setLogs(null);
      try {
        const response = await getPodLogsCmd(clusterId, namespace, pod.name, containerName);
        setLogs(response.logs);
      } catch (err) {
        setLogsError(err instanceof Error ? err.message : String(err));
      } finally {
        setLogsLoading(false);
      }
    },
    [clusterId, namespace, pod.name]
  );

  const handleTabChange = (tab: string) => {
    setActiveTab(tab);
    if (tab === "logs" && logs === null && !logsLoading && !logsError) {
      void fetchLogs(selectedContainer);
    }
  };

  const handleContainerChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const name = e.target.value;
    setSelectedContainer(name);
    void fetchLogs(name);
  };

  const copyLogs = () => {
    if (logs) void navigator.clipboard.writeText(logs);
  };

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">Pod: {pod.name}</h2>
          <Badge variant="outline">{namespace}</Badge>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={() => setPortForwardOpen(true)}>
            <Network className="w-4 h-4 mr-1.5" />
            Port Forward
          </Button>
          <Button variant="ghost" size="sm" onClick={onClose}>
            <X className="w-4 h-4" />
          </Button>
        </div>
      </div>

      <PortForwardDialog
        open={portForwardOpen}
        onOpenChange={setPortForwardOpen}
        clusterId={clusterId}
        namespace={namespace}
        podName={pod.name}
      />

      <Tabs value={activeTab} onValueChange={handleTabChange}>
        <TabsList className="grid grid-cols-3 mb-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="logs">Logs</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
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
                    <span className="font-mono">{pod.name}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Status</span>
                    <Badge variant={pod.status === "Running" ? "default" : "secondary"}>
                      {pod.status}
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Ready</span>
                    <span className="font-mono">{pod.ready}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Age</span>
                    <span className="text-sm">{pod.age}</span>
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
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {pod.containers.map((c) => (
                        <TableRow key={c}>
                          <TableCell className="font-mono">{c}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="logs" className="h-full">
            <Card className="h-full flex flex-col">
              <CardHeader className="flex flex-row items-center justify-between">
                <CardTitle>Container Logs</CardTitle>
                <div className="flex items-center gap-2">
                  {pod.containers.length > 1 && (
                    <select
                      value={selectedContainer}
                      onChange={handleContainerChange}
                      className="text-sm border rounded px-2 py-1 bg-background"
                    >
                      {pod.containers.map((c) => (
                        <option key={c} value={c}>
                          {c}
                        </option>
                      ))}
                    </select>
                  )}
                  <Button variant="outline" size="sm" onClick={copyLogs} disabled={!logs}>
                    <Copy className="w-4 h-4 mr-2" />
                    Copy
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="flex-1 bg-slate-900 rounded-md p-4 overflow-auto font-mono text-sm">
                {logsLoading && (
                  <div
                    data-testid="logs-loading"
                    className="flex items-center gap-2 text-muted-foreground"
                  >
                    <Loader2 className="w-4 h-4 animate-spin" />
                    Loading logs…
                  </div>
                )}
                {logsError && (
                  <div data-testid="logs-error" className="text-red-400">
                    Failed to load logs: {logsError}
                  </div>
                )}
                {!logsLoading && !logsError && logs !== null && (
                  <pre className="text-green-400 whitespace-pre-wrap break-words">{logs}</pre>
                )}
                {!logsLoading && !logsError && logs === null && (
                  <span className="text-muted-foreground">Select a container to view logs.</span>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="yaml" className="h-full">
            <YamlEditor
              readOnly
              showControls={false}
              content={JSON.stringify(pod, null, 2)}
            />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
