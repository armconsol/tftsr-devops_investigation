import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Badge, StatusBadge } from "./Badge";

describe("Badge", () => {
  it("renders with default variant", () => {
    render(<Badge>Test Badge</Badge>);
    expect(screen.getByText("Test Badge")).toBeInTheDocument();
  });

  it("renders with success variant", () => {
    const { container } = render(<Badge variant="success">Success</Badge>);
    expect(screen.getByText("Success")).toBeInTheDocument();
    expect(container.firstChild).toHaveClass("bg-green-500");
  });

  it("renders with destructive variant", () => {
    const { container } = render(<Badge variant="destructive">Error</Badge>);
    expect(screen.getByText("Error")).toBeInTheDocument();
    expect(container.firstChild).toHaveClass("bg-destructive");
  });

  it("renders with icon", () => {
    const icon = <span data-testid="icon">★</span>;
    render(<Badge icon={icon}>With Icon</Badge>);
    expect(screen.getByTestId("icon")).toBeInTheDocument();
    expect(screen.getByText("With Icon")).toBeInTheDocument();
  });

  it("applies custom className", () => {
    const { container } = render(<Badge className="custom-class">Custom</Badge>);
    expect(container.firstChild).toHaveClass("custom-class");
  });
});

describe("StatusBadge", () => {
  it("renders running status with green badge", () => {
    const { container } = render(<StatusBadge status="Running" />);
    expect(screen.getByText("Running")).toBeInTheDocument();
    expect(container.firstChild).toHaveClass("bg-green-500");
  });

  it("renders pending status with yellow badge", () => {
    const { container } = render(<StatusBadge status="Pending" />);
    expect(screen.getByText("Pending")).toBeInTheDocument();
    expect(container.firstChild).toHaveClass("bg-yellow-500");
  });

  it("renders failed status with red badge", () => {
    const { container } = render(<StatusBadge status="Failed" />);
    expect(screen.getByText("Failed")).toBeInTheDocument();
    expect(container.firstChild).toHaveClass("bg-red-500");
  });

  it("renders succeeded status with blue badge", () => {
    const { container } = render(<StatusBadge status="Succeeded" />);
    expect(screen.getByText("Succeeded")).toBeInTheDocument();
    expect(container.firstChild).toHaveClass("bg-blue-500");
  });

  it("renders unknown status with gray badge", () => {
    const { container } = render(<StatusBadge status="Unknown" />);
    expect(screen.getByText("Unknown")).toBeInTheDocument();
    expect(container.firstChild).toHaveClass("bg-gray-500");
  });

  it("handles case-insensitive status matching", () => {
    const { container } = render(<StatusBadge status="RUNNING" />);
    expect(container.firstChild).toHaveClass("bg-green-500");
  });

  it("maps active to running", () => {
    const { container } = render(<StatusBadge status="Active" />);
    expect(container.firstChild).toHaveClass("bg-green-500");
  });

  it("maps error to failed", () => {
    const { container } = render(<StatusBadge status="Error" />);
    expect(container.firstChild).toHaveClass("bg-red-500");
  });

  it("maps completed to succeeded", () => {
    const { container } = render(<StatusBadge status="Completed" />);
    expect(container.firstChild).toHaveClass("bg-blue-500");
  });
});
