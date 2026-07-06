import type { Group, ServerProfile } from "./types";

export function filterServers(
  servers: ServerProfile[],
  groups: Group[],
  query: string,
  groupId: string | null,
) {
  const normalized = query.trim().toLowerCase();
  const groupMap = new Map(groups.map((group) => [group.id, group.name.toLowerCase()]));

  return servers.filter((server) => {
    if (groupId !== null && server.groupId !== groupId) {
      return false;
    }

    if (normalized.length === 0) {
      return true;
    }

    const searchable = [
      server.displayName,
      server.host,
      server.username,
      server.notes ?? "",
      groupMap.get(server.groupId ?? "") ?? "",
      ...server.tags.map((tag) => tag.name),
    ]
      .join(" ")
      .toLowerCase();

    return searchable.includes(normalized);
  });
}

export function groupName(groups: Group[], groupId: string | null) {
  if (!groupId) {
    return "Ungrouped";
  }

  return groups.find((group) => group.id === groupId)?.name ?? "Unknown group";
}
