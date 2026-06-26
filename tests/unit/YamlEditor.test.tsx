import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { YamlEditor } from "@/components/Kubernetes/YamlEditor";

vi.mock("@monaco-editor/react", () => ({
  default: ({
    value,
    onChange,
  }: {
    value?: string;
    onChange?: (v: string | undefined) => void;
  }) => (
    <textarea
      data-testid="monaco-editor"
      value={value ?? ""}
      onChange={(e) => onChange?.(e.target.value)}
      readOnly={false}
    />
  ),
}));

describe("YamlEditor", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing", () => {
    render(<YamlEditor />);
    expect(screen.getByTestId("monaco-editor")).toBeInTheDocument();
  });

  it("renders Monaco editor with initial content", () => {
    const content = "apiVersion: v1\nkind: Pod";
    render(<YamlEditor content={content} />);
    const editor = screen.getByTestId("monaco-editor") as HTMLTextAreaElement;
    expect(editor.value).toBe(content);
  });

  it("Apply button fires onApply with current YAML content", () => {
    const onApply = vi.fn();
    const content = "apiVersion: v1\nkind: Service";
    render(<YamlEditor content={content} showControls onApply={onApply} />);

    const editor = screen.getByTestId("monaco-editor") as HTMLTextAreaElement;
    fireEvent.change(editor, { target: { value: "apiVersion: v1\nkind: Pod" } });

    fireEvent.click(screen.getByRole("button", { name: /apply/i }));
    expect(onApply).toHaveBeenCalledWith("apiVersion: v1\nkind: Pod");
  });

  it("Apply button also fires onChange with YAML content", () => {
    const onChange = vi.fn();
    const content = "apiVersion: v1\nkind: Service";
    render(<YamlEditor content={content} showControls onChange={onChange} />);

    const editor = screen.getByTestId("monaco-editor") as HTMLTextAreaElement;
    fireEvent.change(editor, { target: { value: "new: yaml" } });
    fireEvent.click(screen.getByRole("button", { name: /apply/i }));

    expect(onChange).toHaveBeenCalledWith("new: yaml");
  });

  it("Cancel button fires onCancel callback", () => {
    const onCancel = vi.fn();
    render(<YamlEditor showControls onCancel={onCancel} />);

    fireEvent.click(screen.getByRole("button", { name: /cancel/i }));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("showControls defaults to true — Apply and Cancel are visible", () => {
    render(<YamlEditor />);
    expect(screen.getByRole("button", { name: /apply/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /cancel/i })).toBeInTheDocument();
  });

  it("showControls=false hides Apply and Cancel buttons", () => {
    render(<YamlEditor showControls={false} />);
    expect(screen.queryByRole("button", { name: /apply/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /cancel/i })).toBeNull();
  });

  it("readOnly=true disables the Apply button", () => {
    render(<YamlEditor readOnly showControls />);
    const applyBtn = screen.getByRole("button", { name: /apply/i });
    expect(applyBtn).toBeDisabled();
  });
});
