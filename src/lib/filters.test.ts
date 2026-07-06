import { describe, expect, it } from "vitest";
import { filterServers, groupName } from "./filters";
import type { Group, ServerProfile } from "./types";

const groups: Group[] = [
  {
    id: "grp_lab",
    name: "Homelab",
    color: "#42d392",
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
  },
  {
    id: "grp_edge",
    name: "Edge",
    color: "#4da3ff",
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
  },
];

function server(overrides: Partial<ServerProfile>): ServerProfile {
  return {
    id: "srv_default",
    displayName: "NAS",
    host: "nas.local",
    port: 22,
    username: "admin",
    identityFileId: null,
    groupId: "grp_lab",
    notes: null,
    favorite: false,
    tags: [],
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
    ...overrides,
  };
}

describe("filterServers", () => {
  const servers = [
    server({
      id: "srv_nas",
      displayName: "NAS",
      host: "nas.local",
      notes: "Backups and media",
      tags: [{ id: "tag_storage", name: "storage", createdAt: "2026-01-01T00:00:00.000Z", updatedAt: "2026-01-01T00:00:00.000Z" }],
    }),
    server({
      id: "srv_router",
      displayName: "Router",
      host: "10.0.0.1",
      username: "root",
      groupId: "grp_edge",
      tags: [{ id: "tag_network", name: "network", createdAt: "2026-01-01T00:00:00.000Z", updatedAt: "2026-01-01T00:00:00.000Z" }],
    }),
  ];

  it("searches server fields, notes, tags, and group names", () => {
    expect(filterServers(servers, groups, "media", null).map((item) => item.id)).toEqual(["srv_nas"]);
    expect(filterServers(servers, groups, "network", null).map((item) => item.id)).toEqual(["srv_router"]);
    expect(filterServers(servers, groups, "edge", null).map((item) => item.id)).toEqual(["srv_router"]);
    expect(filterServers(servers, groups, "ROOT", null).map((item) => item.id)).toEqual(["srv_router"]);
  });

  it("filters by group before applying text search", () => {
    expect(filterServers(servers, groups, "", "grp_edge").map((item) => item.id)).toEqual(["srv_router"]);
    expect(filterServers(servers, groups, "nas", "grp_edge")).toEqual([]);
  });
});

describe("groupName", () => {
  it("formats missing groups", () => {
    expect(groupName(groups, null)).toBe("Ungrouped");
    expect(groupName(groups, "missing")).toBe("Unknown group");
  });
});
