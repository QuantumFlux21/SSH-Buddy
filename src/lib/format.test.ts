import { describe, expect, it } from "vitest";
import { serverDestination, tagInputValue } from "./format";
import type { ServerProfile } from "./types";

function serverWithTags(tagNames: string[]): ServerProfile {
  return {
    id: "srv_nas",
    displayName: "NAS",
    host: "nas.local",
    port: 22,
    username: "admin",
    identityFileId: null,
    proxyJump: null,
    groupId: null,
    notes: null,
    favorite: false,
    tags: tagNames.map((name, index) => ({
      id: `tag_${index}`,
      name,
      createdAt: "2026-01-01T00:00:00.000Z",
      updatedAt: "2026-01-01T00:00:00.000Z",
    })),
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
  };
}

describe("tagInputValue", () => {
  it("returns an empty value without selected tags", () => {
    expect(tagInputValue(null)).toBe("");
    expect(tagInputValue(serverWithTags([]))).toBe("");
  });

  it("joins tag names in profile order", () => {
    expect(tagInputValue(serverWithTags(["storage", "production"]))).toBe("storage, production");
  });
});

describe("serverDestination", () => {
  it("includes a non-empty username", () => {
    expect(serverDestination(serverWithTags([]))).toBe("admin@nas.local");
  });

  it("omits empty and whitespace-only usernames", () => {
    const server = serverWithTags([]);

    expect(serverDestination({ ...server, username: "" })).toBe("nas.local");
    expect(serverDestination({ ...server, username: "   " })).toBe("nas.local");
  });
});
