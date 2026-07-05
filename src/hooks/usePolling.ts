import { useEffect, useRef } from "react";

/**
 * Runs `fn` immediately and then on a fixed interval while `enabled` is true.
 * Stops and restarts cleanly whenever `enabled` toggles, and always cleans
 * up its interval on unmount.
 */
export function usePolling(
  fn: () => void | Promise<void>,
  intervalMs: number,
  enabled: boolean
): void {
  const fnRef = useRef(fn);
  useEffect(() => {
    fnRef.current = fn;
  }, [fn]);

  useEffect(() => {
    if (!enabled) return;

    void fnRef.current();
    const id = setInterval(() => {
      void fnRef.current();
    }, intervalMs);

    return () => clearInterval(id);
  }, [enabled, intervalMs]);
}
