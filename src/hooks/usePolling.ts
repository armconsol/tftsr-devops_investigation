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

    const run = () => {
      void Promise.resolve(fnRef.current()).catch((err) => {
        console.error("usePolling: polled function rejected", err);
      });
    };

    run();
    const id = setInterval(run, intervalMs);

    return () => clearInterval(id);
  }, [enabled, intervalMs]);
}
