import { useEffect, useMemo, useState } from "react";
import {
  Cable,
  Copy,
  Download,
  ExternalLink,
  Folder,
  FolderOpen,
  KeyRound,
  Monitor,
  Pencil,
  Plus,
  Search,
  Server,
  Settings,
  ShieldAlert,
  Star,
  Tag,
  Terminal,
  Trash2,
  X,
} from "lucide-react";
import { api } from "./lib/api";
import { filterServers, groupName } from "./lib/filters";
import { serverDestination, shortDate } from "./lib/format";
import { formatImportPreviewSummary, formatImportResult } from "./lib/importSummary";
import {
  hasRdpFormErrors,
  newRdpSettingsDraft,
  rdpSettingsSummary,
  toRdpSettingsInput,
  validateRdpSettingsForm,
  type RdpSettingsFormModel,
} from "./lib/rdp";
import {
  hasServerFormErrors,
  newServerDraft,
  serverToInput,
  toServerInput,
  validateServerForm,
  type ServerFormModel,
} from "./lib/serverForm";
import {
  hasTunnelFormErrors,
  newTunnelDraft,
  toTunnelInput,
  tunnelSummary,
  validateTunnelForm,
  type TunnelFormModel,
} from "./lib/tunnels";
import {
  hasWebLinkFormErrors,
  newWebLinkDraft,
  toWebLinkInput,
  validateWebLinkForm,
  type WebLinkFormModel,
} from "./lib/webLinks";
import type {
  AppSettings,
  AppStateSnapshot,
  Group,
  GroupInput,
  ImportCandidate,
  ImportResult,
  RdpSettings,
  ServerProfile,
  SshKeyInput,
  SshKeyRef,
  Tunnel,
  WebLink,
} from "./lib/types";

type Section = "servers" | "groups" | "keys" | "settings";

export default function App() {
  const [snapshot, setSnapshot] = useState<AppStateSnapshot | null>(null);
  const [activeSection, setActiveSection] = useState<Section>("servers");
  const [query, setQuery] = useState("");
  const [groupFilter, setGroupFilter] = useState<string | null>(null);
  const [selectedServerId, setSelectedServerId] = useState<string | null>(null);
  const [editingServer, setEditingServer] = useState<ServerFormModel | null>(null);
  const [webLinks, setWebLinks] = useState<WebLink[]>([]);
  const [webLinksLoading, setWebLinksLoading] = useState(false);
  const [editingWebLink, setEditingWebLink] = useState<WebLinkFormModel | null>(null);
  const [tunnels, setTunnels] = useState<Tunnel[]>([]);
  const [tunnelsLoading, setTunnelsLoading] = useState(false);
  const [editingTunnel, setEditingTunnel] = useState<TunnelFormModel | null>(null);
  const [rdpSettings, setRdpSettings] = useState<RdpSettings | null>(null);
  const [rdpLoading, setRdpLoading] = useState(false);
  const [editingRdpSettings, setEditingRdpSettings] = useState<RdpSettingsFormModel | null>(null);
  const [serverPendingDelete, setServerPendingDelete] = useState<ServerProfile | null>(null);
  const [showImport, setShowImport] = useState(false);
  const [busyMessage, setBusyMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  async function loadState(nextSelectedId?: string | null) {
    setError(null);
    const next = await api.getAppState();
    setSnapshot(next);
    setSelectedServerId((currentId) => {
      if (nextSelectedId !== undefined) {
        return nextSelectedId;
      }

      if (currentId && next.servers.some((server) => server.id === currentId)) {
        return currentId;
      }

      return next.servers[0]?.id ?? null;
    });
  }

  useEffect(() => {
    loadState().catch((cause: unknown) => {
      setError(cause instanceof Error ? cause.message : String(cause));
    });
  }, []);

  const filteredServers = useMemo(() => {
    if (!snapshot) {
      return [];
    }
    return filterServers(snapshot.servers, snapshot.groups, query, groupFilter);
  }, [snapshot, query, groupFilter]);

  const selectedServer = useMemo(() => {
    return filteredServers.find((server) => server.id === selectedServerId) ?? filteredServers[0] ?? null;
  }, [selectedServerId, filteredServers]);

  useEffect(() => {
    if (filteredServers.length === 0) {
      setSelectedServerId(null);
      return;
    }

    if (!selectedServerId || !filteredServers.some((server) => server.id === selectedServerId)) {
      setSelectedServerId(filteredServers[0].id);
    }
  }, [filteredServers, selectedServerId]);

  useEffect(() => {
    if (!selectedServer || activeSection !== "servers") {
      setWebLinks([]);
      setEditingWebLink(null);
      setWebLinksLoading(false);
      setTunnels([]);
      setEditingTunnel(null);
      setTunnelsLoading(false);
      setRdpSettings(null);
      setEditingRdpSettings(null);
      setRdpLoading(false);
      return;
    }

    let cancelled = false;
    setWebLinksLoading(true);
    setTunnelsLoading(true);
    setRdpLoading(true);
    setEditingWebLink(null);
    setEditingTunnel(null);
    setEditingRdpSettings(null);

    Promise.all([api.listWebLinks(selectedServer.id), api.listTunnels(selectedServer.id), api.getRdpSettings(selectedServer.id)])
      .then(([links, nextTunnels, nextRdpSettings]) => {
        if (!cancelled) {
          setWebLinks(links);
          setTunnels(nextTunnels);
          setRdpSettings(nextRdpSettings);
        }
      })
      .catch((cause: unknown) => {
        if (!cancelled) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setWebLinksLoading(false);
          setTunnelsLoading(false);
          setRdpLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [activeSection, selectedServer?.id]);

  async function runAction(label: string, action: () => Promise<void>, successMessage?: string) {
    setBusyMessage(label);
    setError(null);
    setStatusMessage(null);
    try {
      await action();
      if (successMessage) {
        setStatusMessage(successMessage);
      }
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusyMessage(null);
    }
  }

  async function saveServer(form: ServerFormModel) {
    const errors = validateServerForm(form);
    if (hasServerFormErrors(errors)) {
      setStatusMessage(null);
      setError(Object.values(errors).find(Boolean) ?? "Fix the highlighted server fields.");
      return;
    }

    await runAction(
      "Saving server",
      async () => {
        const server = await api.saveServer(form.id ?? null, toServerInput(form));
        setEditingServer(null);
        await loadState(server.id);
      },
      form.id ? "Server updated." : "Server created.",
    );
  }

  async function confirmDeleteServer() {
    if (!serverPendingDelete) {
      return;
    }

    const server = serverPendingDelete;
    await runAction(
      "Deleting server",
      async () => {
        await api.deleteServer(server.id);
        setServerPendingDelete(null);
        await loadState();
      },
      "Server deleted.",
    );
  }

  async function copySshCommand(server: ServerProfile) {
    await runAction(
      "Copying SSH command",
      async () => {
        const command = await api.getSshCommand(server.id);
        await navigator.clipboard.writeText(command);
      },
      "SSH command copied to clipboard.",
    );
  }

  async function launchSsh(server: ServerProfile) {
    await runAction(
      "Launching SSH",
      async () => {
        await api.launchSsh(server.id);
      },
      "SSH launch requested in your external terminal.",
    );
  }

  async function copySftpCommand(server: ServerProfile) {
    await runAction(
      "Copying SFTP command",
      async () => {
        const command = await api.getSftpCommand(server.id);
        await navigator.clipboard.writeText(command);
      },
      "SFTP command copied to clipboard.",
    );
  }

  async function launchSftp(server: ServerProfile) {
    await runAction(
      "Launching SFTP",
      async () => {
        await api.launchSftp(server.id);
      },
      "SFTP launch requested in your external terminal.",
    );
  }

  async function toggleFavorite(server: ServerProfile) {
    await runAction(
      server.favorite ? "Removing favorite" : "Marking favorite",
      async () => {
        const input = serverToInput(server);
        const updated = await api.saveServer(server.id, { ...input, favorite: !server.favorite });
        await loadState(updated.id);
      },
      server.favorite ? "Removed from favorites." : "Marked as favorite.",
    );
  }

  async function refreshWebLinks(serverId: string) {
    const links = await api.listWebLinks(serverId);
    setWebLinks(links);
  }

  async function refreshTunnels(serverId: string) {
    const nextTunnels = await api.listTunnels(serverId);
    setTunnels(nextTunnels);
  }

  async function refreshRdpSettings(serverId: string) {
    const nextSettings = await api.getRdpSettings(serverId);
    setRdpSettings(nextSettings);
  }

  async function saveWebLink(server: ServerProfile, form: WebLinkFormModel) {
    const errors = validateWebLinkForm(form);
    if (hasWebLinkFormErrors(errors)) {
      setStatusMessage(null);
      setError(Object.values(errors).find(Boolean) ?? "Fix the highlighted web link fields.");
      return;
    }

    await runAction(
      "Saving web link",
      async () => {
        await api.saveWebLink(server.id, form.id ?? null, toWebLinkInput(form));
        setEditingWebLink(null);
        await refreshWebLinks(server.id);
      },
      form.id ? "Web link updated." : "Web link added.",
    );
  }

  async function deleteWebLink(server: ServerProfile, link: WebLink) {
    await runAction(
      "Deleting web link",
      async () => {
        await api.deleteWebLink(link.id);
        await refreshWebLinks(server.id);
      },
      "Web link deleted.",
    );
  }

  async function openWebLink(server: ServerProfile, link: WebLink) {
    await runAction(
      "Opening web link",
      async () => {
        await api.openWebLink(server.id, link.id);
      },
      "Web link opened in your browser.",
    );
  }

  async function saveTunnel(server: ServerProfile, form: TunnelFormModel) {
    const errors = validateTunnelForm(form);
    if (hasTunnelFormErrors(errors)) {
      setStatusMessage(null);
      setError(Object.values(errors).find(Boolean) ?? "Fix the highlighted tunnel fields.");
      return;
    }

    await runAction(
      "Saving tunnel",
      async () => {
        await api.saveTunnel(server.id, form.id ?? null, toTunnelInput(form));
        setEditingTunnel(null);
        await refreshTunnels(server.id);
      },
      form.id ? "Tunnel updated." : "Tunnel added.",
    );
  }

  async function deleteTunnel(server: ServerProfile, tunnel: Tunnel) {
    await runAction(
      "Deleting tunnel",
      async () => {
        await api.deleteTunnel(tunnel.id);
        await refreshTunnels(server.id);
      },
      "Tunnel deleted.",
    );
  }

  async function copyTunnelCommand(server: ServerProfile, tunnel: Tunnel) {
    await runAction(
      "Copying tunnel command",
      async () => {
        const command = await api.getTunnelCommand(server.id, tunnel.id);
        await navigator.clipboard.writeText(command);
      },
      "Tunnel command copied to clipboard.",
    );
  }

  async function launchTunnel(server: ServerProfile, tunnel: Tunnel) {
    await runAction(
      "Launching tunnel",
      async () => {
        await api.launchTunnel(server.id, tunnel.id);
      },
      "Tunnel launch requested in your external terminal.",
    );
  }

  async function saveRdpSettings(server: ServerProfile, form: RdpSettingsFormModel) {
    const errors = validateRdpSettingsForm(form);
    if (hasRdpFormErrors(errors)) {
      setStatusMessage(null);
      setError(Object.values(errors).find(Boolean) ?? "Fix the highlighted RDP fields.");
      return;
    }

    await runAction(
      "Saving RDP settings",
      async () => {
        await api.saveRdpSettings(server.id, toRdpSettingsInput(form));
        setEditingRdpSettings(null);
        await refreshRdpSettings(server.id);
      },
      "RDP settings saved.",
    );
  }

  async function resetRdpSettings(server: ServerProfile) {
    await runAction(
      "Resetting RDP settings",
      async () => {
        await api.deleteRdpSettings(server.id);
        setEditingRdpSettings(null);
        await refreshRdpSettings(server.id);
      },
      "RDP settings reset.",
    );
  }

  async function copyRdpCommand(server: ServerProfile) {
    await runAction(
      "Copying RDP command",
      async () => {
        const command = await api.getRdpCommand(server.id);
        await navigator.clipboard.writeText(command);
      },
      "RDP command copied to clipboard.",
    );
  }

  async function launchRdp(server: ServerProfile) {
    await runAction(
      "Launching RDP",
      async () => {
        await api.launchRdp(server.id);
      },
      "RDP launch requested.",
    );
  }

  if (!snapshot) {
    return (
      <div className="boot-screen">
        <div className="boot-mark">
          <Terminal size={34} />
        </div>
        <p>Loading SSH-Buddy...</p>
        {error ? <p className="error-text">{error}</p> : null}
      </div>
    );
  }

  const hasAnyServers = snapshot.servers.length > 0;
  const hasActiveServerFilter = query.trim().length > 0 || groupFilter !== null;
  const isBusy = Boolean(busyMessage);

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">
            <Terminal size={22} />
          </div>
          <div>
            <strong>SSH-Buddy</strong>
            <span>Homelab SSH manager</span>
          </div>
        </div>

        <nav className="nav-list" aria-label="Primary navigation">
          <NavButton icon={<Server size={18} />} label="Servers" active={activeSection === "servers"} onClick={() => setActiveSection("servers")} />
          <NavButton icon={<Folder size={18} />} label="Groups" active={activeSection === "groups"} onClick={() => setActiveSection("groups")} />
          <NavButton icon={<KeyRound size={18} />} label="SSH Keys" active={activeSection === "keys"} onClick={() => setActiveSection("keys")} />
          <NavButton icon={<Settings size={18} />} label="Settings" active={activeSection === "settings"} onClick={() => setActiveSection("settings")} />
        </nav>

        <div className="sidebar-search">
          <Search size={16} />
          <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search hosts, tags, notes" />
        </div>

        <div className="sidebar-section">
          <div className="section-title">
            <span>Groups</span>
            <span>{snapshot.groups.length}</span>
          </div>
          <button className={groupFilter === null ? "filter-pill active" : "filter-pill"} onClick={() => setGroupFilter(null)}>
            <FolderOpen size={15} />
            All servers
          </button>
          {snapshot.groups.map((group) => (
            <button
              key={group.id}
              className={groupFilter === group.id ? "filter-pill active" : "filter-pill"}
              onClick={() => setGroupFilter(group.id)}
            >
              <span className="color-dot" style={{ backgroundColor: group.color ?? "#3aa675" }} />
              {group.name}
            </button>
          ))}
        </div>

        <div className="server-list">
          <div className="section-title">
            <span>Servers</span>
            <span>{filteredServers.length}</span>
          </div>
          {filteredServers.map((server) => (
            <button
              key={server.id}
              className={selectedServer?.id === server.id ? "server-row active" : "server-row"}
              onClick={() => {
                setActiveSection("servers");
                setSelectedServerId(server.id);
              }}
            >
              <span className="server-row-top">
                <span className="server-row-name">{server.displayName}</span>
                {server.favorite ? <Star size={13} fill="currentColor" /> : null}
              </span>
              <span className="server-row-host">{serverDestination(server)}</span>
              <span className="server-row-tags">
                {server.tags.slice(0, 3).map((tag) => (
                  <span key={tag.id}>{tag.name}</span>
                ))}
              </span>
            </button>
          ))}
          {filteredServers.length === 0 ? (
            <div className="empty-mini">
              <strong>{hasAnyServers ? "No matching servers" : "No servers yet"}</strong>
              <span>{hasAnyServers ? "Try a different search or group." : "Add your first SSH profile to get started."}</span>
            </div>
          ) : null}
        </div>
      </aside>

      <main className="workspace">
        <header className="workspace-topbar">
          <div>
            <p className="eyebrow">{activeSection}</p>
            <h1>{sectionTitle(activeSection, selectedServer)}</h1>
          </div>
          <div className="topbar-actions">
            <button className="button ghost" disabled={isBusy} title="Preview ~/.ssh/config before importing" onClick={() => setShowImport(true)}>
              <Download size={17} />
              Import SSH config
            </button>
            <button className="button primary" disabled={isBusy} onClick={() => setEditingServer(newServerDraft())}>
              <Plus size={17} />
              Add server
            </button>
          </div>
        </header>

        {error ? <div className="status-banner danger">{error}</div> : null}
        {statusMessage ? <div className="status-banner success">{statusMessage}</div> : null}
        {busyMessage ? <div className="status-banner">{busyMessage}...</div> : null}

        {activeSection === "servers" ? (
          <ServerDetails
            server={selectedServer}
            groups={snapshot.groups}
            onEdit={(server) => setEditingServer(newServerDraft(server))}
            onDelete={setServerPendingDelete}
            onCopyCommand={copySshCommand}
            onLaunch={launchSsh}
            onCopySftpCommand={copySftpCommand}
            onLaunchSftp={launchSftp}
            onToggleFavorite={toggleFavorite}
            webLinks={webLinks}
            webLinksLoading={webLinksLoading}
            editingWebLink={editingWebLink}
            onAddWebLink={() => setEditingWebLink(newWebLinkDraft())}
            onEditWebLink={(link) => setEditingWebLink(newWebLinkDraft(link))}
            onCancelWebLink={() => setEditingWebLink(null)}
            onSaveWebLink={saveWebLink}
            onDeleteWebLink={deleteWebLink}
            onOpenWebLink={openWebLink}
            tunnels={tunnels}
            tunnelsLoading={tunnelsLoading}
            editingTunnel={editingTunnel}
            onAddTunnel={() => setEditingTunnel(newTunnelDraft())}
            onEditTunnel={(tunnel) => setEditingTunnel(newTunnelDraft(tunnel))}
            onCancelTunnel={() => setEditingTunnel(null)}
            onSaveTunnel={saveTunnel}
            onDeleteTunnel={deleteTunnel}
            onCopyTunnelCommand={copyTunnelCommand}
            onLaunchTunnel={launchTunnel}
            rdpSettings={rdpSettings}
            rdpLoading={rdpLoading}
            editingRdpSettings={editingRdpSettings}
            onConfigureRdp={() => setEditingRdpSettings(newRdpSettingsDraft(rdpSettings))}
            onCancelRdp={() => setEditingRdpSettings(null)}
            onSaveRdp={saveRdpSettings}
            onResetRdp={resetRdpSettings}
            onCopyRdpCommand={copyRdpCommand}
            onLaunchRdp={launchRdp}
            keyRefs={snapshot.sshKeys}
            hasAnyServers={hasAnyServers}
            hasActiveServerFilter={hasActiveServerFilter}
            onAddServer={() => setEditingServer(newServerDraft())}
            onImport={() => setShowImport(true)}
            onClearFilters={() => {
              setQuery("");
              setGroupFilter(null);
            }}
            busy={isBusy}
          />
        ) : null}

        {activeSection === "groups" ? (
          <GroupsPanel
            groups={snapshot.groups}
            onSave={async (input) => {
              await runAction(
                "Saving group",
                async () => {
                  await api.saveGroup(input);
                  await loadState();
                },
                "Group created.",
              );
            }}
            onDelete={async (group) => {
              await runAction(
                "Deleting group",
                async () => {
                  await api.deleteGroup(group.id);
                  await loadState();
                },
                "Group deleted.",
              );
            }}
          />
        ) : null}

        {activeSection === "keys" ? (
          <KeysPanel
            keys={snapshot.sshKeys}
            onSave={async (input) => {
              await runAction(
                "Saving key reference",
                async () => {
                  await api.saveSshKey(input);
                  await loadState();
                },
                "SSH key reference saved.",
              );
            }}
            onDelete={async (key) => {
              await runAction(
                "Deleting key reference",
                async () => {
                  await api.deleteSshKey(key.id);
                  await loadState();
                },
                "SSH key reference deleted.",
              );
            }}
          />
        ) : null}

        {activeSection === "settings" ? (
          <SettingsPanel
            settings={snapshot.settings}
            onSave={async (settings) => {
              await runAction(
                "Saving settings",
                async () => {
                  await api.saveSettings(settings);
                  await loadState();
                },
                "Settings saved.",
              );
            }}
          />
        ) : null}
      </main>

      {editingServer ? (
        <ServerForm
          form={editingServer}
          groups={snapshot.groups}
          keyRefs={snapshot.sshKeys}
          onChange={setEditingServer}
          onCancel={() => setEditingServer(null)}
          onSave={saveServer}
          busy={isBusy}
        />
      ) : null}

      {showImport ? (
        <ImportDialog
          onClose={() => setShowImport(false)}
          onImported={async (result) => {
            setShowImport(false);
            await loadState(result.servers[0]?.id ?? null);
            setStatusMessage(formatImportResult(result));
          }}
        />
      ) : null}

      {serverPendingDelete ? (
        <DeleteServerDialog
          server={serverPendingDelete}
          onCancel={() => setServerPendingDelete(null)}
          onConfirm={confirmDeleteServer}
          busy={isBusy}
        />
      ) : null}
    </div>
  );
}

function sectionTitle(section: Section, server: ServerProfile | null) {
  if (section === "servers") {
    return server?.displayName ?? "Servers";
  }

  if (section === "groups") {
    return "Groups";
  }

  if (section === "keys") {
    return "SSH Keys";
  }

  return "Settings";
}

function NavButton({
  icon,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button className={active ? "nav-button active" : "nav-button"} onClick={onClick}>
      {icon}
      {label}
    </button>
  );
}

function ServerDetails({
  server,
  groups,
  keyRefs,
  onEdit,
  onDelete,
  onCopyCommand,
  onLaunch,
  onCopySftpCommand,
  onLaunchSftp,
  onToggleFavorite,
  webLinks,
  webLinksLoading,
  editingWebLink,
  onAddWebLink,
  onEditWebLink,
  onCancelWebLink,
  onSaveWebLink,
  onDeleteWebLink,
  onOpenWebLink,
  tunnels,
  tunnelsLoading,
  editingTunnel,
  onAddTunnel,
  onEditTunnel,
  onCancelTunnel,
  onSaveTunnel,
  onDeleteTunnel,
  onCopyTunnelCommand,
  onLaunchTunnel,
  rdpSettings,
  rdpLoading,
  editingRdpSettings,
  onConfigureRdp,
  onCancelRdp,
  onSaveRdp,
  onResetRdp,
  onCopyRdpCommand,
  onLaunchRdp,
  hasAnyServers,
  hasActiveServerFilter,
  onAddServer,
  onImport,
  onClearFilters,
  busy,
}: {
  server: ServerProfile | null;
  groups: Group[];
  keyRefs: SshKeyRef[];
  onEdit: (server: ServerProfile) => void;
  onDelete: (server: ServerProfile) => void;
  onCopyCommand: (server: ServerProfile) => void;
  onLaunch: (server: ServerProfile) => void;
  onCopySftpCommand: (server: ServerProfile) => void;
  onLaunchSftp: (server: ServerProfile) => void;
  onToggleFavorite: (server: ServerProfile) => void;
  webLinks: WebLink[];
  webLinksLoading: boolean;
  editingWebLink: WebLinkFormModel | null;
  onAddWebLink: () => void;
  onEditWebLink: (link: WebLink) => void;
  onCancelWebLink: () => void;
  onSaveWebLink: (server: ServerProfile, form: WebLinkFormModel) => void;
  onDeleteWebLink: (server: ServerProfile, link: WebLink) => void;
  onOpenWebLink: (server: ServerProfile, link: WebLink) => void;
  tunnels: Tunnel[];
  tunnelsLoading: boolean;
  editingTunnel: TunnelFormModel | null;
  onAddTunnel: () => void;
  onEditTunnel: (tunnel: Tunnel) => void;
  onCancelTunnel: () => void;
  onSaveTunnel: (server: ServerProfile, form: TunnelFormModel) => void;
  onDeleteTunnel: (server: ServerProfile, tunnel: Tunnel) => void;
  onCopyTunnelCommand: (server: ServerProfile, tunnel: Tunnel) => void;
  onLaunchTunnel: (server: ServerProfile, tunnel: Tunnel) => void;
  rdpSettings: RdpSettings | null;
  rdpLoading: boolean;
  editingRdpSettings: RdpSettingsFormModel | null;
  onConfigureRdp: () => void;
  onCancelRdp: () => void;
  onSaveRdp: (server: ServerProfile, form: RdpSettingsFormModel) => void;
  onResetRdp: (server: ServerProfile) => void;
  onCopyRdpCommand: (server: ServerProfile) => void;
  onLaunchRdp: (server: ServerProfile) => void;
  hasAnyServers: boolean;
  hasActiveServerFilter: boolean;
  onAddServer: () => void;
  onImport: () => void;
  onClearFilters: () => void;
  busy: boolean;
}) {
  if (!server) {
    return (
      <section className="empty-state">
        <Server size={42} />
        <h2>{hasAnyServers ? "No matching servers" : "No servers yet"}</h2>
        <p>
          {hasAnyServers
            ? "No saved profile matches the current search or group filter."
            : "Create your first SSH profile or import concrete aliases from your OpenSSH config."}
        </p>
        <div className="empty-actions">
          {hasActiveServerFilter ? (
            <button className="button ghost" type="button" disabled={busy} onClick={onClearFilters}>
              Clear filters
            </button>
          ) : null}
          <button className="button primary" type="button" disabled={busy} onClick={onAddServer}>
            <Plus size={17} />
            Add server
          </button>
          {!hasAnyServers ? (
            <button className="button ghost" type="button" disabled={busy} onClick={onImport}>
              <Download size={17} />
              Import SSH config
            </button>
          ) : null}
        </div>
      </section>
    );
  }

  return (
    <div className="detail-grid">
      <section className="detail-main">
        <div className="server-hero">
          <div>
            <p className="eyebrow">SSH profile</p>
            <h2>{serverDestination(server)}</h2>
            <p>{groupName(groups, server.groupId)} · Updated {shortDate(server.updatedAt)}</p>
          </div>
          <div className="hero-actions">
            <button
              className={server.favorite ? "icon-button favorite active" : "icon-button favorite"}
              aria-label={server.favorite ? "Remove favorite" : "Mark as favorite"}
              title={server.favorite ? "Remove favorite" : "Mark as favorite"}
              disabled={busy}
              onClick={() => onToggleFavorite(server)}
            >
              <Star size={18} fill={server.favorite ? "currentColor" : "none"} />
            </button>
            <button className="icon-button" aria-label="Edit server" title="Edit server" disabled={busy} onClick={() => onEdit(server)}>
              <Pencil size={18} />
            </button>
            <button className="icon-button danger" aria-label="Delete server" title="Delete server" disabled={busy} onClick={() => onDelete(server)}>
              <Trash2 size={18} />
            </button>
          </div>
        </div>

        <div className="action-strip">
          <button className="button primary" disabled={busy} title="Launch SSH in an external terminal" onClick={() => onLaunch(server)}>
            <Terminal size={17} />
            Open SSH
          </button>
          <button className="button" disabled={busy} onClick={() => onCopyCommand(server)}>
            <Copy size={17} />
            Copy SSH command
          </button>
          <button className="button" disabled={busy} title="Launch SFTP in an external terminal" onClick={() => onLaunchSftp(server)}>
            <FolderOpen size={17} />
            Open SFTP
          </button>
          <button className="button" disabled={busy} onClick={() => onCopySftpCommand(server)}>
            <Copy size={17} />
            Copy SFTP command
          </button>
        </div>
        <p className="field-hint">SFTP uses system OpenSSH, the same key references, ssh-agent, ProxyJump settings, and terminal prompts as SSH.</p>

        <div className="info-grid">
          <Info label="Host" value={server.host} />
          <Info label="Port" value={String(server.port)} />
          <Info label="Username" value={server.username || "Default OpenSSH user"} />
          <Info label="Identity file" value={keyLabel(keyRefs, server.identityFileId)} />
          {server.proxyJump ? <Info label="ProxyJump" value={server.proxyJump} /> : null}
        </div>

        <section className="panel">
          <div className="panel-heading">
            <h3>Notes</h3>
            <span>No secrets here</span>
          </div>
          <p className={server.notes?.trim() ? "notes" : "muted"}>{server.notes?.trim() || "No notes saved for this server."}</p>
          <p className="field-hint">Notes are plaintext local metadata. Do not store passwords, private keys, tokens, or sudo details here.</p>
        </section>

        <TunnelsPanel
          server={server}
          tunnels={tunnels}
          loading={tunnelsLoading}
          editingTunnel={editingTunnel}
          busy={busy}
          onAdd={onAddTunnel}
          onEdit={onEditTunnel}
          onCancel={onCancelTunnel}
          onSave={onSaveTunnel}
          onDelete={onDeleteTunnel}
          onCopyCommand={onCopyTunnelCommand}
          onLaunch={onLaunchTunnel}
        />

        <RdpPanel
          server={server}
          settings={rdpSettings}
          loading={rdpLoading}
          editingSettings={editingRdpSettings}
          busy={busy}
          onConfigure={onConfigureRdp}
          onCancel={onCancelRdp}
          onSave={onSaveRdp}
          onReset={onResetRdp}
          onCopyCommand={onCopyRdpCommand}
          onLaunch={onLaunchRdp}
        />

        <WebLinksPanel
          server={server}
          links={webLinks}
          loading={webLinksLoading}
          editingLink={editingWebLink}
          busy={busy}
          onAdd={onAddWebLink}
          onEdit={onEditWebLink}
          onCancel={onCancelWebLink}
          onSave={onSaveWebLink}
          onDelete={onDeleteWebLink}
          onOpen={onOpenWebLink}
        />
      </section>

      <aside className="detail-side">
        <section className="panel">
          <div className="panel-heading">
            <h3>Tags</h3>
            <Tag size={16} />
          </div>
          <div className="tag-list">
            {server.tags.length ? server.tags.map((tag) => <span key={tag.id}>{tag.name}</span>) : <p className="muted">No tags</p>}
          </div>
        </section>

        <section className="panel warning">
          <ShieldAlert size={20} />
          <div>
            <h3>Security posture</h3>
            <p>Private keys and passphrases stay outside SSH-Buddy. Use OpenSSH, ssh-agent, and normal terminal prompts.</p>
          </div>
        </section>

        <section className="panel">
          <div className="panel-heading">
            <h3>Planned actions</h3>
            <span>Later</span>
          </div>
          <div className="planned-list">
            <span>Embedded terminal</span>
            <span>SFTP file browser</span>
            <span>VNC</span>
          </div>
          <p className="muted">These are intentionally disabled until their backend behavior is implemented.</p>
        </section>
      </aside>
    </div>
  );
}

function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="info-cell">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function RdpPanel({
  server,
  settings,
  loading,
  editingSettings,
  busy,
  onConfigure,
  onCancel,
  onSave,
  onReset,
  onCopyCommand,
  onLaunch,
}: {
  server: ServerProfile;
  settings: RdpSettings | null;
  loading: boolean;
  editingSettings: RdpSettingsFormModel | null;
  busy: boolean;
  onConfigure: () => void;
  onCancel: () => void;
  onSave: (server: ServerProfile, form: RdpSettingsFormModel) => void;
  onReset: (server: ServerProfile) => void;
  onCopyCommand: (server: ServerProfile) => void;
  onLaunch: (server: ServerProfile) => void;
}) {
  const canLaunch = Boolean(settings?.enabled);

  return (
    <section className="panel">
      <div className="panel-heading">
        <div>
          <h3>RDP</h3>
          <span>FreeRDP external launch</span>
        </div>
        <button className="button compact" type="button" disabled={busy || Boolean(editingSettings)} onClick={onConfigure}>
          <Pencil size={15} />
          {settings ? "Edit RDP" : "Configure RDP"}
        </button>
      </div>

      <p className="field-hint">SSH-Buddy does not store RDP passwords. FreeRDP will prompt if credentials are needed.</p>

      {editingSettings ? <RdpSettingsForm form={editingSettings} busy={busy} onCancel={onCancel} onSave={(form) => onSave(server, form)} /> : null}

      {loading ? <p className="muted">Loading RDP settings...</p> : null}

      {!loading && !settings && !editingSettings ? (
        <div className="empty-inline">
          <Monitor size={30} />
          <strong>No RDP profile yet</strong>
          <span>Configure FreeRDP launch options for this server without storing passwords.</span>
        </div>
      ) : null}

      {settings && !editingSettings ? (
        <div className="web-link-list">
          <div className="web-link-row">
            <Monitor size={18} />
            <div>
              <strong>{settings.enabled ? "RDP enabled" : "RDP disabled"}</strong>
              <span>
                {settings.username ? `${settings.username}${settings.domain ? ` @ ${settings.domain}` : ""}` : "FreeRDP will prompt for username"} ·{" "}
                port {settings.port} · {rdpSettingsSummary(settings)}
              </span>
            </div>
            <div className="row-actions">
              <button className="button compact" type="button" disabled={busy || !canLaunch} onClick={() => onLaunch(server)}>
                <Monitor size={15} />
                Open
              </button>
              <button className="button compact" type="button" disabled={busy || !canLaunch} onClick={() => onCopyCommand(server)}>
                <Copy size={15} />
                Copy
              </button>
              <button className="icon-button danger" type="button" aria-label="Reset RDP settings" disabled={busy} onClick={() => onReset(server)}>
                <Trash2 size={16} />
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}

function RdpSettingsForm({
  form,
  busy,
  onCancel,
  onSave,
}: {
  form: RdpSettingsFormModel;
  busy: boolean;
  onCancel: () => void;
  onSave: (form: RdpSettingsFormModel) => void;
}) {
  const [draft, setDraft] = useState(form);
  const [submitted, setSubmitted] = useState(false);
  const errors = validateRdpSettingsForm(draft);

  useEffect(() => {
    setDraft(form);
    setSubmitted(false);
  }, [form]);

  const update = <Key extends keyof RdpSettingsFormModel>(key: Key, value: RdpSettingsFormModel[Key]) => {
    setDraft({ ...draft, [key]: value });
  };

  return (
    <form
      className="web-link-form"
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        setSubmitted(true);
        if (hasRdpFormErrors(errors)) {
          return;
        }
        onSave(draft);
      }}
    >
      <div className="form-grid">
        <label className="span-2 checkbox-field">
          <input type="checkbox" checked={draft.enabled} onChange={(event) => update("enabled", event.target.checked)} />
          Enable RDP actions for this server
        </label>
        <label>
          Username
          <input value={draft.username} onChange={(event) => update("username", event.target.value)} placeholder="optional" />
        </label>
        <label>
          Domain
          <input value={draft.domain} onChange={(event) => update("domain", event.target.value)} placeholder="optional" />
        </label>
        <label>
          Port
          <input
            type="number"
            min={1}
            max={65535}
            value={draft.port}
            onChange={(event) => update("port", event.target.value)}
            aria-invalid={submitted && Boolean(errors.port)}
            aria-describedby={submitted && errors.port ? "rdp-port-error" : undefined}
          />
          {submitted && errors.port ? (
            <span className="field-error" id="rdp-port-error">
              {errors.port}
            </span>
          ) : null}
        </label>
        <label>
          Color depth
          <select
            value={draft.colorDepth}
            onChange={(event) => update("colorDepth", event.target.value as RdpSettingsFormModel["colorDepth"])}
            aria-invalid={submitted && Boolean(errors.colorDepth)}
            aria-describedby={submitted && errors.colorDepth ? "rdp-color-depth-error" : undefined}
          >
            <option value="">Default</option>
            <option value="16">16 bpp</option>
            <option value="24">24 bpp</option>
            <option value="32">32 bpp</option>
          </select>
          {submitted && errors.colorDepth ? (
            <span className="field-error" id="rdp-color-depth-error">
              {errors.colorDepth}
            </span>
          ) : null}
        </label>
        <label>
          Width
          <input
            type="number"
            min={320}
            max={16384}
            value={draft.width}
            onChange={(event) => update("width", event.target.value)}
            placeholder="optional"
            aria-invalid={submitted && Boolean(errors.width)}
            aria-describedby={submitted && errors.width ? "rdp-width-error" : undefined}
          />
          {submitted && errors.width ? (
            <span className="field-error" id="rdp-width-error">
              {errors.width}
            </span>
          ) : null}
        </label>
        <label>
          Height
          <input
            type="number"
            min={320}
            max={16384}
            value={draft.height}
            onChange={(event) => update("height", event.target.value)}
            placeholder="optional"
            aria-invalid={submitted && Boolean(errors.height)}
            aria-describedby={submitted && errors.height ? "rdp-height-error" : undefined}
          />
          {submitted && errors.height ? (
            <span className="field-error" id="rdp-height-error">
              {errors.height}
            </span>
          ) : null}
        </label>
        <label className="checkbox-field">
          <input type="checkbox" checked={draft.fullscreen} onChange={(event) => update("fullscreen", event.target.checked)} />
          Fullscreen
        </label>
        <label className="checkbox-field">
          <input type="checkbox" checked={draft.multiMonitor} onChange={(event) => update("multiMonitor", event.target.checked)} />
          Multi-monitor
        </label>
      </div>
      <p className="field-hint">No password field is provided. FreeRDP prompts interactively when credentials are needed.</p>
      <div className="modal-actions">
        <button type="button" className="button ghost" disabled={busy} onClick={onCancel}>
          Cancel
        </button>
        <button type="submit" className="button primary" disabled={busy}>
          Save RDP settings
        </button>
      </div>
    </form>
  );
}

function TunnelsPanel({
  server,
  tunnels,
  loading,
  editingTunnel,
  busy,
  onAdd,
  onEdit,
  onCancel,
  onSave,
  onDelete,
  onCopyCommand,
  onLaunch,
}: {
  server: ServerProfile;
  tunnels: Tunnel[];
  loading: boolean;
  editingTunnel: TunnelFormModel | null;
  busy: boolean;
  onAdd: () => void;
  onEdit: (tunnel: Tunnel) => void;
  onCancel: () => void;
  onSave: (server: ServerProfile, form: TunnelFormModel) => void;
  onDelete: (server: ServerProfile, tunnel: Tunnel) => void;
  onCopyCommand: (server: ServerProfile, tunnel: Tunnel) => void;
  onLaunch: (server: ServerProfile, tunnel: Tunnel) => void;
}) {
  return (
    <section className="panel">
      <div className="panel-heading">
        <div>
          <h3>SSH Tunnels</h3>
          <span>Local forwarding</span>
        </div>
        <button className="button compact" type="button" disabled={busy || Boolean(editingTunnel)} onClick={onAdd}>
          <Plus size={15} />
          Add tunnel
        </button>
      </div>

      <p className="field-hint">Tunnels use OpenSSH local forwarding and stay open while the external terminal session is running.</p>

      {editingTunnel ? (
        <TunnelForm form={editingTunnel} busy={busy} onCancel={onCancel} onSave={(form) => onSave(server, form)} />
      ) : null}

      {loading ? <p className="muted">Loading tunnels...</p> : null}

      {!loading && tunnels.length === 0 && !editingTunnel ? (
        <div className="empty-inline">
          <Cable size={30} />
          <strong>No SSH tunnels yet</strong>
          <span>Add local forwards for databases, admin panels, or internal services reachable from this server.</span>
        </div>
      ) : null}

      {tunnels.length > 0 ? (
        <div className="web-link-list">
          {tunnels.map((tunnel) => (
            <div className="web-link-row" key={tunnel.id}>
              <Cable size={18} />
              <div>
                <strong>{tunnel.label}</strong>
                <span>{tunnelSummary(tunnel)}</span>
              </div>
              <div className="row-actions">
                <button className="button compact" type="button" disabled={busy} onClick={() => onLaunch(server, tunnel)}>
                  <Terminal size={15} />
                  Launch
                </button>
                <button className="button compact" type="button" disabled={busy} onClick={() => onCopyCommand(server, tunnel)}>
                  <Copy size={15} />
                  Copy
                </button>
                <button className="icon-button" type="button" aria-label={`Edit ${tunnel.label}`} disabled={busy} onClick={() => onEdit(tunnel)}>
                  <Pencil size={16} />
                </button>
                <button className="icon-button danger" type="button" aria-label={`Delete ${tunnel.label}`} disabled={busy} onClick={() => onDelete(server, tunnel)}>
                  <Trash2 size={16} />
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : null}
    </section>
  );
}

function TunnelForm({
  form,
  busy,
  onCancel,
  onSave,
}: {
  form: TunnelFormModel;
  busy: boolean;
  onCancel: () => void;
  onSave: (form: TunnelFormModel) => void;
}) {
  const [draft, setDraft] = useState(form);
  const [submitted, setSubmitted] = useState(false);
  const errors = validateTunnelForm(draft);

  useEffect(() => {
    setDraft(form);
    setSubmitted(false);
  }, [form]);

  const update = <Key extends keyof TunnelFormModel>(key: Key, value: TunnelFormModel[Key]) => {
    setDraft({ ...draft, [key]: value });
  };

  return (
    <form
      className="web-link-form"
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        setSubmitted(true);
        if (hasTunnelFormErrors(errors)) {
          return;
        }
        onSave(draft);
      }}
    >
      <div className="form-grid">
        <label className="span-2">
          Label
          <input
            value={draft.label}
            onChange={(event) => update("label", event.target.value)}
            aria-invalid={submitted && Boolean(errors.label)}
            aria-describedby={submitted && errors.label ? "tunnel-label-error" : undefined}
          />
          {submitted && errors.label ? (
            <span className="field-error" id="tunnel-label-error">
              {errors.label}
            </span>
          ) : null}
        </label>
        <label>
          Local bind host
          <input
            value={draft.localBindHost}
            onChange={(event) => update("localBindHost", event.target.value)}
            placeholder="127.0.0.1"
            aria-invalid={submitted && Boolean(errors.localBindHost)}
            aria-describedby={submitted && errors.localBindHost ? "tunnel-local-host-error" : "tunnel-local-host-hint"}
          />
          {submitted && errors.localBindHost ? (
            <span className="field-error" id="tunnel-local-host-error">
              {errors.localBindHost}
            </span>
          ) : (
            <span className="field-hint" id="tunnel-local-host-hint">
              Defaults to 127.0.0.1 when empty.
            </span>
          )}
        </label>
        <label>
          Local port
          <input
            type="number"
            min={1}
            max={65535}
            value={draft.localPort}
            onChange={(event) => update("localPort", event.target.value)}
            placeholder="15432"
            aria-invalid={submitted && Boolean(errors.localPort)}
            aria-describedby={submitted && errors.localPort ? "tunnel-local-port-error" : undefined}
          />
          {submitted && errors.localPort ? (
            <span className="field-error" id="tunnel-local-port-error">
              {errors.localPort}
            </span>
          ) : null}
        </label>
        <label>
          Remote host
          <input
            value={draft.remoteHost}
            onChange={(event) => update("remoteHost", event.target.value)}
            placeholder="db.internal"
            aria-invalid={submitted && Boolean(errors.remoteHost)}
            aria-describedby={submitted && errors.remoteHost ? "tunnel-remote-host-error" : undefined}
          />
          {submitted && errors.remoteHost ? (
            <span className="field-error" id="tunnel-remote-host-error">
              {errors.remoteHost}
            </span>
          ) : null}
        </label>
        <label>
          Remote port
          <input
            type="number"
            min={1}
            max={65535}
            value={draft.remotePort}
            onChange={(event) => update("remotePort", event.target.value)}
            placeholder="5432"
            aria-invalid={submitted && Boolean(errors.remotePort)}
            aria-describedby={submitted && errors.remotePort ? "tunnel-remote-port-error" : undefined}
          />
          {submitted && errors.remotePort ? (
            <span className="field-error" id="tunnel-remote-port-error">
              {errors.remotePort}
            </span>
          ) : null}
        </label>
      </div>
      <div className="modal-actions">
        <button type="button" className="button ghost" disabled={busy} onClick={onCancel}>
          Cancel
        </button>
        <button type="submit" className="button primary" disabled={busy}>
          Save tunnel
        </button>
      </div>
    </form>
  );
}

function WebLinksPanel({
  server,
  links,
  loading,
  editingLink,
  busy,
  onAdd,
  onEdit,
  onCancel,
  onSave,
  onDelete,
  onOpen,
}: {
  server: ServerProfile;
  links: WebLink[];
  loading: boolean;
  editingLink: WebLinkFormModel | null;
  busy: boolean;
  onAdd: () => void;
  onEdit: (link: WebLink) => void;
  onCancel: () => void;
  onSave: (server: ServerProfile, form: WebLinkFormModel) => void;
  onDelete: (server: ServerProfile, link: WebLink) => void;
  onOpen: (server: ServerProfile, link: WebLink) => void;
}) {
  return (
    <section className="panel">
      <div className="panel-heading">
        <h3>Web/Admin Links</h3>
        <button className="button compact" type="button" disabled={busy || Boolean(editingLink)} onClick={onAdd}>
          <Plus size={15} />
          Add link
        </button>
      </div>

      {editingLink ? (
        <WebLinkForm form={editingLink} busy={busy} onCancel={onCancel} onSave={(form) => onSave(server, form)} />
      ) : null}

      {loading ? <p className="muted">Loading web links...</p> : null}

      {!loading && links.length === 0 && !editingLink ? (
        <div className="empty-inline">
          <ExternalLink size={30} />
          <strong>No web admin links yet</strong>
          <span>Add URLs for dashboards like Proxmox, router admin, NAS, or monitoring pages.</span>
        </div>
      ) : null}

      {links.length > 0 ? (
        <div className="web-link-list">
          {links.map((link) => (
            <div className="web-link-row" key={link.id}>
              <ExternalLink size={18} />
              <div>
                <strong>{link.label}</strong>
                <span>{link.url}</span>
              </div>
              <div className="row-actions">
                <button className="button compact" type="button" disabled={busy} onClick={() => onOpen(server, link)}>
                  <ExternalLink size={15} />
                  Open
                </button>
                <button className="icon-button" type="button" aria-label={`Edit ${link.label}`} disabled={busy} onClick={() => onEdit(link)}>
                  <Pencil size={16} />
                </button>
                <button className="icon-button danger" type="button" aria-label={`Delete ${link.label}`} disabled={busy} onClick={() => onDelete(server, link)}>
                  <Trash2 size={16} />
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : null}
    </section>
  );
}

function WebLinkForm({
  form,
  busy,
  onCancel,
  onSave,
}: {
  form: WebLinkFormModel;
  busy: boolean;
  onCancel: () => void;
  onSave: (form: WebLinkFormModel) => void;
}) {
  const [draft, setDraft] = useState(form);
  const [submitted, setSubmitted] = useState(false);
  const errors = validateWebLinkForm(draft);

  useEffect(() => {
    setDraft(form);
    setSubmitted(false);
  }, [form]);

  return (
    <form
      className="web-link-form"
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        setSubmitted(true);
        if (hasWebLinkFormErrors(errors)) {
          return;
        }
        onSave(draft);
      }}
    >
      <label>
        Label
        <input
          value={draft.label}
          onChange={(event) => setDraft({ ...draft, label: event.target.value })}
          aria-invalid={submitted && Boolean(errors.label)}
          aria-describedby={submitted && errors.label ? "web-link-label-error" : undefined}
        />
        {submitted && errors.label ? (
          <span className="field-error" id="web-link-label-error">
            {errors.label}
          </span>
        ) : null}
      </label>
      <label>
        URL
        <input
          value={draft.url}
          onChange={(event) => setDraft({ ...draft, url: event.target.value })}
          placeholder="https://server.local:8006"
          aria-invalid={submitted && Boolean(errors.url)}
          aria-describedby={submitted && errors.url ? "web-link-url-error" : undefined}
        />
        {submitted && errors.url ? (
          <span className="field-error" id="web-link-url-error">
            {errors.url}
          </span>
        ) : (
          <span className="field-hint">Only http:// and https:// links are allowed. Do not include credentials.</span>
        )}
      </label>
      <div className="modal-actions">
        <button type="button" className="button ghost" disabled={busy} onClick={onCancel}>
          Cancel
        </button>
        <button type="submit" className="button primary" disabled={busy}>
          Save link
        </button>
      </div>
    </form>
  );
}

function keyLabel(keyRefs: SshKeyRef[], keyId: string | null) {
  if (!keyId) {
    return "OpenSSH default";
  }

  const key = keyRefs.find((item) => item.id === keyId);
  return key ? `${key.label} (${key.path})` : "Missing key reference";
}

function DeleteServerDialog({
  server,
  onCancel,
  onConfirm,
  busy,
}: {
  server: ServerProfile;
  onCancel: () => void;
  onConfirm: () => void;
  busy: boolean;
}) {
  return (
    <div className="modal-backdrop" role="presentation">
      <section className="modal confirm-modal" role="dialog" aria-modal="true" aria-labelledby="delete-server-title">
        <div className="modal-heading">
          <div>
            <p className="eyebrow">Confirm delete</p>
            <h2 id="delete-server-title">Delete {server.displayName}?</h2>
          </div>
          <button type="button" className="icon-button" aria-label="Close" disabled={busy} onClick={onCancel}>
            <X size={18} />
          </button>
        </div>

        <div className="confirm-body">
          <Trash2 size={24} />
          <div>
            <p>This removes only the local SSH-Buddy profile metadata for this server.</p>
            <p className="muted">OpenSSH keys, ssh-agent state, and remote systems are not changed.</p>
          </div>
        </div>

        <div className="modal-actions">
          <button type="button" className="button ghost" disabled={busy} onClick={onCancel}>
            Cancel
          </button>
          <button type="button" className="button danger" disabled={busy} onClick={onConfirm}>
            Delete server
          </button>
        </div>
      </section>
    </div>
  );
}

function ServerForm({
  form,
  groups,
  keyRefs,
  onChange,
  onCancel,
  onSave,
  busy,
}: {
  form: ServerFormModel;
  groups: Group[];
  keyRefs: SshKeyRef[];
  onChange: (form: ServerFormModel) => void;
  onCancel: () => void;
  onSave: (form: ServerFormModel) => void;
  busy: boolean;
}) {
  const [submitted, setSubmitted] = useState(false);
  const errors = validateServerForm(form);
  const selectedKey = keyRefs.find((key) => key.id === form.identityFileId) ?? null;
  const selectedGroup = groups.find((group) => group.id === form.groupId) ?? null;

  const update = <Key extends keyof ServerFormModel>(key: Key, value: ServerFormModel[Key]) => {
    onChange({ ...form, [key]: value });
  };

  return (
    <div className="modal-backdrop" role="presentation">
      <form
        className="modal"
        noValidate
        onSubmit={(event) => {
          event.preventDefault();
          setSubmitted(true);
          if (hasServerFormErrors(errors)) {
            return;
          }
          onSave(form);
        }}
      >
        <div className="modal-heading">
          <div>
            <p className="eyebrow">Server profile</p>
            <h2>{form.id ? "Edit server" : "Add server"}</h2>
          </div>
          <button type="button" className="icon-button" aria-label="Close" disabled={busy} onClick={onCancel}>
            <X size={18} />
          </button>
        </div>

        <div className="form-grid">
          <label>
            Display name
            <input
              value={form.displayName}
              onChange={(event) => update("displayName", event.target.value)}
              aria-invalid={submitted && Boolean(errors.displayName)}
              aria-describedby={submitted && errors.displayName ? "display-name-error" : undefined}
            />
            {submitted && errors.displayName ? (
              <span className="field-error" id="display-name-error">
                {errors.displayName}
              </span>
            ) : null}
          </label>
          <label>
            Hostname or IP
            <input
              value={form.host}
              onChange={(event) => update("host", event.target.value)}
              aria-invalid={submitted && Boolean(errors.host)}
              aria-describedby={submitted && errors.host ? "host-error" : undefined}
            />
            {submitted && errors.host ? (
              <span className="field-error" id="host-error">
                {errors.host}
              </span>
            ) : null}
          </label>
          <label>
            Port
            <input
              type="number"
              min={1}
              max={65535}
              value={form.port}
              onChange={(event) => update("port", event.target.value)}
              aria-invalid={submitted && Boolean(errors.port)}
              aria-describedby={submitted && errors.port ? "port-error" : undefined}
            />
            {submitted && errors.port ? (
              <span className="field-error" id="port-error">
                {errors.port}
              </span>
            ) : null}
          </label>
          <label>
            Username
            <input value={form.username} onChange={(event) => update("username", event.target.value)} placeholder="OpenSSH default" />
          </label>
          <label className="span-2">
            ProxyJump
            <input
              value={form.proxyJump}
              onChange={(event) => update("proxyJump", event.target.value)}
              placeholder="user@bastion:22"
              aria-invalid={submitted && Boolean(errors.proxyJump)}
              aria-describedby={submitted && errors.proxyJump ? "proxy-jump-error" : "proxy-jump-hint"}
            />
            {submitted && errors.proxyJump ? (
              <span className="field-error" id="proxy-jump-error">
                {errors.proxyJump}
              </span>
            ) : (
              <span className="field-hint" id="proxy-jump-hint">
                Uses OpenSSH `-J`. Example: `user@bastion:22`.
              </span>
            )}
          </label>
          <div className="field-stack">
            <label>
              SSH key reference
              <select value={form.identityFileId} onChange={(event) => update("identityFileId", event.target.value)}>
                <option value="">OpenSSH default</option>
                {keyRefs.map((key) => (
                  <option key={key.id} value={key.id}>
                    {key.label} - {key.path}
                  </option>
                ))}
              </select>
            </label>
            {selectedKey ? (
              <div className="selected-meta">
                <strong>{selectedKey.label}</strong>
                <span>{selectedKey.path}</span>
                {selectedKey.fingerprint ? <code>{selectedKey.fingerprint}</code> : null}
                {selectedKey.comment ? <span>{selectedKey.comment}</span> : null}
              </div>
            ) : (
              <p className="field-hint">OpenSSH config and ssh-agent decide which default key is used.</p>
            )}
          </div>
          <div className="field-stack">
            <label>
              Group
              <select value={form.groupId} onChange={(event) => update("groupId", event.target.value)}>
                <option value="">Ungrouped</option>
                {groups.map((group) => (
                  <option key={group.id} value={group.id}>
                    {group.name}
                  </option>
                ))}
              </select>
            </label>
            {selectedGroup ? (
              <div className="selected-meta inline">
                <span className="color-dot" style={{ backgroundColor: selectedGroup.color ?? "#3aa675" }} />
                <strong>{selectedGroup.name}</strong>
              </div>
            ) : (
              <p className="field-hint">This profile will stay in the ungrouped list.</p>
            )}
          </div>
          <label className="span-2">
            Tags
            <input value={form.tagText} onChange={(event) => update("tagText", event.target.value)} placeholder="linux, prod, nas" />
            <span className="field-hint">Use commas to separate tags.</span>
          </label>
          <label className="check-row span-2">
            <input type="checkbox" checked={form.favorite} onChange={(event) => update("favorite", event.target.checked)} />
            Favorite
          </label>
          <label className="span-2">
            Notes
            <textarea value={form.notes} onChange={(event) => update("notes", event.target.value)} rows={5} />
            <span className="field-hint">Notes are plaintext local metadata. Do not store secrets here.</span>
          </label>
        </div>

        <div className="modal-actions">
          <button type="button" className="button ghost" disabled={busy} onClick={onCancel}>
            Cancel
          </button>
          <button type="submit" className="button primary" disabled={busy}>
            Save server
          </button>
        </div>
      </form>
    </div>
  );
}

function GroupsPanel({
  groups,
  onSave,
  onDelete,
}: {
  groups: Group[];
  onSave: (input: GroupInput) => void;
  onDelete: (group: Group) => void;
}) {
  const [draft, setDraft] = useState<GroupInput>({ name: "", color: "#3aa675" });

  return (
    <section className="management-grid">
      <form
        className="panel edit-panel"
        onSubmit={(event) => {
          event.preventDefault();
          onSave(draft);
          setDraft({ name: "", color: "#3aa675" });
        }}
      >
        <h2>Add group</h2>
        <label>
          Name
          <input value={draft.name} onChange={(event) => setDraft({ ...draft, name: event.target.value })} required />
        </label>
        <label>
          Color
          <input type="color" value={draft.color ?? "#3aa675"} onChange={(event) => setDraft({ ...draft, color: event.target.value })} />
        </label>
        <button className="button primary" type="submit">
          <Plus size={17} />
          Save group
        </button>
      </form>

      <div className="list-panel">
        {groups.map((group) => (
          <div className="list-row" key={group.id}>
            <span className="color-dot" style={{ backgroundColor: group.color ?? "#3aa675" }} />
            <strong>{group.name}</strong>
            <button className="icon-button danger" aria-label={`Delete ${group.name}`} onClick={() => onDelete(group)}>
              <Trash2 size={17} />
            </button>
          </div>
        ))}
        {groups.length === 0 ? (
          <div className="empty-state compact">
            <Folder size={34} />
            <h3>No groups yet</h3>
            <p>Create groups to organize homelab hosts by role, site, or environment.</p>
          </div>
        ) : null}
      </div>
    </section>
  );
}

function KeysPanel({
  keys,
  onSave,
  onDelete,
}: {
  keys: SshKeyRef[];
  onSave: (input: SshKeyInput) => void;
  onDelete: (key: SshKeyRef) => void;
}) {
  const emptyDraft: SshKeyInput = { label: "", path: "", fingerprint: "", comment: "" };
  const [draft, setDraft] = useState<SshKeyInput>(emptyDraft);

  return (
    <section className="management-grid">
      <form
        className="panel edit-panel"
        onSubmit={(event) => {
          event.preventDefault();
          onSave({
            label: draft.label,
            path: draft.path,
            fingerprint: draft.fingerprint?.trim() || null,
            comment: draft.comment?.trim() || null,
          });
          setDraft(emptyDraft);
        }}
      >
        <h2>Add key reference</h2>
        <p className="muted">Only the key path and public metadata are stored.</p>
        <label>
          Label
          <input value={draft.label} onChange={(event) => setDraft({ ...draft, label: event.target.value })} required />
        </label>
        <label>
          Path
          <input value={draft.path} onChange={(event) => setDraft({ ...draft, path: event.target.value })} placeholder="~/.ssh/id_ed25519" required />
        </label>
        <label>
          Fingerprint
          <input
            value={draft.fingerprint ?? ""}
            onChange={(event) => setDraft({ ...draft, fingerprint: event.target.value })}
            placeholder="SHA256:..."
          />
        </label>
        <label>
          Comment
          <input value={draft.comment ?? ""} onChange={(event) => setDraft({ ...draft, comment: event.target.value })} placeholder="Laptop key" />
        </label>
        <button className="button primary" type="submit">
          <Plus size={17} />
          Save key reference
        </button>
      </form>

      <div className="list-panel">
        {keys.map((key) => (
          <div className="list-row key-row" key={key.id}>
            <KeyRound size={18} />
            <div>
              <strong>{key.label}</strong>
              <span>{key.path}</span>
              {key.fingerprint ? <code>{key.fingerprint}</code> : null}
              {key.comment ? <span>{key.comment}</span> : null}
            </div>
            <button className="icon-button danger" aria-label={`Delete ${key.label}`} onClick={() => onDelete(key)}>
              <Trash2 size={17} />
            </button>
          </div>
        ))}
        {keys.length === 0 ? (
          <div className="empty-state compact">
            <KeyRound size={34} />
            <h3>No key references yet</h3>
            <p>Add paths to existing OpenSSH keys. SSH-Buddy never stores private key contents.</p>
          </div>
        ) : null}
      </div>
    </section>
  );
}

function SettingsPanel({
  settings,
  onSave,
}: {
  settings: AppSettings;
  onSave: (settings: AppSettings) => void;
}) {
  const [draft, setDraft] = useState(settings);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  return (
    <section className="settings-grid">
      <form
        className="panel edit-panel"
        onSubmit={(event) => {
          event.preventDefault();
          onSave(draft);
        }}
      >
        <h2>Connection behavior</h2>
        <label>
          Terminal preference
          <select value={draft.terminalPreference} onChange={(event) => setDraft({ ...draft, terminalPreference: event.target.value })}>
            <option value="auto">Auto detect</option>
            <option value="konsole">Konsole</option>
            <option value="kitty">Kitty</option>
            <option value="alacritty">Alacritty</option>
            <option value="wezterm">WezTerm</option>
            <option value="gnome-terminal">GNOME Terminal</option>
            <option value="xterm">xterm</option>
          </select>
        </label>
        <label className="check-row">
          <input
            type="checkbox"
            checked={draft.safetyWarningsEnabled}
            onChange={(event) => setDraft({ ...draft, safetyWarningsEnabled: event.target.checked })}
          />
          Show safety warnings for risky features
        </label>
        <button className="button primary" type="submit">
          Save settings
        </button>
      </form>

      <section className="panel warning wide">
        <ShieldAlert size={21} />
        <div>
          <h3>Sudo and root workflows</h3>
          <p>
            SSH-Buddy does not automate sudo or store privileged credentials. Use normal sudo prompts, or carefully scoped sudoers rules for exact
            commands when you intentionally need passwordless homelab automation.
          </p>
        </div>
      </section>
    </section>
  );
}

function ImportDialog({
  onClose,
  onImported,
}: {
  onClose: () => void;
  onImported: (result: ImportResult) => void | Promise<void>;
}) {
  const [candidates, setCandidates] = useState<ImportCandidate[]>([]);
  const [selectedAliases, setSelectedAliases] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(true);
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const selectedCount = selectedAliases.size;

  useEffect(() => {
    api
      .importSshConfigPreview()
      .then((items) => {
        setCandidates(items);
        setSelectedAliases(new Set(items.filter((item) => item.selected && !item.skipped && !item.duplicate).map((item) => item.alias)));
      })
      .catch((cause: unknown) => {
        setError(cause instanceof Error ? cause.message : String(cause));
      })
      .finally(() => setLoading(false));
  }, []);

  async function importSelected() {
    if (selectedCount === 0 || importing) {
      return;
    }

    setError(null);
    setImporting(true);
    try {
      const result = await api.importSshConfig([...selectedAliases]);
      setImporting(false);
      await onImported(result);
    } catch (cause: unknown) {
      setImporting(false);
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="modal import-modal">
        <div className="modal-heading">
          <div>
            <p className="eyebrow">OpenSSH</p>
            <h2>Import ~/.ssh/config</h2>
          </div>
          <button type="button" className="icon-button" aria-label="Close" disabled={importing} onClick={onClose}>
            <X size={18} />
          </button>
        </div>

        {loading ? <p className="muted">Scanning SSH config...</p> : null}
        {error ? <div className="status-banner danger">{error}</div> : null}
        {!loading && candidates.length > 0 ? <div className="status-banner neutral">{formatImportPreviewSummary(candidates)}</div> : null}

        <div className="import-list">
          {candidates.map((candidate) => {
            const selectable = !candidate.skipped && !candidate.duplicate;
            return (
              <label className={selectable ? "import-row" : "import-row disabled"} key={candidate.alias}>
                <input
                  type="checkbox"
                  disabled={!selectable || importing}
                  checked={selectedAliases.has(candidate.alias)}
                  onChange={(event) => {
                    const next = new Set(selectedAliases);
                    if (event.target.checked) {
                      next.add(candidate.alias);
                    } else {
                      next.delete(candidate.alias);
                    }
                    setSelectedAliases(next);
                  }}
                />
                <div>
                  <div className="import-title-row">
                    <strong>{candidate.name}</strong>
                    <span className="import-badges">
                      {candidate.skipped ? <span className="import-badge warning">Skipped</span> : null}
                      {candidate.duplicate ? <span className="import-badge warning">Duplicate</span> : null}
                      {candidate.proxyJump ? <span className="import-badge">ProxyJump</span> : null}
                    </span>
                  </div>
                  <span>
                    {candidate.username ? `${candidate.username}@` : ""}
                    {candidate.host}:{candidate.port}
                  </span>
                  {candidate.identityFile ? <small>IdentityFile: {candidate.identityFile}</small> : null}
                  {candidate.proxyJump ? <small>ProxyJump: {candidate.proxyJump}</small> : null}
                  {candidate.warnings.length ? (
                    <small className="import-warnings">{candidate.warnings.join(" ")}</small>
                  ) : null}
                </div>
              </label>
            );
          })}
        </div>

        {!loading && candidates.length === 0 ? (
          <div className="empty-state compact">
            <Download size={34} />
            <h3>No importable hosts found</h3>
            <p>Only concrete Host aliases are imported in the MVP.</p>
          </div>
        ) : null}

        <div className="modal-actions">
          <button type="button" className="button ghost" disabled={importing} onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="button primary" disabled={selectedCount === 0 || importing} onClick={importSelected}>
            {importing ? "Importing..." : selectedCount > 0 ? `Import ${selectedCount} selected` : "No hosts selected"}
          </button>
        </div>
      </section>
    </div>
  );
}
