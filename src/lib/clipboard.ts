import { writeText as writeTauriClipboardText } from "@tauri-apps/plugin-clipboard-manager";

export const CLIPBOARD_MANUAL_COPY_MESSAGE =
  "Clipboard copy failed. The command is shown below so you can copy it manually.";

export interface ClipboardCopyResult {
  ok: boolean;
  error?: string;
}

export function canUseTauriClipboard() {
  return typeof window !== "undefined" && Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);
}

export async function copyTextToClipboard(text: string): Promise<ClipboardCopyResult> {
  try {
    if (canUseTauriClipboard()) {
      await writeTauriClipboardText(text);
      return { ok: true };
    }

    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return { ok: true };
    }

    return { ok: false, error: "Clipboard API is unavailable." };
  } catch (cause: unknown) {
    return {
      ok: false,
      error: cause instanceof Error ? cause.message : String(cause),
    };
  }
}
