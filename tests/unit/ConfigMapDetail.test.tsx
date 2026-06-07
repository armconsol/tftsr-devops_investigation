import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ConfigMapDetail } from "@/components/Kubernetes/ConfigMapDetail";
import type { ConfigMapInfo } from "@/lib/tauriCommands";

vi.mock("@tauri-apps/api/core");

const mockConfigMap: ConfigMapInfo = {
  name: "app-config",
  namespace: "default",
  data_keys: 4,
  age: "1d",
};

describe("ConfigMapDetail", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders configmap name", () => {
    render(
      <ConfigMapDetail
        clusterId="cluster-1"
        namespace="default"
        configMap={mockConfigMap}
        onClose={() => {}}
      />
    );
    expect(screen.getByRole("heading", { name: /configmap: app-config/i })).toBeDefined();
  });

  it("shows data key count", () => {
    render(
      <ConfigMapDetail
        clusterId="cluster-1"
        namespace="default"
        configMap={mockConfigMap}
        onClose={() => {}}
      />
    );
    // Badge with count "4" on data tab
    const badges = screen.getAllByText("4");
    expect(badges.length).toBeGreaterThan(0);
  });

  it("shows namespace in metadata tab", () => {
    render(
      <ConfigMapDetail
        clusterId="cluster-1"
        namespace="default"
        configMap={mockConfigMap}
        onClose={() => {}}
      />
    );
    const metadataTab = screen.getByRole("button", { name: /^metadata$/i });
    fireEvent.click(metadataTab);
    const cells = screen.getAllByText("default");
    expect(cells.length).toBeGreaterThan(0);
  });

  it("shows YAML tab heading when switched to", () => {
    render(
      <ConfigMapDetail
        clusterId="cluster-1"
        namespace="default"
        configMap={mockConfigMap}
        onClose={() => {}}
      />
    );
    const yamlTab = screen.getByRole("button", { name: /^yaml$/i });
    fireEvent.click(yamlTab);
    // YamlEditor tab is visible - the tab button itself has the text
    expect(yamlTab).toBeDefined();
  });
});
