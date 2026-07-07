import { describe, expect, it } from "vitest";
import {
  newRdpSettingsDraft,
  rdpCertificateModeLabel,
  rdpSettingsSummary,
  toRdpSettingsInput,
  validateRdpSettingsForm,
} from "./rdp";
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
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), scalingMode: "percentage", scalingPercent: "140" }).scalingPercent).toBeUndefined();
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), scalingMode: "percentage", scalingPercent: "" }).scalingPercent).toBe(
      "Scaling percent must be 100, 140, or 180.",
    );
    expect(
      validateRdpSettingsForm({ ...newRdpSettingsDraft(), scalingMode: "percentage", scalingPercent: "125" as "140" }).scalingPercent,
    ).toBe("Scaling percent must be 100, 140, or 180.");
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), multiMonitor: true, monitorIds: "0,1" }).monitorIds).toBeUndefined();
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), multiMonitor: true, monitorIds: "0, 1" }).monitorIds).toBe(
      "Monitor IDs must not contain whitespace.",
    );
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), multiMonitor: true, monitorIds: "0,,1" }).monitorIds).toBe(
      "Monitor IDs must not contain empty entries.",
    );
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), multiMonitor: true, monitorIds: "-1" }).monitorIds).toBe(
      "Monitor IDs must be comma-separated monitor numbers.",
    );
    expect(validateRdpSettingsForm({ ...newRdpSettingsDraft(), multiMonitor: false, monitorIds: "0,1" }).monitorIds).toBe(
      "Monitor IDs require multi-monitor.",
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
      scalingMode: "percentage",
      scalingPercent: "140",
    });

    expect(input).toEqual({
      enabled: true,
      username: "admin",
      domain: null,
      port: 3389,
      certificateMode: "tofu",
      fullscreen: false,
      multiMonitor: false,
      monitorIds: null,
      width: 1280,
      height: 720,
      colorDepth: 24,
      scalingMode: "percentage",
      scalingPercent: 140,
    });
    expect("password" in input).toBe(false);
  });

  it("drops stale scaling percent outside percentage mode", () => {
    const input = toRdpSettingsInput({
      ...newRdpSettingsDraft(),
      scalingMode: "dynamic-resolution",
      scalingPercent: "140",
    });

    expect(input.scalingMode).toBe("dynamic-resolution");
    expect(input.scalingPercent).toBeNull();
  });

  it("formats settings summaries", () => {
    const settings: RdpSettings = {
      serverProfileId: "srv",
      enabled: true,
      username: "admin",
      domain: null,
      port: 3389,
      certificateMode: "ignore",
      fullscreen: false,
      multiMonitor: true,
      monitorIds: "0,1",
      width: 1920,
      height: 1080,
      colorDepth: 32,
      scalingMode: "percentage",
      scalingPercent: 140,
      createdAt: "2026-01-01T00:00:00.000Z",
      updatedAt: "2026-01-01T00:00:00.000Z",
    };

    expect(rdpSettingsSummary(settings)).toBe("1920x1080, scale 140%, multi-monitor, monitors 0,1, 32 bpp");
    expect(rdpCertificateModeLabel(settings.certificateMode)).toContain("less secure");
  });
});
