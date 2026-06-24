import { describe, it, expect } from "vitest";
import {
  buildLoginLine,
  encodeData,
  encodeResize,
  encodePing,
} from "../../src/lib/termproxy";

describe("termproxy wire protocol", () => {
  it("builds the login line as user:ticket with trailing newline", () => {
    expect(buildLoginLine("root@pam", "PBS:TICKET")).toBe(
      "root@pam:PBS:TICKET\n"
    );
  });

  it("frames ASCII data with its byte length", () => {
    expect(encodeData("ls\r")).toBe("0:3:ls\r");
  });

  it("uses UTF-8 byte length, not string length, for multibyte data", () => {
    // "é" is 2 bytes in UTF-8 but length 1 as a JS string.
    expect(encodeData("é")).toBe("0:2:é");
  });

  it("frames a resize as 1:cols:rows:", () => {
    expect(encodeResize(120, 40)).toBe("1:120:40:");
  });

  it("encodes a ping as 2", () => {
    expect(encodePing()).toBe("2");
  });
});
