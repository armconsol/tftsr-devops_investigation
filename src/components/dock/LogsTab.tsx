import React, { useCallback, useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import { Download, DownloadCloud, Search, Square, Trash2, Play } from "lucide-react";
import { Button, Input } from "@/components/ui";
import {
  streamPodLogsCmd,
  stopLogStreamCmd,
  listPodsCmd,
  getPodLogsCmd,
  saveLogFileCmd,
} from "@/lib/tauriCommands";
import type { PodInfo } from "@/lib/tauriCommands";

export interface LogsTabData {
  clusterId: string;
  namespace: string;
  /** Single-pod mode: the pod to stream. */
  podName?: string;
  /** Single-pod mode: containers belonging to {@link podName}. */
  containers?: string[];
  /**
   * Workload mode: when set, the tab resolves the pods belonging to this
   * workload (by name prefix) and exposes a pod picker. Mutually exclusive with
   * {@link podName}.
   */
  workloadName?: string;
  /** Workload mode: workload kind, used only for labelling. */
  workloadType?: string;
}

interface LogsTabProps {
  data: LogsTabData;
}

const MAX_LINES = 5000;

/**
 * In-dock pod log viewer. Mirrors the structure of LogStreamPanel but renders
 * inline (no Dialog) and at the dock's available height.
 *
 * Supports two modes:
 *  - single-pod: `data.podName` + `data.containers` are provided directly.
 *  - workload:   `data.workloadName` is provided; the tab resolves matching
 *                pods and renders a pod selector (freelens-style).
 */
export function LogsTab({ data }: LogsTabProps) {
  const { clusterId, namespace, workloadName } = data;
  const isWorkloadMode = Boolean(workloadName);

  // ── Workload mode: resolve pods belonging to the workload ──────────────────
  const [pods, setPods] = useState<PodInfo[]>([]);
  const [podsError, setPodsError] = useState<string | null>(null);
  const [selectedPodName, setSelectedPodName] = useState<string>(
    data.podName ?? ""
  );

  useEffect(() => {
    if (!isWorkloadMode) return;
    let cancelled = false;

    (async () => {
      setPodsError(null);
      try {
        const allPods = await listPodsCmd(clusterId, namespace);
        const namePattern = new RegExp(`^${escapeRegExp(workloadName!)}-`);
        const matching = allPods.filter((p) => namePattern.test(p.name));
        if (cancelled) return;
        setPods(matching);
        setSelectedPodName((prev) =>
          prev && matching.some((p) => p.name === prev)
            ? prev
            : (matching[0]?.name ?? "")
        );
      } catch (err) {
        if (!cancelled) {
          setPodsError(err instanceof Error ? err.message : String(err));
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [isWorkloadMode, clusterId, namespace, workloadName]);

  // The effective pod + containers depend on mode.
  const selectedPodData = pods.find((p) => p.name === selectedPodName);
  const podName = isWorkloadMode ? selectedPodName : (data.podName ?? "");
  const containers = isWorkloadMode
    ? (selectedPodData?.containers ?? [])
    : (data.containers ?? []);
  const containersKey = containers.join(",");

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

  // Keep the selected container valid as the pod/containers change. Including
  // selectedContainer is safe: when it is already valid the guard short-circuits
  // without a setState, so this cannot loop.
  useEffect(() => {
    if (!containers.includes(selectedContainer)) {
      setSelectedContainer(containers[0] ?? "");
    }
  }, [podName, containersKey, selectedContainer, containers]);

  useEffect(() => {
    if (follow && streaming && bottomRef.current) {
      bottomRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [lines, follow, streaming]);

  const startStream = async () => {
    if (streaming || !podName || !selectedContainer) return;
    // Defensive: ensure any prior listener/stream is torn down before we open a
    // new one so listeners can never accumulate.
    if (streamIdRef.current || unlistenRef.current) {
      await stopStream();
    }
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

  const handlePodChange = (name: string) => {
    void stopStream();
    setLines([]);
    setSelectedPodName(name);
  };

  const handleDownloadVisible = async () => {
    try {
      const filePath = await save({
        defaultPath: `${podName || "workload"}-${selectedContainer}-visible-logs.txt`,
        filters: [{ name: "Text", extensions: ["txt"] }],
      });
      if (!filePath) return;
      await saveLogFileCmd(
        filePath,
        filteredLines.join("\n"),
        clusterId,
        namespace,
        podName,
        selectedContainer
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDownloadAll = async () => {
    try {
      const filePath = await save({
        defaultPath: `${podName || "workload"}-${selectedContainer}-all-logs.txt`,
        filters: [{ name: "Text", extensions: ["txt"] }],
      });
      if (!filePath) return;
      const { logs } = await getPodLogsCmd(clusterId, namespace, podName, selectedContainer);
      await saveLogFileCmd(filePath, logs, clusterId, namespace, podName, selectedContainer);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleClear = () => setLines([]);

  const filteredLines =
    search.trim() === "" ? lines : lines.filter((l) => l.includes(search));

  return (
    <div className="flex flex-col gap-2 h-full p-3 min-h-0" data-testid="logs-tab">
      <div className="flex flex-wrap items-center gap-2">
        {isWorkloadMode && (
          <select
            aria-label="Pod"
            value={selectedPodName}
            onChange={(e) => handlePodChange(e.target.value)}
            disabled={streaming || pods.length === 0}
            className="h-8 rounded-md border border-input bg-background px-2 py-1 text-xs focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50"
          >
            {pods.length === 0 ? (
              <option value="">No pods found</option>
            ) : (
              pods.map((p) => (
                <option key={p.name} value={p.name}>
                  {p.name} ({p.status})
                </option>
              ))
            )}
          </select>
        )}

        <select
          aria-label="Container"
          value={selectedContainer}
          onChange={(e) => setSelectedContainer(e.target.value)}
          disabled={streaming || containers.length === 0}
          className="h-8 rounded-md border border-input bg-background px-2 py-1 text-xs focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50"
        >
          {containers.length === 0 ? (
            <option value="">No containers</option>
          ) : (
            containers.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))
          )}
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
            <Button
              size="sm"
              onClick={() => void startStream()}
              disabled={!podName || !selectedContainer}
            >
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
            onClick={() => void handleDownloadVisible()}
            disabled={filteredLines.length === 0}
          >
            <Download className="h-3.5 w-3.5 mr-1" />
            Download Visible
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => void handleDownloadAll()}
            disabled={!podName || !selectedContainer}
          >
            <DownloadCloud className="h-3.5 w-3.5 mr-1" />
            Download All
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

      {(error || podsError) && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 px-2 py-1 text-xs text-destructive">
          {error ?? podsError}
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

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
