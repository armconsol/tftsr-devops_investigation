/**
 * Thin wrapper over `tauri-plugin-clipboard-manager`.
 *
 * The Proxmox consoles run inside the Tauri WebView (WebKitGTK on Linux), where
 * `navigator.clipboard.readText()` is frequently blocked without a trusted user
 * gesture. Routing clipboard access through the Tauri plugin gives reliable
 * read/write and keeps a single, easily-mocked surface for the console
 * components. All operations fail soft: a denied/unavailable clipboard must
 * never crash a live console session.
 */
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";

/** Read the system clipboard as text. Returns "" on empty/denied/unavailable. */
export async function readClipboardText(): Promise<string> {
  try {
    const text = await readText();
    return text ?? "";
  } catch (err) {
    console.warn("Clipboard read failed:", err);
    return "";
  }
}

/** Write text to the system clipboard. No-ops on empty input; never throws. */
export async function writeClipboardText(text: string): Promise<void> {
  if (!text) return;
  try {
    await writeText(text);
  } catch (err) {
    console.warn("Clipboard write failed:", err);
  }
}
