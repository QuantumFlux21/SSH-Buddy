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

export interface ServerProfile {
  id: string;
  displayName: string;
  host: string;
  port: number;
  username: string;
  identityFileId: string | null;
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
