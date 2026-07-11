const BYTE_UNITS = ["B", "KB", "MB", "GB", "TB", "PB"] as const;

export interface FormatBytesOptions {
  /** Decimal places to round to (default 2). */
  decimals?: number;
}

/**
 * Format a byte count as a human-readable string (B through PB).
 * `undefined`/`null` render as an em dash; negative or NaN values as "0 B".
 */
export function formatBytes(
  bytes: number | undefined | null,
  options: FormatBytesOptions = {}
): string {
  if (bytes === undefined || bytes === null) return "—";
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";

  const decimals = options.decimals ?? 2;
  const exponent = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    BYTE_UNITS.length - 1
  );
  const value = bytes / Math.pow(1024, exponent);
  const rounded = parseFloat(value.toFixed(decimals));
  return `${rounded} ${BYTE_UNITS[exponent]}`;
}

/**
 * Format a duration in seconds as "Xd Yh Zm" (day component omitted when 0).
 * `undefined`/`null` render as an em dash.
 */
export function formatUptime(seconds: number | undefined | null): string {
  if (seconds === undefined || seconds === null || Number.isNaN(seconds)) return "—";
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return d > 0 ? `${d}d ${h}h ${m}m` : `${h}h ${m}m`;
}
