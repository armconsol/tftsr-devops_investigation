import React, { useCallback, useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Download, Search, Square, Trash2, Play } from "lucide-react";
import { Button, Input } from "@/components/ui";
import { streamPodLogsCmd, stopLogStreamCmd } from "@/lib/tauriCommands";

export interface LogsTabData {
  clusterId: string;
  namespace: string;
  podName: string;
  containers: string[];
}

interface LogsTabProps {
  data: LogsTabData;
}

const MAX_LINES = 5000;

/**
 * In-dock pod log viewer. Mirrors the structure of LogStreamPanel but renders
 * inline (no Dialog) and at the dock's available height.
 */
export function LogsTab({ data }: LogsTabProps) {
  const { clusterId, namespace, podName, containers } = data;

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

  const streamIdRef = useRef<string | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);
  const bottomRef = useRef<HTMLDivElement | null>(null);

  const stopStream = useCallback(async () => {
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

  useEffect(() => {
    return () => {
      void stopStream();
    };
  }, [stopStream]);

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

  const handleDownload = () => {
    const content = lines.join("\n");
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${podName}-${selectedContainer}-logs.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleClear = () => setLines([]);

  const filteredLines =
    search.trim() === "" ? lines : lines.filter((l) => l.includes(search));

  return (
    <div className="flex flex-col gap-2 h-full p-3 min-h-0" data-testid="logs-tab">
      <div className="flex flex-wrap items-center gap-2">
        <select
          aria-label="Container"
          value={selectedContainer}
          onChange={(e) => setSelectedContainer(e.target.value)}
          disabled={streaming}
          className="h-8 rounded-md border border-input bg-background px-2 py-1 text-xs focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50"
        >
          {containers.map((c) => (
            <option key={c} value={c}>
              {c}
            </option>
          ))}
        </select>

        <label className="flex items-center gap-1 text-xs cursor-pointer select-none">
          <input
            type="checkbox"
            className="rounded border-input"
            checked={follow}
            disabled={streaming}
            onChange={(e) => setFollow(e.target.checked)}
          />
          Follow
        </label>

        <label className="flex items-center gap-1 text-xs cursor-pointer select-none">
          <input
            type="checkbox"
            className="rounded border-input"
            checked={timestamps}
            disabled={streaming}
            onChange={(e) => setTimestamps(e.target.checked)}
          />
          Timestamps
        </label>

        <div className="flex items-center gap-1 text-xs">
          <span className="text-muted-foreground whitespace-nowrap">Tail:</span>
          <input
            type="number"
            value={tailLines}
            min={10}
            max={10000}
            disabled={streaming}
            onChange={(e) =>
              setTailLines(Math.min(10000, Math.max(10, Number(e.target.value))))
            }
            className="h-8 w-20 rounded-md border border-input bg-background px-2 py-1 text-xs focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50"
          />
        </div>

        <div className="flex items-center gap-1 ml-auto">
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
          <Button
            size="sm"
            variant="outline"
            onClick={handleDownload}
            disabled={lines.length === 0}
          >
            <Download className="h-3.5 w-3.5" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={handleClear}
            disabled={lines.length === 0}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      <div className="relative">
        <Search className="absolute left-2 top-2 h-3.5 w-3.5 text-muted-foreground" />
        <Input
          placeholder="Filter log lines..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="pl-7 h-8 text-xs"
        />
      </div>

      {error && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 px-2 py-1 text-xs text-destructive">
          {error}
        </div>
      )}

      <div className="flex-1 overflow-y-auto rounded-md border bg-slate-950 p-2 font-mono text-xs text-slate-200 min-h-0">
        {filteredLines.length === 0 ? (
          <span className="text-muted-foreground">
            {streaming ? "Waiting for log data..." : "No logs to display. Press Stream to begin."}
          </span>
        ) : (
          <>
            {filteredLines.map((line, i) => (
              <div key={i} className="whitespace-pre-wrap break-all leading-5">
                {line}
              </div>
            ))}
            <div ref={bottomRef} />
          </>
        )}
      </div>

      <div className="text-xs text-muted-foreground">
        {lines.length.toLocaleString()} line{lines.length !== 1 ? "s" : ""}
        {search.trim() !== "" && ` — ${filteredLines.length.toLocaleString()} matching`}
      </div>
    </div>
  );
}
