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
  WebLink,
  WebLinkInput,
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
      createdAt: now(),
      updatedAt: now(),
    },
  ],
  tags: [
    {
      id: "tag_linux",
      name: "linux",
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
const mockWebLinks: Record<string, WebLink[]> = {};

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
    case "create_server":
    case "update_server": {
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
            createdAt: timestamp,
            updatedAt: timestamp,
          };
          mockState.tags.push(tag);
          return tag;
        });
      const serverId = command === "update_server" ? (args?.id as string) : id("srv");
      const server: ServerProfile = {
        id: serverId,
        displayName: input.displayName,
        host: input.host,
        port: input.port,
        username: input.username,
        identityFileId: input.identityFileId ?? null,
        groupId: input.groupId ?? null,
        notes: input.notes ?? null,
        favorite: input.favorite,
        tags,
        createdAt: timestamp,
        updatedAt: timestamp,
      };
      const existingIndex = mockState.servers.findIndex((item) => item.id === server.id);
      if (existingIndex >= 0) {
        server.createdAt = mockState.servers[existingIndex].createdAt;
        mockState.servers[existingIndex] = server;
      } else {
        mockState.servers.push(server);
      }
      return structuredClone(server) as T;
    }
    case "delete_server": {
      const serverId = args?.id as string;
      mockState.servers = mockState.servers.filter((server) => server.id !== serverId);
      delete mockWebLinks[serverId];
      return undefined as T;
    }
    case "create_group": {
      const input = args?.input as GroupInput;
      const timestamp = now();
      const group: Group = {
        id: id("grp"),
        name: input.name,
        color: input.color ?? null,
        createdAt: timestamp,
        updatedAt: timestamp,
      };
      mockState.groups.push(group);
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
    case "create_ssh_key_ref": {
      const input = args?.input as SshKeyInput;
      const timestamp = now();
      const key: SshKeyRef = {
        id: id("key"),
        label: input.label,
        path: input.path,
        fingerprint: input.fingerprint ?? null,
        comment: input.comment ?? null,
        createdAt: timestamp,
        updatedAt: timestamp,
      };
      mockState.sshKeys.push(key);
      return structuredClone(key) as T;
    }
    case "delete_ssh_key_ref": {
      const keyId = args?.id as string;
      mockState.sshKeys = mockState.sshKeys.filter((key) => key.id !== keyId);
      return undefined as T;
    }
    case "save_settings":
      mockState.settings = args?.input as AppSettings;
      return structuredClone(mockState.settings) as T;
    case "get_ssh_command": {
      const server = mockState.servers.find((item) => item.id === args?.serverId);
      if (!server) {
        throw new Error("Server not found");
      }
      const key = server.identityFileId ? mockState.sshKeys.find((item) => item.id === server.identityFileId) : null;
      const parts = ["ssh"];
      if (server.port !== 22) {
        parts.push("-p", String(server.port));
      }
      if (key) {
        parts.push("-i", key.path);
      }
      parts.push(`${server.username ? `${server.username}@` : ""}${server.host}`);
      return parts.join(" ") as T;
    }
    case "launch_ssh": {
      const server = mockState.servers.find((item) => item.id === args?.serverId);
      if (!server) {
        throw new Error("Server not found");
      }
      throw new Error("SSH launch requires the Tauri desktop app");
    }
    case "list_web_links": {
      const serverId = args?.serverId as string;
      if (!mockState.servers.some((server) => server.id === serverId)) {
        throw new Error("Server not found");
      }
      return structuredClone(mockWebLinks[serverId] ?? []) as T;
    }
    case "create_web_link": {
      const serverId = args?.serverId as string;
      const input = args?.input as WebLinkInput;
      if (!mockState.servers.some((server) => server.id === serverId)) {
        throw new Error("Server not found");
      }
      const timestamp = now();
      const link: WebLink = {
        id: id("web"),
        serverProfileId: serverId,
        label: input.label,
        url: input.url,
        createdAt: timestamp,
        updatedAt: timestamp,
      };
      mockWebLinks[serverId] = [...(mockWebLinks[serverId] ?? []), link];
      return structuredClone(link) as T;
    }
    case "update_web_link": {
      const linkId = args?.id as string;
      const input = args?.input as WebLinkInput;
      for (const [serverId, links] of Object.entries(mockWebLinks)) {
        const index = links.findIndex((link) => link.id === linkId);
        if (index >= 0) {
          const link: WebLink = {
            ...links[index],
            label: input.label,
            url: input.url,
            updatedAt: now(),
          };
          mockWebLinks[serverId] = links.map((item) => (item.id === linkId ? link : item));
          return structuredClone(link) as T;
        }
      }
      throw new Error("Web link not found");
    }
    case "delete_web_link": {
      const linkId = args?.id as string;
      for (const [serverId, links] of Object.entries(mockWebLinks)) {
        const next = links.filter((link) => link.id !== linkId);
        if (next.length !== links.length) {
          mockWebLinks[serverId] = next;
          return undefined as T;
        }
      }
      throw new Error("Web link not found");
    }
    case "open_web_link": {
      const serverId = args?.serverId as string;
      const linkId = args?.linkId as string;
      const link = (mockWebLinks[serverId] ?? []).find((item) => item.id === linkId);
      if (!link) {
        throw new Error("Web link not found");
      }
      const opened = window.open(link.url, "_blank", "noopener,noreferrer");
      if (!opened) {
        throw new Error("Failed to open web link");
      }
      return undefined as T;
    }
    case "import_ssh_config_preview":
      return [] as T;
    case "import_ssh_config":
      return {
        imported: 0,
        skipped: (args?.aliases as string[] | undefined)?.length ?? 0,
        servers: [],
      } as T;
    default:
      throw new Error(`Unknown mock command: ${command}`);
  }
}

export const api = {
  getAppState: async () => {
    if (!canUseTauri()) {
      return mockCall<AppStateSnapshot>("get_app_state");
    }

    return call<AppStateSnapshot>("get_app_state");
  },
  saveServer: (id: string | null, input: ServerInput) =>
    id ? call<ServerProfile>("update_server", { id, input }) : call<ServerProfile>("create_server", { input }),
  deleteServer: (id: string) => call<void>("delete_server", { id }),
  saveGroup: (input: GroupInput) => call<Group>("create_group", { input }),
  deleteGroup: (id: string) => call<void>("delete_group", { id }),
  saveSshKey: (input: SshKeyInput) => call<SshKeyRef>("create_ssh_key_ref", { input }),
  deleteSshKey: (id: string) => call<void>("delete_ssh_key_ref", { id }),
  saveSettings: (input: AppSettings) => call<AppSettings>("save_settings", { input }),
  getSshCommand: (serverId: string) => call<string>("get_ssh_command", { serverId }),
  launchSsh: (serverId: string) => call<void>("launch_ssh", { serverId }),
  listWebLinks: (serverId: string) => call<WebLink[]>("list_web_links", { serverId }),
  saveWebLink: (serverId: string, id: string | null, input: WebLinkInput) =>
    id ? call<WebLink>("update_web_link", { id, input }) : call<WebLink>("create_web_link", { serverId, input }),
  deleteWebLink: (id: string) => call<void>("delete_web_link", { id }),
  openWebLink: (serverId: string, linkId: string) => call<void>("open_web_link", { serverId, linkId }),
  importSshConfigPreview: () => call<ImportCandidate[]>("import_ssh_config_preview"),
  importSshConfig: (aliases: string[]) => call<ImportResult>("import_ssh_config", { aliases }),
};
