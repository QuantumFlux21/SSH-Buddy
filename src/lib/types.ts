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
  certificateMode: RdpCertificateMode;
  fullscreen: boolean;
  multiMonitor: boolean;
  monitorIds: string | null;
  width: number | null;
  height: number | null;
  colorDepth: number | null;
  scalingMode: RdpScalingMode;
  scalingPercent: number | null;
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
  certificateMode?: RdpCertificateMode | null;
  fullscreen: boolean;
  multiMonitor: boolean;
  monitorIds?: string | null;
  width?: number | null;
  height?: number | null;
  colorDepth?: number | null;
  scalingMode?: RdpScalingMode | null;
  scalingPercent?: number | null;
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

export interface LaunchBinaryStatus {
  name: string;
  exists: boolean;
}

export interface LaunchDiagnostics {
  actionType: string;
  selectedTerminalOrClient: string | null;
  executable: string | null;
  commandPreview: string;
  argvPreview: string | null;
  keyPath: string | null;
  keyFileExists: boolean | null;
  publicKeyPath: string | null;
  publicKeyFileExists: boolean | null;
  requiredBinaries: LaunchBinaryStatus[];
  backendResult: "spawned" | "preflightFailed" | "spawnFailed" | string;
  message: string;
  freeRdpExecutable: string | null;
  launchedViaTerminal: boolean | null;
  certificateMode: RdpCertificateMode | string | null;
  rdpUsername: string | null;
  rdpDomain: string | null;
  rdpPort: number | null;
  rdpFullscreen: boolean | null;
  rdpWidth: number | null;
  rdpHeight: number | null;
  rdpMultiMonitor: boolean | null;
  rdpMonitorIds: string | null;
  rdpScalingMode: RdpScalingMode | string | null;
  rdpScalingPercent: number | null;
  rdpSmartSizing: boolean | null;
  rdpDynamicResolution: boolean | null;
  targetUsername: string | null;
  targetHost: string | null;
  targetPort: number | null;
  proxyJump: string | null;
}

export interface PingCheck {
  attempted: boolean;
  available: boolean;
  success: boolean;
  packetLossPercent: number | null;
  minMs: number | null;
  avgMs: number | null;
  maxMs: number | null;
  mdevMs: number | null;
  error: string | null;
}

export interface TcpCheck {
  attempted: boolean;
  success: boolean;
  latencyMs: number | null;
  error: string | null;
}

export type ServerStatusState = "unknown" | "online" | "degraded" | "offline" | "checking" | string;

export interface ServerStatus {
  serverId: string;
  state: ServerStatusState;
  checkedAt: string;
  host: string;
  primaryPort: number;
  primaryService: string;
  ping: PingCheck;
  tcp: TcpCheck;
}

export interface PortScanResult {
  port: number;
  label: string;
  state: "open" | "closed" | "timeout" | "error" | string;
  latencyMs: number | null;
  error: string | null;
}

export interface PortScanReport {
  serverId: string;
  host: string;
  scannedAt: string;
  results: PortScanResult[];
  warning: string;
}

export type RdpCertificateMode = "prompt" | "tofu" | "ignore";
export type RdpScalingMode = "native" | "percentage" | "smart-sizing" | "dynamic-resolution";
