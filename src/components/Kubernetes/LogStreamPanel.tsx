import React, { useCallback, useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Download, Search, Square, Trash2, Play, ChevronUp, ChevronDown, DownloadCloud } from "lucide-react";
import Ansi from "ansi-to-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  Button,
  Input,
} from "@/components/ui";
import { streamPodLogsCmd, stopLogStreamCmd } from "@/lib/tauriCommands";

interface LogStreamPanelProps {
  clusterId: string;
  namespace: string;
  podName: string;
  containers: string[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const MAX_LINES = 5000;

export function LogStreamPanel({
  clusterId,
  namespace,
  podName,
  containers,
  open,
  onOpenChange,
}: LogStreamPanelProps) {
  const [selectedContainer, setSelectedContainer] = useState<string>(
    containers[0] ?? ""
  );
  const [follow, setFollow] = useState(true);
  const [timestamps, setTimestamps] = useState(false);
  const [tailLines, setTailLines] = useState(100);
  const [lines, setLines] = useState<string[]>([]);
  const [streaming, setStreaming] = useState(false);
  const [search, setSearch] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [currentMatchIndex, setCurrentMatchIndex] = useState(0);

  const streamIdRef = useRef<string | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);
  const bottomRef = useRef<HTMLDivElement | null>(null);
  const matchRefs = useRef<(HTMLDivElement | null)[]>([]);

  const stopStream = useCallback(async () => {
    // Critical: Always unlisten FIRST to prevent memory leaks
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
    if (streamIdRef.current) {
      try {
        await stopLogStreamCmd(streamIdRef.current);
      } catch {
        // best-effort
      }
      streamIdRef.current = null;
    }
    setStreaming(false);
  }, []);

  // Cleanup on unmount - use synchronous cleanup for immediate effect
  useEffect(() => {
    return () => {
      // Synchronous cleanup to ensure unlisten is called immediately
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
      // Fire-and-forget cleanup for backend stream
      if (streamIdRef.current) {
        stopLogStreamCmd(streamIdRef.current).catch(() => {
          // best-effort
        });
        streamIdRef.current = null;
      }
    };
  }, []);

  // Stop stream when dialog closes
  useEffect(() => {
    if (!open) {
      void stopStream();
    }
  }, [open, stopStream]);

  useEffect(() => {
    if (follow && streaming && bottomRef.current) {
      bottomRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [lines, follow, streaming]);

  const startStream = async () => {
    if (streaming) return;
    setError(null);
    setLines([]);

    try {
      const streamId = await streamPodLogsCmd({
        cluster_id: clusterId,
        namespace,
        pod_name: podName,
        container_name: selectedContainer,
        follow,
        timestamps,
        tail_lines: tailLines,
      });

      streamIdRef.current = streamId;

      const unlisten = await listen<{ stream_id: string; line: string }>(
        "pod-log-line",
        (event) => {
          if (event.payload.stream_id !== streamId) return;
          setLines((prev) => {
            const next = [...prev, event.payload.line];
            return next.length > MAX_LINES ? next.slice(next.length - MAX_LINES) : next;
          });
        }
      );

      unlistenRef.current = unlisten;
      setStreaming(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDownloadVisible = () => {
    const content = displayLines.join("\n");
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${podName}-${selectedContainer}-visible-logs.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleDownloadAll = async () => {
    try {
      // Fetch all logs from the beginning
      const streamId = await streamPodLogsCmd({
        cluster_id: clusterId,
        namespace,
        pod_name: podName,
        container_name: selectedContainer,
        follow: false,
        timestamps,
        tail_lines: 0, // Get all logs
      });

      const allLines: string[] = [];
      const unlisten = await listen<{ stream_id: string; line: string }>(
        "pod-log-line",
        (event) => {
          if (event.payload.stream_id !== streamId) return;
          allLines.push(event.payload.line);
        }
      );

      // Wait for logs to complete (timeout after 10 seconds)
      await new Promise((resolve) => setTimeout(resolve, 10000));
      unlisten();

      const content = allLines.join("\n");
      const blob = new Blob([content], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${podName}-${selectedContainer}-all-logs.txt`;
      a.click();
      URL.revokeObjectURL(url);

      await stopLogStreamCmd(streamId);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleClear = () => {
    setLines([]);
  };

  const filteredLines =
    search.trim() === "" ? lines : lines.filter((l) => l.includes(search));

  const displayLines = search.trim() !== "" ? filteredLines : lines;

  const matchingLineIndices = search.trim() !== ""
    ? lines.map((line, i) => (line.includes(search) ? i : -1)).filter((i) => i !== -1)
    : [];

  const goToNextMatch = () => {
    if (matchingLineIndices.length === 0) return;
    const nextIndex = (currentMatchIndex + 1) % matchingLineIndices.length;
    setCurrentMatchIndex(nextIndex);
    const lineIndex = matchingLineIndices[nextIndex];
    if (lineIndex !== undefined && matchRefs.current[lineIndex]) {
      matchRefs.current[lineIndex]?.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  };

  const goToPreviousMatch = () => {
    if (matchingLineIndices.length === 0) return;
    const prevIndex = currentMatchIndex === 0
      ? matchingLineIndices.length - 1
      : currentMatchIndex - 1;
    setCurrentMatchIndex(prevIndex);
    const lineIndex = matchingLineIndices[prevIndex];
    if (lineIndex !== undefined && matchRefs.current[lineIndex]) {
      matchRefs.current[lineIndex]?.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  };

  // Reset match index when search changes
  useEffect(() => {
    setCurrentMatchIndex(0);
  }, [search]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-5xl w-full max-h-[80vh]">
        <DialogHeader>
          <DialogTitle>
            Log Stream — {podName}
          </DialogTitle>
        </DialogHeader>

        <div className="flex flex-col gap-3 overflow-hidden" style={{ maxHeight: "calc(80vh - 80px)" }}>
          {/* Controls row */}
          <div className="flex flex-wrap items-center gap-2">
            <select
              value={selectedContainer}
              onChange={(e) => setSelectedContainer(e.target.value)}
              disabled={streaming}
              className="flex h-9 rounded-md border border-input bg-background px-3 py-1 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
            >
              {containers.map((c) => (
                <option key={c} value={c}>
                  {c}
                </option>
              ))}
            </select>

            <label className="flex items-center gap-1.5 text-sm cursor-pointer select-none">
              <input
                type="checkbox"
                className="rounded border-input"
                checked={follow}
                disabled={streaming}
                onChange={(e) => setFollow(e.target.checked)}
              />
              Follow
            </label>

            <label className="flex items-center gap-1.5 text-sm cursor-pointer select-none">
              <input
                type="checkbox"
                className="rounded border-input"
                checked={timestamps}
                disabled={streaming}
                onChange={(e) => setTimestamps(e.target.checked)}
              />
              Timestamps
            </label>

            <div className="flex items-center gap-1.5 text-sm">
              <span className="text-muted-foreground whitespace-nowrap">Tail lines:</span>
              <input
                type="number"
                value={tailLines}
                min={10}
                max={10000}
                disabled={streaming}
                onChange={(e) =>
                  setTailLines(Math.min(10000, Math.max(10, Number(e.target.value))))
                }
                className="flex h-9 w-24 rounded-md border border-input bg-background px-3 py-1 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              />
            </div>

            <div className="flex items-center gap-2 ml-auto">
              {!streaming ? (
                <Button size="sm" onClick={() => void startStream()}>
                  <Play className="h-3.5 w-3.5 mr-1" />
                  Stream
                </Button>
              ) : (
                <Button size="sm" variant="destructive" onClick={() => void stopStream()}>
                  <Square className="h-3.5 w-3.5 mr-1" />
                  Stop
                </Button>
              )}
              <Button size="sm" variant="outline" onClick={handleDownloadVisible} disabled={lines.length === 0}>
                <Download className="h-3.5 w-3.5 mr-1" />
                Download Visible
              </Button>
              <Button size="sm" variant="outline" onClick={() => void handleDownloadAll()} disabled={lines.length === 0}>
                <DownloadCloud className="h-3.5 w-3.5 mr-1" />
                Download All
              </Button>
              <Button size="sm" variant="ghost" onClick={handleClear} disabled={lines.length === 0}>
                <Trash2 className="h-3.5 w-3.5 mr-1" />
                Clear
              </Button>
            </div>
          </div>

          {/* Search bar */}
          <div className="flex items-center gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Filter log lines…"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                className="pl-9"
              />
            </div>
            {search.trim() !== "" && matchingLineIndices.length > 0 && (
              <div className="flex items-center gap-1">
                <span className="text-xs text-muted-foreground whitespace-nowrap">
                  {currentMatchIndex + 1} / {matchingLineIndices.length}
                </span>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={goToPreviousMatch}
                  aria-label="Previous match"
                  className="h-8 w-8 p-0"
                >
                  <ChevronUp className="h-4 w-4" />
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={goToNextMatch}
                  aria-label="Next match"
                  className="h-8 w-8 p-0"
                >
                  <ChevronDown className="h-4 w-4" />
                </Button>
              </div>
            )}
          </div>

          {error && (
            <div className="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
              {error}
            </div>
          )}

          {/* Log output */}
          <div className="flex-1 overflow-y-auto rounded-md border bg-slate-950 p-3 font-mono text-xs text-slate-200 min-h-0">
            {displayLines.length === 0 ? (
              <span className="text-muted-foreground">
                {streaming ? "Waiting for log data…" : "No logs to display. Press Stream to begin."}
              </span>
            ) : (
              <>
                {(search.trim() !== "" ? lines : displayLines).map((line, i) => {
                  const matches = search.trim() !== "" && line.includes(search);
                  const visible = search.trim() === "" || matches;
                  const isCurrentMatch = matches && matchingLineIndices[currentMatchIndex] === i;
                  return (
                    <div
                      key={i}
                      ref={(el) => {
                        if (matches) {
                          matchRefs.current[i] = el;
                        }
                      }}
                      className={[
                        "whitespace-pre-wrap break-all leading-5",
                        !visible ? "opacity-40" : "",
                        isCurrentMatch ? "bg-amber-500/20 border-l-2 border-amber-500 pl-2" : "",
                      ]
                        .filter(Boolean)
                        .join(" ")}
                    >
                      {matches && search.trim() !== "" ? (
                        highlightMatchWithAnsi(line, search)
                      ) : (
                        <Ansi>{line}</Ansi>
                      )}
                    </div>
                  );
                })}
                <div ref={bottomRef} />
              </>
            )}
          </div>

          <div className="text-xs text-muted-foreground">
            {lines.length.toLocaleString()} line{lines.length !== 1 ? "s" : ""}
            {search.trim() !== "" && ` — ${filteredLines.length.toLocaleString()} matching`}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function highlightMatchWithAnsi(line: string, search: string): React.ReactNode {
  const idx = line.indexOf(search);
  if (idx === -1) return <Ansi>{line}</Ansi>;

  return (
    <>
      <Ansi>{line.slice(0, idx)}</Ansi>
      <mark className="bg-amber-400/30 text-amber-200 rounded-sm px-0.5">
        <Ansi>{line.slice(idx, idx + search.length)}</Ansi>
      </mark>
      <Ansi>{line.slice(idx + search.length)}</Ansi>
    </>
  );
}
