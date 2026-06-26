import { describe, it, expect, beforeEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { sendPtyStdinCmd } from "@/lib/tauriCommands";

const mockInvoke = vi.mocked(invoke);

describe("sendPtyStdinCmd", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(undefined);
  });

  it("invokes the send_pty_stdin command", async () => {
    await sendPtyStdinCmd("session-1", "l");
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke.mock.calls[0]?.[0]).toBe("send_pty_stdin");
  });

  it("encodes the data as a UTF-8 byte array, not a raw string", async () => {
    // The backend command signature is `data: Vec<u8>`, so the payload must be
    // a sequence of byte values. Passing a JS string makes serde fail with
    // `invalid type: string "l", expected a sequence`.
    await sendPtyStdinCmd("session-1", "l");
    const payload = mockInvoke.mock.calls[0]?.[1] as {
      sessionId: string;
      data: unknown;
    };
    expect(payload.sessionId).toBe("session-1");
    expect(Array.isArray(payload.data)).toBe(true);
    expect(payload.data).toEqual([108]); // UTF-8 code for 'l'
  });

  it("encodes multi-byte UTF-8 characters correctly", async () => {
    await sendPtyStdinCmd("session-2", "é");
    const payload = mockInvoke.mock.calls[0]?.[1] as { data: number[] };
    // 'é' (U+00E9) encodes to two bytes in UTF-8: 0xC3 0xA9.
    expect(payload.data).toEqual([0xc3, 0xa9]);
  });

  it("encodes control sequences (e.g. carriage return) as bytes", async () => {
    await sendPtyStdinCmd("session-3", "\r");
    const payload = mockInvoke.mock.calls[0]?.[1] as { data: number[] };
    expect(payload.data).toEqual([13]);
  });
});
