import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Search, Download, ExternalLink, FileText, Image, Eye } from "lucide-react";
import {
  Card,
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
import { useAttachmentStore } from "@/stores/attachmentStore";
import { getLogFileContentCmd, getImageAttachmentDataCmd } from "@/lib/tauriCommands";
import { DOMAINS } from "@/lib/domainPrompts";

type TabId = "issues" | "attachments";

export default function History() {
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<TabId>("issues");

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-3xl font-bold">History</h1>

      {/* Tab bar */}
      <div className="flex border-b">
        {(["issues", "attachments"] as TabId[]).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={[
              "px-5 py-2 text-sm font-medium capitalize transition-colors",
              activeTab === tab
                ? "border-b-2 border-primary text-primary"
                : "text-muted-foreground hover:text-foreground",
            ].join(" ")}
          >
            {tab}
          </button>
        ))}
      </div>

      {activeTab === "issues" ? <IssuesTab navigate={navigate} /> : <AttachmentsTab navigate={navigate} />}
    </div>
  );
}

// ─── Issues Tab (unchanged content) ────────────────────────────────────────

function IssuesTab({ navigate }: { navigate: ReturnType<typeof useNavigate> }) {
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
    <>
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
        <Button onClick={handleSearch}>
          <Search className="w-4 h-4 mr-2" />
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
    </>
  );
}

// ─── Attachments Tab ────────────────────────────────────────────────────────

function AttachmentsTab({ navigate }: { navigate: ReturnType<typeof useNavigate> }) {
  const { logFiles, images, isLoading, error, searchQuery, loadAttachments, searchAttachments, setSearchQuery } =
    useAttachmentStore();

  const [viewModal, setViewModal] = useState<{ type: "log" | "image"; id: string; title: string } | null>(null);
  const [modalContent, setModalContent] = useState<string | null>(null);
  const [modalError, setModalError] = useState<string | null>(null);
  const [modalLoading, setModalLoading] = useState(false);

  useEffect(() => {
    loadAttachments();
  }, [loadAttachments]);

  const handleSearch = () => {
    searchAttachments(searchQuery);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleSearch();
  };

  const openLogModal = async (id: string, fileName: string) => {
    setViewModal({ type: "log", id, title: fileName });
    setModalContent(null);
    setModalLoading(true);
    try {
      const content = await getLogFileContentCmd(id);
      setModalContent(content);
    } catch (e) {
      setModalContent(`Error loading content: ${String(e)}`);
    } finally {
      setModalLoading(false);
    }
  };

  const openImageModal = async (id: string, fileName: string) => {
    setViewModal({ type: "image", id, title: fileName });
    setModalContent(null);
    setModalError(null);
    setModalLoading(true);
    try {
      const dataUrl = await getImageAttachmentDataCmd(id);
      setModalContent(dataUrl);
    } catch (e) {
      setModalError(String(e));
    } finally {
      setModalLoading(false);
    }
  };

  const closeModal = () => {
    setViewModal(null);
    setModalContent(null);
    setModalError(null);
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <>
      {/* Search bar */}
      <div className="flex gap-3">
        <div className="flex-1">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Search attachments by filename..."
              className="pl-9"
            />
          </div>
        </div>
        <Button onClick={handleSearch}>
          <Search className="w-4 h-4 mr-2" />
          Search
        </Button>
      </div>

      {isLoading && (
        <div className="p-8 text-center text-muted-foreground">Loading attachments...</div>
      )}
      {error && (
        <div className="p-4 text-sm text-destructive border border-destructive/30 rounded">{error}</div>
      )}

      {!isLoading && (
        <div className="space-y-6">
          {/* Log Files section */}
          <div>
            <h2 className="text-lg font-semibold mb-3 flex items-center gap-2">
              <FileText className="w-5 h-5" />
              Log Files
              <span className="text-sm font-normal text-muted-foreground">({logFiles.length})</span>
            </h2>
            <Card>
              <CardContent className="p-0">
                {logFiles.length === 0 ? (
                  <div className="p-6 text-center text-muted-foreground text-sm">No log files found.</div>
                ) : (
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <thead>
                        <tr className="border-b">
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">File</th>
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">Incident</th>
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">Size</th>
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">Type</th>
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">Uploaded</th>
                          <th className="text-right text-xs font-medium text-muted-foreground px-4 py-3">Actions</th>
                        </tr>
                      </thead>
                      <tbody>
                        {logFiles.map((lf) => (
                          <tr key={lf.id} className="border-b last:border-0 hover:bg-accent/50 transition-colors">
                            <td className="px-4 py-3 text-sm font-medium">
                              {lf.redacted && (
                                <Badge variant="outline" className="mr-2 text-xs">redacted</Badge>
                              )}
                              {lf.file_name}
                            </td>
                            <td className="px-4 py-3 text-sm">
                              <button
                                className="text-primary hover:underline text-left"
                                onClick={() => navigate(`/issue/${lf.issue_id}/triage`)}
                              >
                                {lf.issue_title}
                              </button>
                            </td>
                            <td className="px-4 py-3 text-sm text-muted-foreground">{formatBytes(lf.file_size)}</td>
                            <td className="px-4 py-3 text-xs text-muted-foreground">{lf.mime_type}</td>
                            <td className="px-4 py-3 text-sm text-muted-foreground">
                              {new Date(lf.uploaded_at).toLocaleDateString()}
                            </td>
                            <td className="px-4 py-3 text-right">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => openLogModal(lf.id, lf.file_name)}
                              >
                                <Eye className="w-3 h-3 mr-1" />
                                View
                              </Button>
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

          {/* Images section */}
          <div>
            <h2 className="text-lg font-semibold mb-3 flex items-center gap-2">
              <Image className="w-5 h-5" />
              Images
              <span className="text-sm font-normal text-muted-foreground">({images.length})</span>
            </h2>
            <Card>
              <CardContent className="p-0">
                {images.length === 0 ? (
                  <div className="p-6 text-center text-muted-foreground text-sm">No images found.</div>
                ) : (
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <thead>
                        <tr className="border-b">
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">Preview</th>
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">File</th>
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">Incident</th>
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">Size</th>
                          <th className="text-left text-xs font-medium text-muted-foreground px-4 py-3">Uploaded</th>
                          <th className="text-right text-xs font-medium text-muted-foreground px-4 py-3">Actions</th>
                        </tr>
                      </thead>
                      <tbody>
                        {images.map((img) => (
                          <tr key={img.id} className="border-b last:border-0 hover:bg-accent/50 transition-colors">
                            <td className="px-4 py-3">
                              <ImageThumbnail attachmentId={img.id} alt={img.file_name} />
                            </td>
                            <td className="px-4 py-3 text-sm font-medium">
                              {img.is_paste && (
                                <Badge variant="outline" className="mr-2 text-xs">paste</Badge>
                              )}
                              {img.file_name}
                            </td>
                            <td className="px-4 py-3 text-sm">
                              <button
                                className="text-primary hover:underline text-left"
                                onClick={() => navigate(`/issue/${img.issue_id}/triage`)}
                              >
                                {img.issue_title}
                              </button>
                            </td>
                            <td className="px-4 py-3 text-sm text-muted-foreground">{formatBytes(img.file_size)}</td>
                            <td className="px-4 py-3 text-sm text-muted-foreground">
                              {new Date(img.uploaded_at).toLocaleDateString()}
                            </td>
                            <td className="px-4 py-3 text-right">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => openImageModal(img.id, img.file_name)}
                              >
                                <Eye className="w-3 h-3 mr-1" />
                                View
                              </Button>
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
        </div>
      )}

      {/* Content modal */}
      {viewModal && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
          onClick={closeModal}
        >
          <div
            className="bg-background border rounded-lg shadow-xl w-[90vw] max-w-4xl max-h-[80vh] flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between px-5 py-3 border-b">
              <span className="font-medium text-sm">{viewModal.title}</span>
              <Button variant="ghost" size="sm" onClick={closeModal}>Close</Button>
            </div>
            <div className="flex-1 overflow-auto p-4">
              {modalLoading && (
                <div className="text-center text-muted-foreground py-8">Loading...</div>
              )}
              {!modalLoading && viewModal.type === "log" && (
                <pre className="text-xs font-mono whitespace-pre-wrap break-words leading-relaxed">
                  {modalContent ?? "No content available."}
                </pre>
              )}
              {!modalLoading && viewModal.type === "image" && modalContent && (
                <img
                  src={modalContent}
                  alt={viewModal.title}
                  className="max-w-full max-h-[60vh] object-contain mx-auto"
                />
              )}
              {!modalLoading && viewModal.type === "image" && !modalContent && (
                <div className="text-center py-8 space-y-2">
                  <div className="text-muted-foreground">Image could not be loaded.</div>
                  {modalError && (
                    <div className="text-xs text-destructive font-mono">{modalError}</div>
                  )}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </>
  );
}

// ─── Image thumbnail (lazy-loads on mount) ──────────────────────────────────

function ImageThumbnail({ attachmentId, alt }: { attachmentId: string; alt: string }) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    getImageAttachmentDataCmd(attachmentId)
      .then((data) => { if (!cancelled) setSrc(data); })
      .catch(() => {});
    return () => { cancelled = true; };
  }, [attachmentId]);

  if (!src) {
    return <div className="w-12 h-12 bg-muted rounded flex items-center justify-center text-muted-foreground text-xs">…</div>;
  }
  return (
    <img src={src} alt={alt} className="w-12 h-12 object-cover rounded" />
  );
}

// ─── Helpers ────────────────────────────────────────────────────────────────

function severityVariant(severity: string): "default" | "destructive" | "secondary" | "outline" {
  switch (severity) {
    case "P1": return "destructive";
    case "P2": return "default";
    case "P3": return "secondary";
    default: return "outline";
  }
}

function statusVariant(status: string): "default" | "destructive" | "secondary" | "outline" {
  switch (status) {
    case "open": return "default";
    case "triaging": return "secondary";
    case "resolved": return "outline";
    default: return "outline";
  }
}
