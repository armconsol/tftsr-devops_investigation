import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SecretDataModal } from "@/components/Kubernetes/SecretDataModal";

describe("SecretDataModal", () => {
  const mockSecretYaml = `apiVersion: v1
kind: Secret
metadata:
  name: test-secret
  namespace: default
type: Opaque
data:
  username: YWRtaW4=
  password: cGFzc3dvcmQxMjM=
  token: dGVzdHRva2VuMTIzNDU=
`;

  const mockOnOpenChange = vi.fn();

  it("renders the secret data modal", () => {
    render(
      <SecretDataModal
        open={true}
        onOpenChange={mockOnOpenChange}
        secretName="test-secret"
        secretYaml={mockSecretYaml}
      />
    );

    expect(screen.getByText(/Secret Data: test-secret/i)).toBeInTheDocument();
  });

  it("displays secret keys in the table", () => {
    render(
      <SecretDataModal
        open={true}
        onOpenChange={mockOnOpenChange}
        secretName="test-secret"
        secretYaml={mockSecretYaml}
      />
    );

    expect(screen.getByText("username")).toBeInTheDocument();
    expect(screen.getByText("password")).toBeInTheDocument();
    expect(screen.getByText("token")).toBeInTheDocument();
  });

  it("initially hides all secret values", () => {
    render(
      <SecretDataModal
        open={true}
        onOpenChange={mockOnOpenChange}
        secretName="test-secret"
        secretYaml={mockSecretYaml}
      />
    );

    const cells = screen.getAllByText("••••••••");
    expect(cells.length).toBeGreaterThanOrEqual(3);
  });

  it("reveals secret value when eye icon is clicked", async () => {
    const user = userEvent.setup();

    render(
      <SecretDataModal
        open={true}
        onOpenChange={mockOnOpenChange}
        secretName="test-secret"
        secretYaml={mockSecretYaml}
      />
    );

    // Find all reveal buttons and click the first one
    const revealButtons = screen.getAllByRole("button", { name: /Reveal value/i });
    await user.click(revealButtons[0]);

    // Check that the decoded value is now visible
    await waitFor(() => {
      expect(screen.getByText("admin")).toBeInTheDocument();
    });
  });

  it("hides secret value when eye-off icon is clicked", async () => {
    const user = userEvent.setup();

    render(
      <SecretDataModal
        open={true}
        onOpenChange={mockOnOpenChange}
        secretName="test-secret"
        secretYaml={mockSecretYaml}
      />
    );

    // Reveal first value
    const revealButtons = screen.getAllByRole("button", { name: /Reveal value/i });
    await user.click(revealButtons[0]);

    await waitFor(() => {
      expect(screen.getByText("admin")).toBeInTheDocument();
    });

    // Hide it again
    const hideButton = screen.getByRole("button", { name: /Hide value/i });
    await user.click(hideButton);

    await waitFor(() => {
      expect(screen.queryByText("admin")).not.toBeInTheDocument();
    });
  });

  it("copies secret value to clipboard when copy icon is clicked", async () => {
    const user = userEvent.setup();
    const mockWriteText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      value: { writeText: mockWriteText },
      writable: true,
      configurable: true,
    });

    render(
      <SecretDataModal
        open={true}
        onOpenChange={mockOnOpenChange}
        secretName="test-secret"
        secretYaml={mockSecretYaml}
      />
    );

    // Find all copy buttons and click the first one
    const copyButtons = screen.getAllByRole("button", { name: /Copy to clipboard/i });
    await user.click(copyButtons[0]);

    await waitFor(() => {
      expect(mockWriteText).toHaveBeenCalledWith("admin");
    });
  });

  it("displays empty state when no data keys exist", () => {
    const emptySecretYaml = `apiVersion: v1
kind: Secret
metadata:
  name: empty-secret
  namespace: default
type: Opaque
data: {}
`;

    render(
      <SecretDataModal
        open={true}
        onOpenChange={mockOnOpenChange}
        secretName="empty-secret"
        secretYaml={emptySecretYaml}
      />
    );

    expect(screen.getByText("No data keys in this secret.")).toBeInTheDocument();
  });

  it("handles malformed base64 gracefully", () => {
    const invalidSecretYaml = `apiVersion: v1
kind: Secret
metadata:
  name: invalid-secret
  namespace: default
type: Opaque
data:
  invalid: !!!not-base64!!!
`;

    render(
      <SecretDataModal
        open={true}
        onOpenChange={mockOnOpenChange}
        secretName="invalid-secret"
        secretYaml={invalidSecretYaml}
      />
    );

    // Should still render without crashing
    expect(screen.getByText(/Secret Data: invalid-secret/i)).toBeInTheDocument();
  });
});
