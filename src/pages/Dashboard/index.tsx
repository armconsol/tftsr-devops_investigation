import React, { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Plus, AlertTriangle, CheckCircle, Clock, RefreshCw, X } from "lucide-react";
import { Card, CardHeader, CardTitle, CardContent, Badge, Button } from "@/components/ui";
import { useHistoryStore } from "@/stores/historyStore";
import { updateIssueCmd } from "@/lib/tauriCommands";

export default function Dashboard() {
  const navigate = useNavigate();
  const { issues, loadIssues, isLoading, error } = useHistoryStore();

  useEffect(() => {
    loadIssues();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const openCount = issues.filter((i) => i.status === "open" || i.status === "triaging").length;
  const resolvedThisWeek = issues.filter((i) => {
    if (i.status !== "resolved") return false;
    const created = new Date(i.created_at);
    const oneWeekAgo = new Date();
    oneWeekAgo.setDate(oneWeekAgo.getDate() - 7);
    return created >= oneWeekAgo;
  }).length;
  const criticalCount = issues.filter(
    (i) => (i.severity === "P1" || i.severity === "P2") && i.status !== "resolved"
  ).length;

  const recentIssues = issues.slice(0, 10);
  const statValue = (n: number) => (isLoading ? "—" : String(n));

  return (
    <div className="p-6 space-y-6">
      {/* Welcome header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Dashboard</h1>
          <p className="text-muted-foreground mt-1">
            Troubleshooting and Root Cause Analysis Assistant
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={() => loadIssues()} disabled={isLoading} className="border-border text-foreground bg-card hover:bg-accent">
            <RefreshCw className={`w-4 h-4 mr-2 ${isLoading ? "animate-spin" : ""}`} />
            Refresh
          </Button>
          <Button onClick={() => navigate("/new-issue")}>
            <Plus className="w-4 h-4 mr-2" />
            New Issue
          </Button>
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          Failed to load issues: {error}
        </div>
      )}

      {/* Stat cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Open Issues</CardTitle>
            <Clock className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statValue(openCount)}</div>
            <p className="text-xs text-muted-foreground">Currently active</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Resolved This Week</CardTitle>
            <CheckCircle className="w-4 h-4 text-green-600" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statValue(resolvedThisWeek)}</div>
            <p className="text-xs text-muted-foreground">Last 7 days</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">P1/P2 Critical</CardTitle>
            <AlertTriangle className="w-4 h-4 text-destructive" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statValue(criticalCount)}</div>
            <p className="text-xs text-muted-foreground">Require immediate attention</p>
          </CardContent>
        </Card>
      </div>

      {/* Recent issues */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Recent Issues</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <p className="text-sm text-muted-foreground">Loading...</p>
          ) : recentIssues.length === 0 ? (
            <div className="text-center py-8">
              <p className="text-muted-foreground">No issues yet.</p>
              <Button
                variant="outline"
                className="mt-3"
                onClick={() => navigate("/new-issue")}
              >
                <Plus className="w-4 h-4 mr-2" />
                Create your first issue
              </Button>
            </div>
          ) : (
            <div className="space-y-2">
              {recentIssues.map((issue) => (
                <div
                  key={issue.id}
                  className="flex items-center justify-between rounded-md border p-3 hover:bg-accent cursor-pointer transition-colors"
                  onClick={() => navigate(`/issue/${issue.id}/triage`)}
                >
                  <div className="flex items-center gap-3">
                    <div>
                      <p className="text-sm font-medium">{issue.title}</p>
                      <p className="text-xs text-muted-foreground">
                        {issue.category} | {new Date(issue.created_at).toLocaleDateString()}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant={severityVariant(issue.severity)}>
                      {issue.severity}
                    </Badge>
                    <Badge variant={statusVariant(issue.status)}>
                      {issue.status}
                    </Badge>
                    {issue.status !== "resolved" && (
                      <button
                        title="Close issue"
                        onClick={(e) => {
                          e.stopPropagation();
                          updateIssueCmd(issue.id, { status: "resolved" }).then(() =>
                            loadIssues()
                          );
                        }}
                        className="p-1 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"
                      >
                        <X className="w-3.5 h-3.5" />
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function severityVariant(severity: string): "default" | "destructive" | "secondary" | "outline" {
  switch (severity) {
    case "P1":
      return "destructive";
    case "P2":
      return "default";
    case "P3":
      return "secondary";
    default:
      return "outline";
  }
}

function statusVariant(status: string): "default" | "destructive" | "secondary" | "outline" {
  switch (status) {
    case "open":
      return "default";
    case "triaging":
      return "secondary";
    case "resolved":
      return "outline";
    default:
      return "outline";
  }
}
