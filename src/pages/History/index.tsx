import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Search, Download, ExternalLink } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  Button,
  Input,
  Badge,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui";
import { useHistoryStore } from "@/stores/historyStore";
import { DOMAINS } from "@/lib/domainPrompts";

export default function History() {
  const navigate = useNavigate();
  const { issues, isLoading, searchQuery, loadIssues, searchIssues, setSearchQuery } =
    useHistoryStore();

  const [statusFilter, setStatusFilter] = useState("");
  const [domainFilter, setDomainFilter] = useState("");
  const [sortField, setSortField] = useState<"created_at" | "title" | "severity">("created_at");
  const [sortAsc, setSortAsc] = useState(false);

  useEffect(() => {
    loadIssues({
      status: statusFilter || undefined,
      domain: domainFilter || undefined,
    });
  }, [statusFilter, domainFilter, loadIssues]);

  const handleSearch = () => {
    if (searchQuery.trim()) {
      searchIssues(searchQuery.trim());
    } else {
      loadIssues({ status: statusFilter || undefined, domain: domainFilter || undefined });
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleSearch();
  };

  const toggleSort = (field: typeof sortField) => {
    if (sortField === field) {
      setSortAsc(!sortAsc);
    } else {
      setSortField(field);
      setSortAsc(false);
    }
  };

  const sorted = [...issues].sort((a, b) => {
    let cmp = 0;
    switch (sortField) {
      case "title":
        cmp = a.title.localeCompare(b.title);
        break;
      case "severity":
        cmp = a.severity.localeCompare(b.severity);
        break;
      case "created_at":
      default:
        cmp = new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
    }
    return sortAsc ? cmp : -cmp;
  });

  const sortIndicator = (field: typeof sortField) =>
    sortField === field ? (sortAsc ? " ↑" : " ↓") : "";

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-3xl font-bold">History</h1>

      {/* Filters */}
      <div className="flex flex-wrap gap-3">
        <div className="flex-1 min-w-[200px]">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Search issues..."
              className="pl-9"
            />
          </div>
        </div>
        <div className="w-40">
          <Select value={domainFilter} onValueChange={setDomainFilter}>
            <SelectTrigger>
              <SelectValue placeholder="All Domains" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Domains</SelectItem>
              {DOMAINS.map((d) => (
                <SelectItem key={d.id} value={d.id}>
                  {d.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="w-40">
          <Select value={statusFilter} onValueChange={setStatusFilter}>
            <SelectTrigger>
              <SelectValue placeholder="All Statuses" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Statuses</SelectItem>
              <SelectItem value="open">Open</SelectItem>
              <SelectItem value="triaging">Triaging</SelectItem>
              <SelectItem value="resolved">Resolved</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <Button variant="outline" onClick={handleSearch}>
          Search
        </Button>
      </div>

      {/* Table */}
      <Card>
        <CardContent className="p-0">
          {isLoading ? (
            <div className="p-8 text-center text-muted-foreground">Loading...</div>
          ) : sorted.length === 0 ? (
            <div className="p-8 text-center text-muted-foreground">No issues found.</div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b">
                    <th
                      className="text-left text-xs font-medium text-muted-foreground px-4 py-3 cursor-pointer hover:text-foreground"
                      onClick={() => toggleSort("title")}
                    >
                      Title{sortIndicator("title")}
                    </th>
                    <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">
                      Domain
                    </th>
                    <th
                      className="text-left text-xs font-medium text-muted-foreground px-4 py-3 cursor-pointer hover:text-foreground"
                      onClick={() => toggleSort("severity")}
                    >
                      Severity{sortIndicator("severity")}
                    </th>
                    <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">
                      Status
                    </th>
                    <th
                      className="text-left text-xs font-medium text-muted-foreground px-4 py-3 cursor-pointer hover:text-foreground"
                      onClick={() => toggleSort("created_at")}
                    >
                      Created{sortIndicator("created_at")}
                    </th>
                    <th className="text-right text-xs font-medium text-muted-foreground px-4 py-3">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {sorted.map((issue) => (
                    <tr
                      key={issue.id}
                      className="border-b last:border-0 hover:bg-accent/50 transition-colors"
                    >
                      <td className="px-4 py-3 text-sm font-medium">{issue.title}</td>
                      <td className="px-4 py-3 text-sm text-foreground/80 capitalize">
                        {issue.category}
                      </td>
                      <td className="px-4 py-3">
                        <Badge variant={severityVariant(issue.severity)}>
                          {issue.severity}
                        </Badge>
                      </td>
                      <td className="px-4 py-3">
                        <Badge variant={statusVariant(issue.status)}>
                          {issue.status}
                        </Badge>
                      </td>
                      <td className="px-4 py-3 text-sm text-muted-foreground">
                        {new Date(issue.created_at).toLocaleDateString()}
                      </td>
                      <td className="px-4 py-3 text-right">
                        <div className="flex items-center justify-end gap-1">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => navigate(`/issue/${issue.id}/triage`)}
                          >
                            <ExternalLink className="w-3 h-3 mr-1" />
                            Open
                          </Button>
                          <Button variant="ghost" size="sm">
                            <Download className="w-3 h-3 mr-1" />
                            Export
                          </Button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
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
