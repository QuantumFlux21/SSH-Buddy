import { describe, expect, it } from "vitest";
import { newServerDraft, toServerInput, validateServerForm } from "./serverForm";

describe("server form helpers", () => {
  it("validates required server fields", () => {
    const errors = validateServerForm({
      ...newServerDraft(),
      displayName: " ",
      host: "",
      port: "",
    });

    expect(errors).toEqual({
      displayName: "Display name is required.",
      host: "Hostname or IP is required.",
      port: "Port must be between 1 and 65535.",
    });
  });

  it("validates port range", () => {
    expect(validateServerForm({ ...newServerDraft(), displayName: "NAS", host: "nas.local", port: "0" }).port).toBe(
      "Port must be between 1 and 65535.",
    );
    expect(validateServerForm({ ...newServerDraft(), displayName: "NAS", host: "nas.local", port: "65536" }).port).toBe(
      "Port must be between 1 and 65535.",
    );
    expect(validateServerForm({ ...newServerDraft(), displayName: "NAS", host: "nas.local", port: "22" })).toEqual({});
  });

  it("trims input and deduplicates tag names", () => {
    const input = toServerInput({
      ...newServerDraft(),
      displayName: "  NAS  ",
      host: " nas.local ",
      port: "2222",
      username: " admin ",
      notes: "  local notes  ",
      tagText: "Linux, prod, linux, storage ",
    });

    expect(input).toEqual({
      displayName: "NAS",
      host: "nas.local",
      port: 2222,
      username: "admin",
      identityFileId: null,
      groupId: null,
      notes: "local notes",
      favorite: false,
      tagNames: ["Linux", "prod", "storage"],
    });
  });
});
