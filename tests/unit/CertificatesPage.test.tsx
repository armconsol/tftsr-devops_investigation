import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { ProxmoxCertificatesPage } from "@/pages/Proxmox/CertificatesPage";
import { useProxmoxStore } from "@/stores/proxmoxStore";

vi.mock("@tauri-apps/api/core");
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), info: vi.fn() },
  Toaster: () => null,
}));

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

const cluster = { id: "cluster-1", name: "TFTSR", clusterType: "ve" };

const cert = {
  filename: "pve-ssl.pem",
  subject: "CN=pve.example.com",
  san: ["pve.example.com"],
  issuer: "CN=Let's Encrypt",
  notbefore: "2026-01-01",
  notafter: "2026-12-31",
  fingerprint: "AA:BB:CC",
};

function setupInvoke(overrides: Record<string, unknown> = {}) {
  mockInvoke.mockImplementation((cmd: string) => {
    switch (cmd) {
      case "list_proxmox_clusters":
        return Promise.resolve([cluster]);
      case "list_certificates":
        return Promise.resolve([cert]);
      case "list_acme_accounts":
        return Promise.resolve(overrides.acmeAccounts ?? []);
      case "register_acme_account":
        return Promise.resolve({ account_id: "new-account" });
      case "request_acme_certificate":
        return Promise.resolve({ certificate_id: "cert-1", status: "valid" });
      case "upload_certificate":
        return Promise.resolve({ certificate_id: "cert-2", status: "valid" });
      default:
        return Promise.resolve(undefined);
    }
  });
}

beforeEach(() => {
  mockInvoke.mockReset();
  localStorage.clear();
  useProxmoxStore.setState({ selectedClusterId: "", selectedNodeByCluster: {} });
  setupInvoke();
});

describe("CertificatesPage — upload", () => {
  it("invokes upload_certificate with the entered cert/key when Upload is clicked", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCertificatesPage />);
    await waitFor(() => expect(screen.getByRole("button", { name: /upload certificate/i })).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /upload certificate/i }));
    await user.type(
      await screen.findByPlaceholderText("-----BEGIN CERTIFICATE-----"),
      "CERTDATA"
    );
    await user.type(
      screen.getByPlaceholderText("-----BEGIN PRIVATE KEY-----"),
      "KEYDATA"
    );
    await user.click(screen.getByRole("button", { name: /^upload$/i }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("upload_certificate", {
        clusterId: "cluster-1",
        certificate: "CERTDATA",
        privateKey: "KEYDATA",
        name: undefined,
      })
    );
  });

  it("disables Upload until both cert and key are provided", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCertificatesPage />);
    await user.click(await screen.findByRole("button", { name: /upload certificate/i }));

    const uploadButton = screen.getByRole("button", { name: /^upload$/i });
    expect(uploadButton).toBeDisabled();

    await user.type(screen.getByPlaceholderText("-----BEGIN CERTIFICATE-----"), "CERTDATA");
    expect(uploadButton).toBeDisabled();
  });
});

describe("CertificatesPage — ACME ordering", () => {
  it("reuses an existing ACME account when ordering a certificate", async () => {
    setupInvoke({ acmeAccounts: [{ account_id: "acct-1", email: "a@b.com" }] });
    const user = userEvent.setup();
    render(<ProxmoxCertificatesPage />);
    await waitFor(() => expect(screen.getByRole("button", { name: /order via acme/i })).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /order via acme/i }));
    await user.type(await screen.findByPlaceholderText("e.g. pve.example.com"), "new.example.com");
    await user.click(screen.getByRole("button", { name: /order certificate/i }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("request_acme_certificate", {
        clusterId: "cluster-1",
        domain: "new.example.com",
        accountId: "acct-1",
      })
    );
    expect(mockInvoke).not.toHaveBeenCalledWith("register_acme_account", expect.anything());
  });

  it("registers a new ACME account from the email field when none exists", async () => {
    setupInvoke({ acmeAccounts: [] });
    const user = userEvent.setup();
    render(<ProxmoxCertificatesPage />);
    await user.click(await screen.findByRole("button", { name: /order via acme/i }));

    await user.type(screen.getByPlaceholderText("e.g. pve.example.com"), "new.example.com");
    await user.type(screen.getByPlaceholderText("admin@example.com"), "admin@example.com");
    await user.click(screen.getByRole("button", { name: /order certificate/i }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("register_acme_account", {
        clusterId: "cluster-1",
        email: "admin@example.com",
        termsOfServiceAgreed: true,
      })
    );
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("request_acme_certificate", {
        clusterId: "cluster-1",
        domain: "new.example.com",
        accountId: "new-account",
      })
    );
  });
});

describe("CertificatesPage — renew", () => {
  it("requests renewal via ACME using the certificate's own domain", async () => {
    setupInvoke({ acmeAccounts: [{ account_id: "acct-1", email: "a@b.com" }] });
    const user = userEvent.setup();
    render(<ProxmoxCertificatesPage />);
    await waitFor(() => expect(screen.getAllByText("pve.example.com").length).toBeGreaterThan(0));

    await user.click(screen.getByRole("button", { name: /^renew$/i }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("request_acme_certificate", {
        clusterId: "cluster-1",
        domain: "pve.example.com",
        accountId: "acct-1",
      })
    );
  });
});
