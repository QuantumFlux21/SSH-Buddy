import type { Tunnel, TunnelInput } from "./types";

export const TUNNEL_TYPE_LOCAL = "local";
export const DEFAULT_LOCAL_BIND_HOST = "127.0.0.1";

export interface TunnelFormModel {
  id?: string | null;
  label: string;
  tunnelType: string;
  localBindHost: string;
  localPort: string;
  remoteHost: string;
  remotePort: string;
}

export type TunnelFormErrors = Partial<Record<"label" | "tunnelType" | "localBindHost" | "localPort" | "remoteHost" | "remotePort", string>>;

export const newTunnelDraft = (tunnel?: Tunnel | null): TunnelFormModel => ({
  id: tunnel?.id ?? null,
  label: tunnel?.label ?? "",
  tunnelType: tunnel?.tunnelType ?? TUNNEL_TYPE_LOCAL,
  localBindHost: tunnel?.localBindHost ?? DEFAULT_LOCAL_BIND_HOST,
  localPort: tunnel?.localPort ? String(tunnel.localPort) : "",
  remoteHost: tunnel?.remoteHost ?? "",
  remotePort: tunnel?.remotePort ? String(tunnel.remotePort) : "",
});

export function toTunnelInput(form: TunnelFormModel): TunnelInput {
  return {
    label: form.label.trim(),
    tunnelType: form.tunnelType,
    localBindHost: form.localBindHost.trim() || null,
    localPort: form.localPort.trim() ? Number(form.localPort) : null,
    remoteHost: form.remoteHost.trim() || null,
    remotePort: form.remotePort.trim() ? Number(form.remotePort) : null,
  };
}

export function validateTunnelForm(form: TunnelFormModel): TunnelFormErrors {
  const errors: TunnelFormErrors = {};

  if (!form.label.trim()) {
    errors.label = "Label is required.";
  }

  if (form.tunnelType !== TUNNEL_TYPE_LOCAL) {
    errors.tunnelType = "Only local SSH tunnels are supported right now.";
  }

  const localBindHost = form.localBindHost.trim();
  if (localBindHost) {
    const error = validateHost(localBindHost, "Local bind host");
    if (error) {
      errors.localBindHost = error;
    }
  }

  const localPort = validatePort(form.localPort, "Local port");
  if (localPort) {
    errors.localPort = localPort;
  }

  const remoteHost = form.remoteHost.trim();
  if (!remoteHost) {
    errors.remoteHost = "Remote host is required.";
  } else {
    const error = validateHost(remoteHost, "Remote host");
    if (error) {
      errors.remoteHost = error;
    }
  }

  const remotePort = validatePort(form.remotePort, "Remote port");
  if (remotePort) {
    errors.remotePort = remotePort;
  }

  return errors;
}

export function hasTunnelFormErrors(errors: TunnelFormErrors) {
  return Object.values(errors).some(Boolean);
}

export function tunnelSummary(tunnel: Tunnel) {
  const localBindHost = tunnel.localBindHost ?? DEFAULT_LOCAL_BIND_HOST;
  const localPort = tunnel.localPort ?? "?";
  const remoteHost = tunnel.remoteHost ?? "?";
  const remotePort = tunnel.remotePort ?? "?";

  return `${localBindHost}:${localPort} -> ${remoteHost}:${remotePort}`;
}

function validatePort(value: string, label: string) {
  const port = Number(value);
  if (!value.trim() || !Number.isInteger(port) || port < 1 || port > 65535) {
    return `${label} must be between 1 and 65535.`;
  }

  return null;
}

function validateHost(value: string, label: string) {
  if (/\s/.test(value)) {
    return `${label} must not contain whitespace.`;
  }

  if (value.startsWith("-")) {
    return `${label} must not start with '-'.`;
  }

  if (/[^A-Za-z0-9._-]/.test(value)) {
    return `${label} contains unsupported characters. Use a hostname or IP address.`;
  }

  return null;
}
