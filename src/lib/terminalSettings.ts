import type { AppSettings } from "./types";

export const TERMINAL_OPTIONS = [
  { value: "auto", label: "Auto detect" },
  { value: "konsole", label: "Konsole" },
  { value: "kitty", label: "kitty" },
  { value: "alacritty", label: "Alacritty" },
  { value: "wezterm", label: "WezTerm" },
  { value: "gnome-terminal", label: "GNOME Terminal" },
  { value: "xterm", label: "xterm" },
] as const;

export function terminalPreferenceForTest(draft: Pick<AppSettings, "terminalPreference">) {
  return draft.terminalPreference;
}
