import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  AppStateSnapshot,
  Group,
  GroupInput,
  ImportCandidate,
  ImportResult,
  ServerInput,
  ServerProfile,
  SshKeyInput,
  SshKeyRef,
  Tag,
} from "./types";

const canUseTauri = () =>
  typeof window !== "undefined" && Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

const now = () => new Date().toISOString();
const id = (prefix: string) => `${prefix}_${Math.random().toString(36).slice(2, 10)}`;

function normalizeTagName(name: string) {
  return name.trim().replace(/\s+/g, " ");
}

const mockState: AppStateSnapshot = {
  groups: [
    {
      id: "grp_lab",
      name: "Homelab",
      color: "#3aa675",
      sortOrder: 0,
      createdAt: now(),
      updatedAt: now(),
    },
  ],
  tags: [
    {
      id: "tag_linux",
      name: "linux",
      color: "#4da3ff",
      createdAt: now(),
      updatedAt: now(),
    },
  ],
  sshKeys: [
    {
      id: "key_main",
      label: "Default ed25519",
      path: "~/.ssh/id_ed25519",
      fingerprint: null,
      comment: null,
      createdAt: now(),
      updatedAt: now(),
    },
  ],
  settings: {
    terminalPreference: "auto",
    safetyWarningsEnabled: true,
  },
  servers: [],
};

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (canUseTauri()) {
    return invoke<T>(command, args);
  }

  return mockCall<T>(command, args);
}

async function mockCall<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  await new Promise((resolve) => window.setTimeout(resolve, 80));

  switch (command) {
    case "get_app_state":
      return structuredClone(mockState) as T;
    case "save_server": {
      const input = args?.input as ServerInput;
      const timestamp = now();
      const tags = input.tagNames
        .map(normalizeTagName)
        .filter(Boolean)
        .map((tagName) => {
          const existing = mockState.tags.find((tag) => tag.name.toLowerCase() === tagName.toLowerCase());
          if (existing) {
            return existing;
          }
          const tag: Tag = {
            id: id("tag"),
            name: tagName,
            color: "#4da3ff",
            createdAt: timestamp,
            updatedAt: timestamp,
          };
          mockState.tags.push(tag);
          return tag;
        });
      const serverId = input.id ?? id("srv");
      const server: ServerProfile = {
        id: serverId,
        name: input.name,
        host: input.host,
        port: input.port,
        username: input.username,
        identityFile: input.identityFile ?? null,
        groupId: input.groupId ?? null,
        notes: input.notes,
        tags,
        webLinks: input.webLinks
          .filter((link) => link.label.trim() || link.url.trim())
          .map((link, index) => ({
            id: link.id ?? id("web"),
            serverId,
            label: link.label.trim() || "Web admin",
            url: link.url.trim(),
            sortOrder: index,
          })),
        actions: [
          {
            id: id("act"),
            serverId,
            type: "ssh",
            label: "Open SSH session",
            enabled: true,
            sortOrder: 0,
            config: {},
          },
          {
            id: id("act"),
            serverId,
            type: "copy-command",
            label: "Copy SSH command",
            enabled: true,
            sortOrder: 1,
            config: {},
          },
        ],
        createdAt: timestamp,
        updatedAt: timestamp,
        lastConnectedAt: null,
      };
      const existingIndex = mockState.servers.findIndex((item) => item.id === server.id);
      if (existingIndex >= 0) {
        server.createdAt = mockState.servers[existingIndex].createdAt;
        server.lastConnectedAt = mockState.servers[existingIndex].lastConnectedAt;
        mockState.servers[existingIndex] = server;
      } else {
        mockState.servers.push(server);
      }
      return structuredClone(server) as T;
    }
    case "delete_server": {
      const serverId = args?.id as string;
      mockState.servers = mockState.servers.filter((server) => server.id !== serverId);
      return undefined as T;
    }
    case "save_group": {
      const input = args?.input as GroupInput;
      const timestamp = now();
      const group: Group = {
        id: input.id ?? id("grp"),
        name: input.name,
        color: input.color,
        sortOrder: mockState.groups.length,
        createdAt: timestamp,
        updatedAt: timestamp,
      };
      const existingIndex = mockState.groups.findIndex((item) => item.id === group.id);
      if (existingIndex >= 0) {
        group.createdAt = mockState.groups[existingIndex].createdAt;
        mockState.groups[existingIndex] = group;
      } else {
        mockState.groups.push(group);
      }
      return structuredClone(group) as T;
    }
    case "delete_group": {
      const groupId = args?.id as string;
      mockState.groups = mockState.groups.filter((group) => group.id !== groupId);
      mockState.servers = mockState.servers.map((server) =>
        server.groupId === groupId ? { ...server, groupId: null } : server,
      );
      return undefined as T;
    }
    case "save_ssh_key": {
      const input = args?.input as SshKeyInput;
      const timestamp = now();
      const key: SshKeyRef = {
        id: input.id ?? id("key"),
        label: input.label,
        path: input.path,
        fingerprint: null,
        comment: null,
        createdAt: timestamp,
        updatedAt: timestamp,
      };
      const existingIndex = mockState.sshKeys.findIndex((item) => item.id === key.id);
      if (existingIndex >= 0) {
        key.createdAt = mockState.sshKeys[existingIndex].createdAt;
        mockState.sshKeys[existingIndex] = key;
      } else {
        mockState.sshKeys.push(key);
      }
      return structuredClone(key) as T;
    }
    case "delete_ssh_key": {
      const keyId = args?.id as string;
      mockState.sshKeys = mockState.sshKeys.filter((key) => key.id !== keyId);
      return undefined as T;
    }
    case "save_settings":
      mockState.settings = args?.input as AppSettings;
      return structuredClone(mockState.settings) as T;
    case "get_ssh_command": {
      const server = mockState.servers.find((item) => item.id === args?.serverId);
      return (server ? `ssh ${server.port !== 22 ? `-p ${server.port} ` : ""}${server.username ? `${server.username}@` : ""}${server.host}` : "") as T;
    }
    case "launch_ssh":
    case "open_web_link":
      return undefined as T;
    case "import_ssh_config_preview":
      return [] as T;
    case "import_ssh_config":
      return { imported: 0, skipped: 0, servers: [] } as T;
    default:
      throw new Error(`Unknown mock command: ${command}`);
  }
}

export const api = {
  getAppState: () => call<AppStateSnapshot>("get_app_state"),
  saveServer: (input: ServerInput) => call<ServerProfile>("save_server", { input }),
  deleteServer: (id: string) => call<void>("delete_server", { id }),
  saveGroup: (input: GroupInput) => call<Group>("save_group", { input }),
  deleteGroup: (id: string) => call<void>("delete_group", { id }),
  saveSshKey: (input: SshKeyInput) => call<SshKeyRef>("save_ssh_key", { input }),
  deleteSshKey: (id: string) => call<void>("delete_ssh_key", { id }),
  saveSettings: (input: AppSettings) => call<AppSettings>("save_settings", { input }),
  getSshCommand: (serverId: string) => call<string>("get_ssh_command", { serverId }),
  launchSsh: (serverId: string) => call<void>("launch_ssh", { serverId }),
  openWebLink: (serverId: string, linkId: string) => call<void>("open_web_link", { serverId, linkId }),
  importSshConfigPreview: () => call<ImportCandidate[]>("import_ssh_config_preview"),
  importSshConfig: (aliases: string[]) => call<ImportResult>("import_ssh_config", { aliases }),
};
