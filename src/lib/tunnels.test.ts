import { describe, expect, it } from "vitest";
import { newTunnelDraft, toTunnelInput, tunnelSummary, validateTunnelForm } from "./tunnels";
import type { Tunnel } from "./types";

describe("tunnel form helpers", () => {
  it("validates required local tunnel fields", () => {
    const errors = validateTunnelForm(newTunnelDraft());

    expect(errors).toEqual({
      label: "Label is required.",
      localPort: "Local port must be between 1 and 65535.",
      remoteHost: "Remote host is required.",
      remotePort: "Remote port must be between 1 and 65535.",
    });
  });

  it("validates ports and host fields", () => {
    expect(
      validateTunnelForm({
        ...newTunnelDraft(),
        label: "DB",
        localPort: "0",
        remoteHost: "db.internal",
        remotePort: "5432",
      }).localPort,
    ).toBe("Local port must be between 1 and 65535.");

    expect(
      validateTunnelForm({
        ...newTunnelDraft(),
        label: "DB",
        localPort: "15432",
        remoteHost: "db internal",
        remotePort: "5432",
      }).remoteHost,
    ).toBe("Remote host must not contain whitespace.");

    expect(
      validateTunnelForm({
        ...newTunnelDraft(),
        label: "DB",
        localPort: "15432",
        remoteHost: "db;touch",
        remotePort: "5432",
      }).remoteHost,
    ).toBe("Remote host contains unsupported characters. Use a hostname or IP address.");
  });

  it("normalizes tunnel input", () => {
    const input = toTunnelInput({
      ...newTunnelDraft(),
      label: " DB ",
      localBindHost: " ",
      localPort: "15432",
      remoteHost: " db.internal ",
      remotePort: "5432",
    });

    expect(input).toEqual({
      label: "DB",
      tunnelType: "local",
      localBindHost: null,
      localPort: 15432,
      remoteHost: "db.internal",
      remotePort: 5432,
    });
  });

  it("formats tunnel summaries", () => {
    const tunnel: Tunnel = {
      id: "tun",
      serverProfileId: "srv",
      label: "DB",
      tunnelType: "local",
      localBindHost: "127.0.0.1",
      localPort: 15432,
      remoteHost: "db.internal",
      remotePort: 5432,
      createdAt: "2026-01-01T00:00:00.000Z",
      updatedAt: "2026-01-01T00:00:00.000Z",
    };

    expect(tunnelSummary(tunnel)).toBe("127.0.0.1:15432 -> db.internal:5432");
  });
});
