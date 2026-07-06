import { describe, expect, it } from "vitest";
import { newRdpSettingsDraft, rdpSettingsSummary, toRdpSettingsInput, validateRdpSettingsForm } from "./rdp";
import type { RdpSettings } from "./types";

describe("RDP form helpers", () => {
  it("validates port, dimensions, and color depth", () => {
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), port: "0" }).port).toBe("RDP port must be between 1 and 65535.");
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), width: "1920", height: "" }).height).toBe(
      "Width and height must be set together.",
    );
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), width: "100", height: "1080" }).width).toBe(
      "Width must be between 320 and 16384.",
    );
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), width: "wide", height: "1080" }).width).toBe(
      "Width must be between 320 and 16384.",
    );
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), colorDepth: "8" as "16" }).colorDepth).toBe(
      "Color depth must be 16, 24, or 32.",
    );
  });

  it("converts empty fields without storing passwords", () => {
    const input = toRdpSettingsInput({
      ...newRdpSettingsDraft(),
      username: " admin ",
      domain: " ",
      width: "1280",
      height: "720",
      colorDepth: "24",
    });

    expect(input).toEqual({
      enabled: true,
      username: "admin",
      domain: null,
      port: 3389,
      fullscreen: false,
      multiMonitor: false,
      width: 1280,
      height: 720,
      colorDepth: 24,
    });
    expect("password" in input).toBe(false);
  });

  it("formats settings summaries", () => {
    const settings: RdpSettings = {
      serverProfileId: "srv",
      enabled: true,
      username: "admin",
      domain: null,
      port: 3389,
      fullscreen: false,
      multiMonitor: true,
      width: 1920,
      height: 1080,
      colorDepth: 32,
      createdAt: "2026-01-01T00:00:00.000Z",
      updatedAt: "2026-01-01T00:00:00.000Z",
    };

    expect(rdpSettingsSummary(settings)).toBe("1920x1080, multi-monitor, 32 bpp");
  });
});
