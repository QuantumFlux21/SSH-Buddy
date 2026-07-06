import { tagInputValue } from "./format";
import type { ServerInput, ServerProfile } from "./types";

export interface ServerFormModel {
  id?: string | null;
  displayName: string;
  host: string;
  port: string;
  username: string;
  identityFileId: string;
  groupId: string;
  favorite: boolean;
  notes: string;
  tagText: string;
}

export type ServerFormErrors = Partial<Record<"displayName" | "host" | "port", string>>;

export const newServerDraft = (server?: ServerProfile | null): ServerFormModel => ({
  id: server?.id ?? null,
  displayName: server?.displayName ?? "",
  host: server?.host ?? "",
  port: String(server?.port ?? 22),
  username: server?.username ?? "",
  identityFileId: server?.identityFileId ?? "",
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
