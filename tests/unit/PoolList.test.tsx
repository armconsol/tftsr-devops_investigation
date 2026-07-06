import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PoolList } from "@/components/Proxmox/PoolList";

const pools = [
  {
    id: "1",
    name: "rbd",
    type: "replicated",
    size: 3,
    minSize: 2,
    used: 66875367424, // ~62.28 GB
    available: 1853503496192, // ~1.69 TB
    total: 1920378863616,
    usedPercent: 3.48,
  },
];

describe("PoolList", () => {
  it("renders Used and Available as human-readable byte values, not raw bytes", () => {
    render(<PoolList pools={pools} />);
    expect(screen.queryByText("66875367424")).toBeNull();
    expect(screen.queryByText("1853503496192")).toBeNull();
    expect(screen.getByText(/GB/)).toBeInTheDocument();
    expect(screen.getByText(/TB/)).toBeInTheDocument();
  });

  it("calls onCreate when New Pool is clicked", async () => {
    const user = userEvent.setup();
    const onCreate = vi.fn();
    render(<PoolList pools={pools} onCreate={onCreate} />);

    await user.click(screen.getByRole("button", { name: /new pool/i }));
    expect(onCreate).toHaveBeenCalledTimes(1);
  });

  it("does not render a non-functional 'More' action button", () => {
    render(<PoolList pools={pools} />);
    expect(screen.queryByTitle("More")).toBeNull();
  });
});
