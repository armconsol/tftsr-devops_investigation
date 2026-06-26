import { describe, it, expect, vi, beforeEach } from "vitest";

const readText = vi.fn();
const writeText = vi.fn();

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  readText: (...args: unknown[]) => readText(...args),
  writeText: (...args: unknown[]) => writeText(...args),
}));

import {
  readClipboardText,
  writeClipboardText,
} from "../../src/lib/clipboard";

describe("clipboard helper", () => {
  beforeEach(() => {
    readText.mockReset();
    writeText.mockReset();
  });

  it("delegates reads to the clipboard-manager plugin", async () => {
    readText.mockResolvedValue("hello");
    await expect(readClipboardText()).resolves.toBe("hello");
    expect(readText).toHaveBeenCalledOnce();
  });

  it("returns an empty string when the plugin yields null/undefined", async () => {
    readText.mockResolvedValue(null as unknown as string);
    await expect(readClipboardText()).resolves.toBe("");
  });

  it("returns an empty string instead of throwing when a read fails", async () => {
    readText.mockRejectedValue(new Error("denied"));
    await expect(readClipboardText()).resolves.toBe("");
  });

  it("delegates writes to the clipboard-manager plugin", async () => {
    writeText.mockResolvedValue(undefined);
    await writeClipboardText("copied");
    expect(writeText).toHaveBeenCalledWith("copied");
  });

  it("never writes empty text to the clipboard", async () => {
    writeText.mockResolvedValue(undefined);
    await writeClipboardText("");
    expect(writeText).not.toHaveBeenCalled();
  });

  it("swallows write failures rather than rejecting", async () => {
    writeText.mockRejectedValue(new Error("denied"));
    await expect(writeClipboardText("x")).resolves.toBeUndefined();
  });
});
