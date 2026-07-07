import type { RdpCertificateMode, RdpScalingMode, RdpSettings, RdpSettingsInput } from "./types";

export const RDP_SCALING_PERCENT_OPTIONS = ["100", "140", "180"] as const;
export type RdpScalingPercentOption = (typeof RDP_SCALING_PERCENT_OPTIONS)[number];

export interface RdpSettingsFormModel {
  enabled: boolean;
  username: string;
  domain: string;
  port: string;
  certificateMode: RdpCertificateMode;
  fullscreen: boolean;
  multiMonitor: boolean;
  monitorIds: string;
  width: string;
  height: string;
  colorDepth: "" | "16" | "24" | "32";
  scalingMode: RdpScalingMode;
  scalingPercent: "" | RdpScalingPercentOption;
}

export type RdpFormErrors = Partial<
  Record<"port" | "monitorIds" | "width" | "height" | "colorDepth" | "scalingPercent", string>
>;

export const newRdpSettingsDraft = (settings?: RdpSettings | null): RdpSettingsFormModel => ({
  enabled: settings?.enabled ?? true,
  username: settings?.username ?? "",
  domain: settings?.domain ?? "",
  port: String(settings?.port ?? 3389),
  certificateMode: settings?.certificateMode ?? "tofu",
  fullscreen: settings?.fullscreen ?? false,
  multiMonitor: settings?.multiMonitor ?? false,
  monitorIds: settings?.monitorIds ?? "",
  width: settings?.width ? String(settings.width) : "",
  height: settings?.height ? String(settings.height) : "",
  colorDepth: settings?.colorDepth ? (String(settings.colorDepth) as RdpSettingsFormModel["colorDepth"]) : "",
  scalingMode: settings?.scalingMode ?? "native",
  scalingPercent: settings?.scalingPercent
    ? (String(settings.scalingPercent) as RdpSettingsFormModel["scalingPercent"])
    : "140",
});

export function toRdpSettingsInput(form: RdpSettingsFormModel): RdpSettingsInput {
  return {
    enabled: form.enabled,
    username: form.username.trim() || null,
    domain: form.domain.trim() || null,
    port: parseOptionalNumber(form.port) ?? 3389,
    certificateMode: form.certificateMode,
    fullscreen: form.fullscreen,
    multiMonitor: form.multiMonitor,
    monitorIds: form.monitorIds.trim() || null,
    width: parseOptionalNumber(form.width),
    height: parseOptionalNumber(form.height),
    colorDepth: parseOptionalNumber(form.colorDepth),
    scalingMode: form.scalingMode,
    scalingPercent: form.scalingMode === "percentage" ? parseOptionalNumber(form.scalingPercent) : null,
  };
}

export function validateRdpSettingsForm(form: RdpSettingsFormModel): RdpFormErrors {
  const errors: RdpFormErrors = {};
  const port = parseIntegerField(form.port);
  if (port.empty || port.invalid || !port.value || port.value < 1 || port.value > 65535) {
    errors.port = "RDP port must be between 1 and 65535.";
  }

  const width = parseIntegerField(form.width);
  const height = parseIntegerField(form.height);
  if (width.invalid || (width.value !== null && (width.value < 320 || width.value > 16384))) {
    errors.width = "Width must be between 320 and 16384.";
  }

  if (height.invalid || (height.value !== null && (height.value < 320 || height.value > 16384))) {
    errors.height = "Height must be between 320 and 16384.";
  }

  if (!errors.width && !errors.height && width.empty !== height.empty) {
    errors.width = "Width and height must be set together.";
    errors.height = "Width and height must be set together.";
  }

  if (form.colorDepth && !["16", "24", "32"].includes(form.colorDepth)) {
    errors.colorDepth = "Color depth must be 16, 24, or 32.";
  }

  if (form.scalingMode === "percentage" && !isRdpScalingPercentOption(form.scalingPercent)) {
    errors.scalingPercent = "Scaling percent must be 100, 140, or 180.";
  }

  const monitorIdsError = validateMonitorIds(form.monitorIds, form.multiMonitor);
  if (monitorIdsError) {
    errors.monitorIds = monitorIdsError;
  }

  return errors;
}

export function hasRdpFormErrors(errors: RdpFormErrors) {
  return Object.values(errors).some(Boolean);
}

export function rdpSettingsSummary(settings: RdpSettings) {
  const mode = settings.fullscreen ? "Fullscreen" : settings.width && settings.height ? `${settings.width}x${settings.height}` : "Windowed";
  const extras = [
    rdpScalingSummary(settings),
    settings.multiMonitor ? "multi-monitor" : null,
    settings.monitorIds ? `monitors ${settings.monitorIds}` : null,
    settings.colorDepth ? `${settings.colorDepth} bpp` : null,
  ]
    .filter(Boolean)
    .join(", ");

  return extras ? `${mode}, ${extras}` : mode;
}

export function rdpScalingSummary(settings: Pick<RdpSettings, "scalingMode" | "scalingPercent">) {
  switch (settings.scalingMode) {
    case "percentage":
      return settings.scalingPercent ? `scale ${settings.scalingPercent}%` : "scale percentage";
    case "smart-sizing":
      return "smart sizing";
    case "dynamic-resolution":
      return "dynamic resolution";
    default:
      return null;
  }
}

export function rdpScalingModeLabel(mode: RdpScalingMode | string) {
  switch (mode) {
    case "percentage":
      return "Scale percentage";
    case "smart-sizing":
      return "Smart sizing";
    case "dynamic-resolution":
      return "Dynamic resolution";
    default:
      return "Native / default";
  }
}

export function rdpCertificateModeLabel(mode: RdpCertificateMode | string) {
  switch (mode) {
    case "tofu":
      return "Trust on first use (/cert:tofu)";
    case "ignore":
      return "Ignore certificate (/cert:ignore, less secure)";
    default:
      return "Default / prompt";
  }
}

function validateMonitorIds(value: string, multiMonitor: boolean) {
  if (!value) {
    return null;
  }

  if (value.trim() !== value || /\s/.test(value)) {
    return "Monitor IDs must not contain whitespace.";
  }

  if (!multiMonitor) {
    return "Monitor IDs require multi-monitor.";
  }

  const entries = value.split(",");
  if (entries.some((entry) => !entry)) {
    return "Monitor IDs must not contain empty entries.";
  }

  if (entries.some((entry) => !/^\d+$/.test(entry))) {
    return "Monitor IDs must be comma-separated monitor numbers.";
  }

  return null;
}

function isRdpScalingPercentOption(value: string): value is RdpScalingPercentOption {
  return (RDP_SCALING_PERCENT_OPTIONS as readonly string[]).includes(value);
}

function parseOptionalNumber(value: string) {
  const parsed = parseIntegerField(value);
  if (parsed.empty || parsed.invalid) {
    return null;
  }

  return parsed.value;
}

function parseIntegerField(value: string): { empty: boolean; invalid: boolean; value: number | null } {
  const trimmed = value.trim();
  if (!trimmed) {
    return { empty: true, invalid: false, value: null };
  }

  const parsed = Number(trimmed);
  if (!Number.isInteger(parsed)) {
    return { empty: false, invalid: true, value: null };
  }

  return { empty: false, invalid: false, value: parsed };
}
