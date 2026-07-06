export type ConnectionActionType =
  | "ssh"
  | "copy-command"
  | "web"
  | "sftp"
  | "scp"
  | "rdp"
  | "vnc"
  | "tunnel"
  | "wake-on-lan"
  | "custom-command";

export interface Group {
  id: string;
  name: string;
  color: string;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface Tag {
  id: string;
  name: string;
  color: string;
  createdAt: string;
  updatedAt: string;
}

export interface SshKeyRef {
  id: string;
  label: string;
  path: string;
  fingerprint: string | null;
  comment: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface WebLink {
  id: string;
  serverId: string;
  label: string;
  url: string;
  sortOrder: number;
}

export interface ConnectionAction {
  id: string;
  serverId: string;
  type: ConnectionActionType;
  label: string;
  enabled: boolean;
  sortOrder: number;
  config: Record<string, unknown>;
}

export interface ServerProfile {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  identityFile: string | null;
  groupId: string | null;
  notes: string;
  tags: Tag[];
  webLinks: WebLink[];
  actions: ConnectionAction[];
  createdAt: string;
  updatedAt: string;
  lastConnectedAt: string | null;
}

export interface AppSettings {
  terminalPreference: string;
  safetyWarningsEnabled: boolean;
}

export interface AppStateSnapshot {
  servers: ServerProfile[];
  groups: Group[];
  tags: Tag[];
  sshKeys: SshKeyRef[];
  settings: AppSettings;
}

export interface WebLinkInput {
  id?: string | null;
  label: string;
  url: string;
  sortOrder: number;
}

export interface ServerInput {
  id?: string | null;
  name: string;
  host: string;
  port: number;
  username: string;
  identityFile?: string | null;
  groupId?: string | null;
  notes: string;
  tagNames: string[];
  webLinks: WebLinkInput[];
}

export interface GroupInput {
  id?: string | null;
  name: string;
  color: string;
}

export interface SshKeyInput {
  id?: string | null;
  label: string;
  path: string;
}

export interface ImportCandidate {
  alias: string;
  name: string;
  host: string;
  port: number;
  username: string;
  identityFile: string | null;
  warnings: string[];
  selected: boolean;
}

export interface ImportResult {
  imported: number;
  skipped: number;
  servers: ServerProfile[];
}
