import { tagInputValue } from "./format";
import type { ServerInput, ServerProfile } from "./types";

export interface ServerFormModel {
  id?: string | null;
  displayName: string;
  host: string;
  port: string;
  username: string;
  identityFileId: string;
  proxyJump: string;
  groupId: string;
  favorite: boolean;
  notes: string;
  tagText: string;
}

export type ServerFormErrors = Partial<Record<"displayName" | "host" | "port" | "proxyJump", string>>;

export const newServerDraft = (server?: ServerProfile | null): ServerFormModel => ({
  id: server?.id ?? null,
  displayName: server?.displayName ?? "",
  host: server?.host ?? "",
  port: String(server?.port ?? 22),
  username: server?.username ?? "",
  identityFileId: server?.identityFileId ?? "",
  proxyJump: server?.proxyJump ?? "",
  groupId: server?.groupId ?? "",
  favorite: server?.favorite ?? false,
  notes: server?.notes ?? "",
  tagText: tagInputValue(server ?? null),
});

export function toServerInput(form: ServerFormModel): ServerInput {
  return {
    displayName: form.displayName.trim(),
    host: form.host.trim(),
    port: Number(form.port),
    username: form.username.trim(),
    identityFileId: form.identityFileId || null,
    proxyJump: form.proxyJump.trim() || null,
    groupId: form.groupId || null,
    notes: form.notes.trim() || null,
    favorite: form.favorite,
    tagNames: tagNamesFromText(form.tagText),
  };
}

export function serverToInput(server: ServerProfile): ServerInput {
  return {
    displayName: server.displayName,
    host: server.host,
    port: server.port,
    username: server.username,
    identityFileId: server.identityFileId,
    proxyJump: server.proxyJump,
    groupId: server.groupId,
    notes: server.notes,
    favorite: server.favorite,
    tagNames: server.tags.map((tag) => tag.name),
  };
}

export function validateServerForm(form: ServerFormModel): ServerFormErrors {
  const errors: ServerFormErrors = {};
  const port = Number(form.port);

  if (!form.displayName.trim()) {
    errors.displayName = "Display name is required.";
  }

  if (!form.host.trim()) {
    errors.host = "Hostname or IP is required.";
  }

  if (!form.port.trim() || !Number.isInteger(port) || port < 1 || port > 65535) {
    errors.port = "Port must be between 1 and 65535.";
  }

  const proxyJump = form.proxyJump.trim();
  if (form.proxyJump.length > 0 && !proxyJump) {
    errors.proxyJump = "ProxyJump cannot be blank.";
  } else if (proxyJump) {
    const proxyJumpError = validateProxyJump(proxyJump);
    if (proxyJumpError) {
      errors.proxyJump = proxyJumpError;
    }
  }

  return errors;
}

export function hasServerFormErrors(errors: ServerFormErrors) {
  return Object.values(errors).some(Boolean);
}

function tagNamesFromText(value: string) {
  const seen = new Set<string>();
  const tags: string[] = [];

  for (const tag of value.split(",")) {
    const normalized = tag.trim().replace(/\s+/g, " ");
    const key = normalized.toLowerCase();
    if (normalized && !seen.has(key)) {
      seen.add(key);
      tags.push(normalized);
    }
  }

  return tags;
}

function validateProxyJump(value: string) {
  if (/\s/.test(value)) {
    return "ProxyJump must not contain whitespace.";
  }

  if (/[^A-Za-z0-9._+\-@:%\[\],]/.test(value)) {
    return "ProxyJump contains unsupported characters. Use OpenSSH host specs like user@bastion:22.";
  }

  for (const entry of value.split(",")) {
    if (!entry) {
      return "ProxyJump entries must not be empty.";
    }

    const parts = entry.split("@");
    if (parts.length > 2) {
      return "ProxyJump entries may contain at most one @.";
    }

    const [first, second] = parts;
    const host = second ?? first;
    if (second !== undefined && !first) {
      return "ProxyJump username must not be empty.";
    }

    if (!host) {
      return "ProxyJump host must not be empty.";
    }

    if (host.startsWith("-")) {
      return "ProxyJump host must not start with '-'.";
    }

    const portError = validateProxyJumpHostPort(host);
    if (portError) {
      return portError;
    }
  }

  return null;
}

function validateProxyJumpHostPort(host: string) {
  if (host.startsWith("[")) {
    const end = host.indexOf("]");
    if (end < 0) {
      return "ProxyJump IPv6 hosts must close ']'.";
    }
    if (end === 1) {
      return "ProxyJump host must not be empty.";
    }
    const suffix = host.slice(end + 1);
    if (suffix && !suffix.startsWith(":")) {
      return "ProxyJump IPv6 host suffix must be a port like :22.";
    }
    if (suffix.startsWith(":")) {
      return validateProxyJumpPort(suffix.slice(1));
    }
    return null;
  }

  const colonCount = [...host].filter((character) => character === ":").length;
  if (colonCount === 1) {
    const [hostPart, portPart] = host.split(":");
    if (!hostPart) {
      return "ProxyJump host must not be empty.";
    }
    return validateProxyJumpPort(portPart);
  }

  return null;
}

function validateProxyJumpPort(port: string) {
  const value = Number(port);
  if (!port || !Number.isInteger(value) || value < 1 || value > 65535) {
    return "ProxyJump port must be between 1 and 65535.";
  }

  return null;
}
