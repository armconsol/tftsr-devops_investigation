import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SecretDetail } from "@/components/Kubernetes/SecretDetail";
import type { SecretInfo } from "@/lib/tauriCommands";

vi.mock("@tauri-apps/api/core");

const mockSecret: SecretInfo = {
  name: "db-credentials",
  namespace: "production",
  type: "Opaque",
  data_keys: 3,
  age: "7d",
};

describe("SecretDetail", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders secret name", () => {
    render(
      <SecretDetail
        clusterId="cluster-1"
        namespace="production"
        secret={mockSecret}
        onClose={() => {}}
      />
    );
    expect(screen.getByRole("heading", { name: /secret: db-credentials/i })).toBeDefined();
  });

  it("shows masked values (*****) by default for all keys", () => {
    render(
      <SecretDetail
        clusterId="cluster-1"
        namespace="production"
        secret={mockSecret}
        onClose={() => {}}
      />
    );
    const masked = screen.getAllByText("*****");
    expect(masked.length).toBe(3);
  });

  it("shows key count (data_keys) in data tab", () => {
    render(
      <SecretDetail
        clusterId="cluster-1"
        namespace="production"
        secret={mockSecret}
        onClose={() => {}}
      />
    );
    expect(screen.getByTestId("secret-key-count")).toBeDefined();
    expect(screen.getByTestId("secret-key-count").textContent).toContain("3");
  });

  it("shows secret type in metadata tab", () => {
    render(
      <SecretDetail
        clusterId="cluster-1"
        namespace="production"
        secret={mockSecret}
        onClose={() => {}}
      />
    );
    const metadataTab = screen.getByRole("button", { name: /^metadata$/i });
    fireEvent.click(metadataTab);
    expect(screen.getByText("Opaque")).toBeDefined();
  });
});
