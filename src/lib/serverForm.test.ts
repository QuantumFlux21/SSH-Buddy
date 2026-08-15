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

  it("allows OpenSSH aliases and IPv6 destination hosts", () => {
    for (const host of ["prod_web+blue", "192.0.2.10", "2001:db8::10", "[2001:db8::10]", "fe80::1%eth0"]) {
      expect(validateServerForm({ ...newServerDraft(), displayName: "NAS", host, username: "admin" })).toEqual({});
    }
  });

  it("rejects unsafe SSH destination hosts", () => {
    for (const [host, message] of [
      ["-oProxyCommand=touch", "Hostname or IP must not start with '-'."],
      ["nas local", "Hostname or IP must not contain whitespace or control characters."],
      ["nas\nlocal", "Hostname or IP must not contain whitespace or control characters."],
      [`nas${String.fromCharCode(7)}local`, "Hostname or IP must not contain whitespace or control characters."],
      ["admin@nas.local", "Hostname or IP must not contain '@'; set the username separately."],
    ]) {
      expect(validateServerForm({ ...newServerDraft(), displayName: "NAS", host }).host).toBe(message);
    }
  });

  it("rejects unsafe SSH destination usernames", () => {
    for (const [username, message] of [
      ["-Fmalicious-config", "Username must not start with '-'."],
      ["admin user", "Username must not contain whitespace or control characters."],
      [`admin${String.fromCharCode(7)}`, "Username must not contain whitespace or control characters."],
      ["admin@ops", "Username must not contain '@'."],
    ]) {
      expect(validateServerForm({ ...newServerDraft(), displayName: "NAS", host: "nas.local", username }).username).toBe(
        message,
      );
    }
  });

  it("validates proxy jump values", () => {
    expect(
      validateServerForm({
        ...newServerDraft(),
        displayName: "NAS",
        host: "nas.local",
        proxyJump: "user@bastion:22,jump2",
      }),
    ).toEqual({});
    expect(
      validateServerForm({
        ...newServerDraft(),
        displayName: "NAS",
        host: "nas.local",
        proxyJump: " ",
      }).proxyJump,
    ).toBe("ProxyJump cannot be blank.");
    expect(
      validateServerForm({
        ...newServerDraft(),
        displayName: "NAS",
        host: "nas.local",
        proxyJump: "bastion;touch",
      }).proxyJump,
    ).toBe("ProxyJump contains unsupported characters. Use OpenSSH host specs like user@bastion:22.");
    expect(
      validateServerForm({
        ...newServerDraft(),
        displayName: "NAS",
        host: "nas.local",
        proxyJump: "bastion proxy",
      }).proxyJump,
    ).toBe("ProxyJump must not contain whitespace.");
  });

  it("trims input and deduplicates tag names", () => {
    const input = toServerInput({
      ...newServerDraft(),
      displayName: "  NAS  ",
      host: " nas.local ",
      port: "2222",
      username: " admin ",
      proxyJump: " user@bastion:22 ",
      notes: "  local notes  ",
      tagText: "Linux, prod, linux, storage ",
    });

    expect(input).toEqual({
      displayName: "NAS",
      host: "nas.local",
      port: 2222,
      username: "admin",
      identityFileId: null,
      proxyJump: "user@bastion:22",
      groupId: null,
      notes: "local notes",
      favorite: false,
      tagNames: ["Linux", "prod", "storage"],
    });
  });
});
