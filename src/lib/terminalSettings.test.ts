import { describe, expect, it } from "vitest";
import { terminalPreferenceForTest } from "./terminalSettings";

describe("terminal settings", () => {
  it("tests the selected draft without changing the persisted preference", () => {
    const persisted = { terminalPreference: "konsole", safetyWarningsEnabled: true };
    const draft = { ...persisted, terminalPreference: "alacritty" };

    expect(terminalPreferenceForTest(draft)).toBe("alacritty");
    expect(persisted.terminalPreference).toBe("konsole");
  });
});
