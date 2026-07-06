import type { ServerProfile } from "./types";

export function shortDate(value: string | null) {
  if (!value) {
    return "Never";
  }

  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(value));
}

export function serverDestination(server: ServerProfile) {
  return server.username.trim().length > 0 ? `${server.username}@${server.host}` : server.host;
}

export function tagInputValue(server: ServerProfile | null) {
  return server?.tags.map((tag) => tag.name).join(", ") ?? "";
}
