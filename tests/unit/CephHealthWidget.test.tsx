import React from "react";
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { CephHealthWidget } from "@/components/Proxmox/CephHealthWidget";

describe("CephHealthWidget", () => {
  it("renders a healthy status with details", () => {
    render(
      <CephHealthWidget
        health={{
          status: "HEALTH_OK",
          summary: "all good",
          details: ["1 osds down"],
        }}
      />
    );
    expect(screen.getByText("HEALTH_OK")).toBeDefined();
    expect(screen.getByText("all good")).toBeDefined();
    expect(screen.getByText("1 osds down")).toBeDefined();
  });

  // Regression: a partial/malformed payload must not throw during render —
  // an uncaught error here blanked the entire Ceph page after data loaded.
  it("does not crash when details is missing", () => {
    const health = { status: "HEALTH_WARN", summary: "" } as unknown as {
      status: "HEALTH_OK" | "HEALTH_WARN" | "HEALTH_ERR";
      summary: string;
      details: string[];
    };
    expect(() => render(<CephHealthWidget health={health} />)).not.toThrow();
    expect(screen.getByText("HEALTH_WARN")).toBeDefined();
  });

  it("does not crash when health is undefined", () => {
    const health = undefined as unknown as {
      status: "HEALTH_OK" | "HEALTH_WARN" | "HEALTH_ERR";
      summary: string;
      details: string[];
    };
    expect(() => render(<CephHealthWidget health={health} />)).not.toThrow();
    expect(screen.getByText("unknown")).toBeDefined();
  });
});
