import React, { useCallback, useEffect, useState } from "react";
import { Plus, RefreshCw, Search, ChevronDown, ChevronRight } from "lucide-react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  Input,
  Label,
  Badge,
} from "@/components/ui";
import {
  helmListReposCmd,
  helmSearchRepoCmd,
  helmAddRepoCmd,
  helmUpdateReposCmd,
} from "@/lib/tauriCommands";
import type { HelmRepository, HelmChart } from "@/lib/tauriCommands";

interface HelmChartListProps {
  clusterId: string;
}

export function HelmChartList({ clusterId }: HelmChartListProps) {
  const [repos, setRepos] = useState<HelmRepository[]>([]);
  const [charts, setCharts] = useState<HelmChart[]>([]);
  const [selectedRepo, setSelectedRepo] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(false);
  const [updatingRepos, setUpdatingRepos] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedChart, setExpandedChart] = useState<string | null>(null);

  const [addRepoOpen, setAddRepoOpen] = useState(false);
  const [newRepoName, setNewRepoName] = useState("");
  const [newRepoUrl, setNewRepoUrl] = useState("");
  const [addingRepo, setAddingRepo] = useState(false);
  const [addRepoError, setAddRepoError] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const repoList = await helmListReposCmd(clusterId);
      setRepos(repoList);
      const chartList = await helmSearchRepoCmd(clusterId, "");
      setCharts(chartList);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [clusterId]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  const handleUpdateRepos = async () => {
    setUpdatingRepos(true);
    setError(null);
    try {
      await helmUpdateReposCmd(clusterId);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setUpdatingRepos(false);
    }
  };

  const handleAddRepo = async () => {
    if (!newRepoName.trim() || !newRepoUrl.trim()) return;
    setAddingRepo(true);
    setAddRepoError(null);
    try {
      await helmAddRepoCmd(clusterId, newRepoName.trim(), newRepoUrl.trim());
      setAddRepoOpen(false);
      setNewRepoName("");
      setNewRepoUrl("");
      await loadData();
    } catch (err) {
      setAddRepoError(err instanceof Error ? err.message : String(err));
    } finally {
      setAddingRepo(false);
    }
  };

  const filteredCharts = charts.filter((c) => {
    const matchesRepo = selectedRepo == null || c.repository === selectedRepo;
    const matchesSearch =
      search.trim() === "" ||
      c.name.toLowerCase().includes(search.toLowerCase()) ||
      c.description.toLowerCase().includes(search.toLowerCase());
    return matchesRepo && matchesSearch;
  });

  return (
    <div className="flex flex-col gap-4 h-full">
      {/* Toolbar */}
      <div className="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          variant="outline"
          onClick={() => void handleUpdateRepos()}
          disabled={updatingRepos}
        >
          <RefreshCw className={`h-3.5 w-3.5 mr-1 ${updatingRepos ? "animate-spin" : ""}`} />
          Update Repos
        </Button>
        <Button size="sm" variant="outline" onClick={() => setAddRepoOpen(true)}>
          <Plus className="h-3.5 w-3.5 mr-1" />
          Add Repository
        </Button>
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search charts…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </div>
      )}

      <div className="flex gap-4 flex-1 min-h-0 overflow-hidden">
        {/* Repository sidebar */}
        <div className="w-48 flex-shrink-0 border rounded-md overflow-y-auto">
          <div className="px-3 py-2 border-b text-xs font-semibold text-muted-foreground uppercase tracking-wide">
            Repositories
          </div>
          <div
            className={`px-3 py-2 text-sm cursor-pointer transition-colors ${
              selectedRepo == null ? "bg-accent text-accent-foreground" : "hover:bg-muted/50"
            }`}
            onClick={() => setSelectedRepo(null)}
          >
            All repositories
          </div>
          {repos.map((repo) => (
            <div
              key={repo.name}
              className={`px-3 py-2 text-sm cursor-pointer transition-colors truncate ${
                selectedRepo === repo.name
                  ? "bg-accent text-accent-foreground"
                  : "hover:bg-muted/50"
              }`}
              title={repo.name}
              onClick={() => setSelectedRepo(repo.name)}
            >
              {repo.name}
            </div>
          ))}
          {repos.length === 0 && !loading && (
            <div className="px-3 py-4 text-xs text-muted-foreground">No repos</div>
          )}
        </div>

        {/* Charts table */}
        <div className="flex-1 overflow-auto border rounded-md">
          {loading ? (
            <div className="flex items-center justify-center h-32 text-muted-foreground">
              <RefreshCw className="h-5 w-5 animate-spin mr-2" />
              Loading charts…
            </div>
          ) : repos.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-32 text-center gap-2 text-muted-foreground text-sm px-4">
              <p>No helm repositories configured.</p>
              <p>Add a repository to get started.</p>
            </div>
          ) : filteredCharts.length === 0 ? (
            <div className="flex items-center justify-center h-32 text-muted-foreground text-sm">
              No charts match your search.
            </div>
          ) : (
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b text-muted-foreground">
                  <th className="text-left px-4 py-3 font-medium">Name</th>
                  <th className="text-left px-4 py-3 font-medium">Version</th>
                  <th className="text-left px-4 py-3 font-medium">App Version</th>
                  <th className="text-left px-4 py-3 font-medium">Repository</th>
                  <th className="text-left px-4 py-3 font-medium">Description</th>
                </tr>
              </thead>
              <tbody>
                {filteredCharts.map((chart) => {
                  const key = `${chart.repository}/${chart.name}`;
                  const isExpanded = expandedChart === key;
                  return (
                    <React.Fragment key={key}>
                      <tr
                        className="border-b last:border-0 hover:bg-muted/30 transition-colors cursor-pointer"
                        onClick={() => setExpandedChart(isExpanded ? null : key)}
                      >
                        <td className="px-4 py-3">
                          <div className="flex items-center gap-1.5 font-medium">
                            {isExpanded ? (
                              <ChevronDown className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />
                            ) : (
                              <ChevronRight className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />
                            )}
                            {chart.name.includes("/") ? chart.name.split("/").slice(1).join("/") : chart.name}
                          </div>
                        </td>
                        <td className="px-4 py-3 font-mono text-xs">{chart.chart_version}</td>
                        <td className="px-4 py-3 font-mono text-xs">{chart.app_version || "—"}</td>
                        <td className="px-4 py-3">
                          <Badge variant="secondary" className="text-xs">
                            {chart.repository}
                          </Badge>
                        </td>
                        <td className="px-4 py-3 text-muted-foreground max-w-xs truncate">
                          {chart.description || "—"}
                        </td>
                      </tr>
                      {isExpanded && (
                        <tr className="border-b bg-muted/20">
                          <td colSpan={5} className="px-6 py-3">
                            <div className="space-y-1.5 text-sm">
                              <div className="font-medium">
                                {chart.repository}/{chart.name}
                              </div>
                              <div className="text-muted-foreground">{chart.description || "No description available."}</div>
                              <div className="flex gap-4 text-xs text-muted-foreground">
                                <span>Chart: {chart.chart_version}</span>
                                {chart.app_version && <span>App: {chart.app_version}</span>}
                              </div>
                            </div>
                          </td>
                        </tr>
                      )}
                    </React.Fragment>
                  );
                })}
              </tbody>
            </table>
          )}
        </div>
      </div>

      {/* Add Repository Dialog */}
      <Dialog open={addRepoOpen} onOpenChange={setAddRepoOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Add Helm Repository</DialogTitle>
          </DialogHeader>
          <div className="flex flex-col gap-4 py-2">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="repo-name">Name</Label>
              <Input
                id="repo-name"
                placeholder="e.g. stable"
                value={newRepoName}
                onChange={(e) => setNewRepoName(e.target.value)}
              />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="repo-url">URL</Label>
              <Input
                id="repo-url"
                placeholder="https://charts.helm.sh/stable"
                value={newRepoUrl}
                onChange={(e) => setNewRepoUrl(e.target.value)}
              />
            </div>
            {addRepoError && (
              <div className="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                {addRepoError}
              </div>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAddRepoOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={() => void handleAddRepo()}
              disabled={addingRepo || !newRepoName.trim() || !newRepoUrl.trim()}
            >
              {addingRepo ? "Adding…" : "Add"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
