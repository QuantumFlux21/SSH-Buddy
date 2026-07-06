export interface Group {
  id: string;
  name: string;
  color: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface Tag {
  id: string;
  name: string;
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
  serverProfileId: string;
  label: string;
  url: string;
  createdAt: string;
  updatedAt: string;
}

export interface Tunnel {
  id: string;
  serverProfileId: string;
  label: string;
  tunnelType: string;
  localBindHost: string | null;
  localPort: number | null;
  remoteHost: string | null;
  remotePort: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface RdpSettings {
  serverProfileId: string;
  enabled: boolean;
  username: string | null;
  domain: string | null;
  port: number;
  fullscreen: boolean;
  multiMonitor: boolean;
  monitorIds: string | null;
  width: number | null;
  height: number | null;
  colorDepth: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface ServerProfile {
  id: string;
  displayName: string;
  host: string;
  port: number;
  username: string;
  identityFileId: string | null;
  proxyJump: string | null;
  groupId: string | null;
  notes: string | null;
  favorite: boolean;
  tags: Tag[];
  createdAt: string;
  updatedAt: string;
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

export interface ServerInput {
  displayName: string;
  host: string;
  port: number;
  username: string;
  identityFileId?: string | null;
  proxyJump?: string | null;
  groupId?: string | null;
  notes?: string | null;
  favorite: boolean;
  tagNames: string[];
}

export interface GroupInput {
  name: string;
  color?: string | null;
}

export interface SshKeyInput {
  label: string;
  path: string;
  fingerprint?: string | null;
  comment?: string | null;
}

export interface WebLinkInput {
  label: string;
  url: string;
}

export interface TunnelInput {
  label: string;
  tunnelType: string;
  localBindHost?: string | null;
  localPort?: number | null;
  remoteHost?: string | null;
  remotePort?: number | null;
}

export interface RdpSettingsInput {
  enabled: boolean;
  username?: string | null;
  domain?: string | null;
  port?: number | null;
  fullscreen: boolean;
  multiMonitor: boolean;
  monitorIds?: string | null;
  width?: number | null;
  height?: number | null;
  colorDepth?: number | null;
}

export interface ImportCandidate {
  alias: string;
  name: string;
  host: string;
  port: number;
  username: string;
  identityFile: string | null;
  proxyJump: string | null;
  warnings: string[];
  selected: boolean;
  duplicate: boolean;
  skipped: boolean;
}

export interface ImportResult {
  imported: number;
  skipped: number;
  servers: ServerProfile[];
}
