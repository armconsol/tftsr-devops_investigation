import { useEffect, useRef, useState, useCallback } from "react";
import { getPodMetricsCmd, type PodMetrics } from "@/lib/tauriCommands";

export interface UseMetricsResult {
  /** Latest pod metrics from kubectl top pods. */
  metrics: PodMetrics[];
  /** True while the initial fetch is in flight. */
  loading: boolean;
  /** Last error message returned from the backend, if any. */
  error: string | null;
  /** Manually trigger a refresh. */
  refresh: () => Promise<void>;
  /** Lookup helper: find metrics for a pod by name. */
  getPodMetrics: (podName: string) => PodMetrics | undefined;
}

const DEFAULT_INTERVAL_MS = 10_000;

/**
 * Subscribe to live pod metrics for a cluster/namespace.
 *
 * Refreshes every {@link intervalMs} milliseconds (default 10s). Automatically
 * cancels the timer on unmount or when the cluster/namespace changes. Errors
 * during a poll are surfaced via {@link UseMetricsResult.error} but do not
 * stop subsequent polls.
 *
 * Pass `null`/`undefined`/empty string for `clusterId` or `namespace` to
 * disable polling (the hook will return an empty list).
 */
export function useMetrics(
  clusterId: string | null | undefined,
  namespace: string | null | undefined,
  intervalMs: number = DEFAULT_INTERVAL_MS
): UseMetricsResult {
  const [metrics, setMetrics] = useState<PodMetrics[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // Track mount state so async fetches that resolve after unmount don't setState.
  const mountedRef = useRef(true);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const enabled = Boolean(clusterId) && Boolean(namespace);

  const fetchMetrics = useCallback(async () => {
    if (!clusterId || !namespace) return;
    try {
      const result = await getPodMetricsCmd(clusterId, namespace);
      if (!mountedRef.current) return;
      setMetrics(result);
      setError(null);
    } catch (err) {
      if (!mountedRef.current) return;
      // Metrics-server may simply be missing - keep previous metrics, surface error.
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      if (mountedRef.current) setLoading(false);
    }
  }, [clusterId, namespace]);

  useEffect(() => {
    mountedRef.current = true;

    // Reset state when inputs change.
    setMetrics([]);
    setError(null);

    if (!enabled) {
      setLoading(false);
      return () => {
        mountedRef.current = false;
      };
    }

    setLoading(true);

    // Kick off an initial fetch immediately.
    void fetchMetrics();

    // Then poll on the configured interval.
    const tick = () => {
      void fetchMetrics().finally(() => {
        if (mountedRef.current) {
          timerRef.current = setTimeout(tick, intervalMs);
        }
      });
    };
    timerRef.current = setTimeout(tick, intervalMs);

    return () => {
      mountedRef.current = false;
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [enabled, fetchMetrics, intervalMs]);

  const getPodMetrics = useCallback(
    (podName: string) => metrics.find((m) => m.name === podName),
    [metrics]
  );

  return {
    metrics,
    loading,
    error,
    refresh: fetchMetrics,
    getPodMetrics,
  };
}

export default useMetrics;
