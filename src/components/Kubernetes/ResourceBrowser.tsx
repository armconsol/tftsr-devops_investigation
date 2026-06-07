import React, { useState, useEffect, useCallback, useMemo } from "react";
import { Card, CardContent, CardHeader } from "@/components/ui";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";
import { Button } from "@/components/ui";
import { Loader2, AlertCircle } from "lucide-react";
import type { NamespaceInfo, PodInfo, ServiceInfo, DeploymentInfo, StatefulSetInfo, DaemonSetInfo } from "@/lib/tauriCommands";
import { listNamespacesCmd, listPodsCmd, listServicesCmd, listDeploymentsCmd, listStatefulsetsCmd, listDaemonsetsCmd } from "@/lib/tauriCommands";
import { PodList } from "./PodList";
import { ServiceList } from "./ServiceList";
import { DeploymentList } from "./DeploymentList";
import { StatefulSetList } from "./StatefulSetList";
import { DaemonSetList } from "./DaemonSetList";

type ResourceType = "pods" | "services" | "deployments" | "statefulsets" | "daemonsets";

interface ResourceBrowserProps {
  clusterId: string;
}

export function ResourceBrowser({ clusterId }: ResourceBrowserProps) {
  const [namespaces, setNamespaces] = useState<NamespaceInfo[]>([]);
  const [selectedNamespace, setSelectedNamespace] = useState<string>("all");
  const [resourceType, setResourceType] = useState<ResourceType>("pods");
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [pods, setPods] = useState<PodInfo[]>([]);
  const [services, setServices] = useState<ServiceInfo[]>([]);
  const [deployments, setDeployments] = useState<DeploymentInfo[]>([]);
  const [statefulsets, setStatefulsets] = useState<StatefulSetInfo[]>([]);
  const [daemonsets, setDaemonsets] = useState<DaemonSetInfo[]>([]);

  const loadData = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [namespacesData, podsData, servicesData, deploymentsData, statefulsetsData, daemonsetsData] = await Promise.all([
        listNamespacesCmd(clusterId),
        selectedNamespace === "all" ? listPodsCmd(clusterId, "") : listPodsCmd(clusterId, selectedNamespace),
        selectedNamespace === "all" ? listServicesCmd(clusterId, "") : listServicesCmd(clusterId, selectedNamespace),
        selectedNamespace === "all" ? listDeploymentsCmd(clusterId, "") : listDeploymentsCmd(clusterId, selectedNamespace),
        selectedNamespace === "all" ? listStatefulsetsCmd(clusterId, "") : listStatefulsetsCmd(clusterId, selectedNamespace),
        selectedNamespace === "all" ? listDaemonsetsCmd(clusterId, "") : listDaemonsetsCmd(clusterId, selectedNamespace),
      ]);

      setNamespaces(namespacesData);
      setPods(podsData);
      setServices(servicesData);
      setDeployments(deploymentsData);
      setStatefulsets(statefulsetsData);
      setDaemonsets(daemonsetsData);
    } catch (err) {
      console.error("Failed to load resources:", err);
      setError(err instanceof Error ? err.message : "Failed to load resources");
    } finally {
      setIsLoading(false);
    }
  }, [clusterId, selectedNamespace]);

  useEffect(() => {
    loadData();
  }, [loadData, resourceType]);

  const namespaceOptions = useMemo(() => {
    const options = [{ name: "All Namespaces", value: "all" }];
    namespaces.forEach(ns => {
      options.push({ name: ns.name, value: ns.name });
    });
    return options;
  }, [namespaces]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="w-8 h-8 animate-spin text-primary" />
          <p className="text-muted-foreground">Loading Kubernetes resources...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <Card className="w-full max-w-md">
          <CardContent className="pt-6">
            <div className="flex flex-col items-center gap-4">
              <AlertCircle className="w-12 h-12 text-destructive" />
              <p className="text-center text-muted-foreground">{error}</p>
              <Button onClick={loadData}>Retry</Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  const renderResourceList = () => {
    switch (resourceType) {
      case "pods":
        return <PodList pods={pods} clusterId={clusterId} namespace={selectedNamespace} />;
      case "services":
        return <ServiceList services={services} clusterId={clusterId} namespace={selectedNamespace} />;
      case "deployments":
        return <DeploymentList deployments={deployments} clusterId={clusterId} namespace={selectedNamespace} />;
      case "statefulsets":
        return <StatefulSetList statefulsets={statefulsets} clusterId={clusterId} namespace={selectedNamespace} />;
      case "daemonsets":
        return <DaemonSetList daemonsets={daemonsets} clusterId={clusterId} namespace={selectedNamespace} />;
      default:
        return null;
    }
  };

  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-bold tracking-tight">Kubernetes Resources</h1>
        <p className="text-muted-foreground">
          Browse and manage your Kubernetes resources
        </p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div className="flex items-center gap-4">
              <Select value={selectedNamespace} onValueChange={setSelectedNamespace}>
                <SelectTrigger className="w-[200px]">
                  <SelectValue placeholder="Select namespace" />
                </SelectTrigger>
                <SelectContent>
                  {namespaceOptions.map((ns) => (
                    <SelectItem key={ns.value} value={ns.value}>
                      {ns.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardHeader>
      </Card>

      <Card className="flex-1 flex flex-col">
        <CardHeader>
          <Tabs value={resourceType} onValueChange={(v) => setResourceType(v as ResourceType)}>
            <TabsList className="grid grid-cols-5">
              <TabsTrigger value="pods">Pods</TabsTrigger>
              <TabsTrigger value="services">Services</TabsTrigger>
              <TabsTrigger value="deployments">Deployments</TabsTrigger>
              <TabsTrigger value="statefulsets">StatefulSets</TabsTrigger>
              <TabsTrigger value="daemonsets">DaemonSets</TabsTrigger>
            </TabsList>
          </Tabs>
        </CardHeader>
        <CardContent className="flex-1 overflow-hidden">
          <div className="h-full overflow-y-auto">
            <TabsContent value={resourceType} className="h-full">
              {renderResourceList()}
            </TabsContent>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
