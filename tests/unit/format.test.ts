import { describe, it, expect } from "vitest";
import { formatBytes, formatUptime } from "@/lib/format";

describe("formatBytes", () => {
  it("formats zero as 0 B", () => {
    expect(formatBytes(0)).toBe("0 B");
  });

  it("formats bytes below 1KB", () => {
    expect(formatBytes(512)).toBe("512 B");
  });

  it("formats kilobytes", () => {
    expect(formatBytes(2048)).toBe("2 KB");
  });

  it("formats megabytes", () => {
    expect(formatBytes(5 * 1024 * 1024)).toBe("5 MB");
  });

  it("formats gigabytes", () => {
    expect(formatBytes(2.5 * 1024 * 1024 * 1024)).toBe("2.5 GB");
  });

  it("formats terabytes", () => {
    expect(formatBytes(1.2 * 1024 * 1024 * 1024 * 1024)).toBe("1.2 TB");
  });

  it("formats petabytes", () => {
    expect(formatBytes(3 * 1024 * 1024 * 1024 * 1024 * 1024)).toBe("3 PB");
  });

  it("treats undefined/null as em dash", () => {
    expect(formatBytes(undefined)).toBe("—");
    expect(formatBytes(null)).toBe("—");
  });

  it("treats negative or NaN as 0 B", () => {
    expect(formatBytes(-5)).toBe("0 B");
    expect(formatBytes(NaN)).toBe("0 B");
  });

  it("respects a custom decimals option", () => {
    expect(formatBytes(1536, { decimals: 0 })).toBe("2 KB");
    expect(formatBytes(1536, { decimals: 3 })).toBe("1.5 KB");
  });
});

describe("formatUptime", () => {
  it("treats undefined/null as em dash", () => {
    expect(formatUptime(undefined)).toBe("—");
    expect(formatUptime(null)).toBe("—");
  });

  it("formats hours and minutes when under a day", () => {
    expect(formatUptime(3 * 3600 + 25 * 60)).toBe("3h 25m");
  });

  it("formats days, hours and minutes", () => {
    expect(formatUptime(2 * 86400 + 5 * 3600 + 10 * 60)).toBe("2d 5h 10m");
  });

  it("formats zero seconds", () => {
    expect(formatUptime(0)).toBe("0h 0m");
  });
});
