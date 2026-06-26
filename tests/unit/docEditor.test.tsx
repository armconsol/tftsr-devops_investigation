import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { DocEditor } from "@/components/DocEditor";

describe("DocEditor Component", () => {
  it("renders export buttons with readable text", () => {
    const mockOnChange = vi.fn();
    const mockOnExport = vi.fn();

    render(
      <DocEditor
        content="# Test Content"
        onChange={mockOnChange}
        onExport={mockOnExport}
      />
    );

    const mdButton = screen.getByRole("button", { name: /MD/i });
    const pdfButton = screen.getByRole("button", { name: /PDF/i });
    const docxButton = screen.getByRole("button", { name: /DOCX/i });

    expect(mdButton).toBeInTheDocument();
    expect(pdfButton).toBeInTheDocument();
    expect(docxButton).toBeInTheDocument();

    // Buttons should have proper variant for visibility
    expect(mdButton.className).toContain("outline");
    expect(pdfButton.className).toContain("outline");
    expect(docxButton.className).toContain("outline");
  });

  it("preview mode shows readable text", () => {
    const mockOnChange = vi.fn();

    render(
      <DocEditor
        content="# Test Heading\n\nTest content"
        onChange={mockOnChange}
      />
    );

    // Switch to preview mode
    const previewButton = screen.getByRole("button", { name: /Preview/i });
    fireEvent.click(previewButton);

    // Preview container should have prose classes for proper contrast
    const previewContainers = document.querySelectorAll(".prose");
    expect(previewContainers.length).toBeGreaterThan(0);

    const mainContainer = previewContainers[0];
    expect(mainContainer.className).toContain("text-foreground");
    expect(mainContainer.className).toContain("dark:prose-invert");
  });
});
