import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { OSDList } from "@/components/Proxmox/OSDList";

const osds = [
  {
    id: 19,
    host: "vmhost4",
    status: "up" as const,
    weight: 1.74,
    size: 1920378863616, // ~1.75 TB
    used: 66875367424, // ~62.28 GB
    avail: 1853503496192, // ~1.69 TB
    usedPercent: 3.48,
  },
];

describe("OSDList", () => {
  it("renders Size, Used and Avail as human-readable byte values, not raw bytes", () => {
    render(<OSDList osds={osds} />);
    expect(screen.queryByText("1920378863616")).toBeNull();
    expect(screen.queryByText("66875367424")).toBeNull();
    expect(screen.queryByText("1853503496192")).toBeNull();
    expect(screen.getAllByText(/TB/).length).toBeGreaterThan(0);
    expect(screen.getByText(/GB/)).toBeInTheDocument();
  });
});
