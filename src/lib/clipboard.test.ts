import { beforeEach, describe, expect, it, vi } from "vitest";
import { writeText as writeTauriClipboardText } from "@tauri-apps/plugin-clipboard-manager";
import { CLIPBOARD_MANUAL_COPY_MESSAGE, copyTextToClipboard } from "./clipboard";

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn(),
}));

const tauriWriteText = vi.mocked(writeTauriClipboardText);

describe("clipboard helper", () => {
  beforeEach(() => {
    tauriWriteText.mockReset();
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: undefined,
    });
  });

  it("uses the Tauri clipboard plugin in desktop mode", async () => {
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    tauriWriteText.mockResolvedValue(undefined);

    await expect(copyTextToClipboard("ssh nas.local")).resolves.toEqual({ ok: true });
    expect(tauriWriteText).toHaveBeenCalledWith("ssh nas.local");
  });

  it("falls back to navigator.clipboard in browser mode", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });

    await expect(copyTextToClipboard("sftp nas.local")).resolves.toEqual({ ok: true });
    expect(writeText).toHaveBeenCalledWith("sftp nas.local");
    expect(tauriWriteText).not.toHaveBeenCalled();
  });

  it("returns a clean failure when clipboard copy is unavailable", async () => {
    await expect(copyTextToClipboard("ssh nas.local")).resolves.toEqual({
      ok: false,
      error: "Clipboard API is unavailable.",
    });
    expect(CLIPBOARD_MANUAL_COPY_MESSAGE).toContain("copy it manually");
  });
});
