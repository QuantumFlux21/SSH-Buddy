import { useEffect, useMemo, useState } from "react";
import {
  Copy,
  Download,
  ExternalLink,
  Folder,
  FolderOpen,
  KeyRound,
  Pencil,
  Plus,
  Search,
  Server,
  Settings,
  ShieldAlert,
  Tag,
  Terminal,
  Trash2,
  X,
} from "lucide-react";
import { api } from "./lib/api";
import { filterServers, groupName } from "./lib/filters";
import { serverDestination, shortDate, tagInputValue } from "./lib/format";
import type {
  AppSettings,
  AppStateSnapshot,
  Group,
  GroupInput,
  ImportCandidate,
  ServerInput,
  ServerProfile,
  SshKeyInput,
  SshKeyRef,
  WebLinkInput,
} from "./lib/types";

type Section = "servers" | "groups" | "keys" | "settings";

interface ServerFormModel {
  id?: string | null;
  name: string;
  host: string;
  port: number;
  username: string;
  identityFile: string;
  groupId: string;
  notes: string;
  tagText: string;
  webLinks: WebLinkInput[];
}

const defaultWebLink = (): WebLinkInput => ({
  label: "",
  url: "",
  sortOrder: 0,
});

const newServerDraft = (server?: ServerProfile | null): ServerFormModel => ({
  id: server?.id ?? null,
  name: server?.name ?? "",
  host: server?.host ?? "",
  port: server?.port ?? 22,
  username: server?.username ?? "",
  identityFile: server?.identityFile ?? "",
  groupId: server?.groupId ?? "",
  notes: server?.notes ?? "",
  tagText: tagInputValue(server ?? null),
  webLinks: server?.webLinks.length ? server.webLinks.map((link) => ({ ...link })) : [defaultWebLink()],
});

function toServerInput(form: ServerFormModel): ServerInput {
  return {
    id: form.id ?? null,
    name: form.name.trim(),
    host: form.host.trim(),
    port: Number(form.port),
    username: form.username.trim(),
    identityFile: form.identityFile.trim() || null,
    groupId: form.groupId || null,
    notes: form.notes,
    tagNames: form.tagText
      .split(",")
      .map((tag) => tag.trim())
      .filter(Boolean),
    webLinks: form.webLinks
      .map((link, index) => ({
        ...link,
        label: link.label.trim(),
        url: link.url.trim(),
        sortOrder: index,
      }))
      .filter((link) => link.label || link.url),
  };
}

export default function App() {
  const [snapshot, setSnapshot] = useState<AppStateSnapshot | null>(null);
  const [activeSection, setActiveSection] = useState<Section>("servers");
  const [query, setQuery] = useState("");
  const [groupFilter, setGroupFilter] = useState<string | null>(null);
  const [selectedServerId, setSelectedServerId] = useState<string | null>(null);
  const [editingServer, setEditingServer] = useState<ServerFormModel | null>(null);
  const [showImport, setShowImport] = useState(false);
  const [busyMessage, setBusyMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function loadState(nextSelectedId?: string | null) {
    setError(null);
    const next = await api.getAppState();
    setSnapshot(next);
    if (nextSelectedId !== undefined) {
      setSelectedServerId(nextSelectedId);
    } else if (!selectedServerId && next.servers.length > 0) {
      setSelectedServerId(next.servers[0].id);
    }
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
    if (!snapshot) {
      return null;
    }
    return snapshot.servers.find((server) => server.id === selectedServerId) ?? filteredServers[0] ?? null;
  }, [snapshot, selectedServerId, filteredServers]);

  useEffect(() => {
    if (!selectedServer && filteredServers.length > 0) {
      setSelectedServerId(filteredServers[0].id);
    }
  }, [selectedServer, filteredServers]);

  async function runAction(label: string, action: () => Promise<void>) {
    setBusyMessage(label);
    setError(null);
    try {
      await action();
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusyMessage(null);
    }
  }

  async function saveServer(form: ServerFormModel) {
    await runAction("Saving server", async () => {
      const server = await api.saveServer(toServerInput(form));
      setEditingServer(null);
      await loadState(server.id);
    });
  }

  async function deleteServer(server: ServerProfile) {
    const confirmed = window.confirm(`Delete ${server.name}? This removes local metadata only.`);
    if (!confirmed) {
      return;
    }

    await runAction("Deleting server", async () => {
      await api.deleteServer(server.id);
      await loadState(null);
    });
  }

  async function copySshCommand(server: ServerProfile) {
    await runAction("Copying SSH command", async () => {
      const command = await api.getSshCommand(server.id);
      await navigator.clipboard.writeText(command);
    });
  }

  async function launchSsh(server: ServerProfile) {
    await runAction("Launching terminal", async () => {
      await api.launchSsh(server.id);
      await loadState(server.id);
    });
  }

  async function openWebLink(server: ServerProfile, linkId: string) {
    await runAction("Opening web link", async () => {
      await api.openWebLink(server.id, linkId);
    });
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
              <span className="color-dot" style={{ backgroundColor: group.color }} />
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
              <span className="server-row-name">{server.name}</span>
              <span className="server-row-host">{serverDestination(server)}</span>
            </button>
          ))}
          {filteredServers.length === 0 ? <div className="empty-mini">No matching servers</div> : null}
        </div>
      </aside>

      <main className="workspace">
        <header className="workspace-topbar">
          <div>
            <p className="eyebrow">{activeSection}</p>
            <h1>{sectionTitle(activeSection, selectedServer)}</h1>
          </div>
          <div className="topbar-actions">
            <button className="button ghost" onClick={() => setShowImport(true)}>
              <Download size={17} />
              Import SSH config
            </button>
            <button className="button primary" onClick={() => setEditingServer(newServerDraft())}>
              <Plus size={17} />
              Add server
            </button>
          </div>
        </header>

        {error ? <div className="status-banner danger">{error}</div> : null}
        {busyMessage ? <div className="status-banner">{busyMessage}...</div> : null}

        {activeSection === "servers" ? (
          <ServerDetails
            server={selectedServer}
            groups={snapshot.groups}
            onEdit={(server) => setEditingServer(newServerDraft(server))}
            onDelete={deleteServer}
            onCopyCommand={copySshCommand}
            onLaunch={launchSsh}
            onOpenWebLink={openWebLink}
          />
        ) : null}

        {activeSection === "groups" ? (
          <GroupsPanel
            groups={snapshot.groups}
            onSave={async (input) => {
              await runAction("Saving group", async () => {
                await api.saveGroup(input);
                await loadState();
              });
            }}
            onDelete={async (group) => {
              await runAction("Deleting group", async () => {
                await api.deleteGroup(group.id);
                await loadState();
              });
            }}
          />
        ) : null}

        {activeSection === "keys" ? (
          <KeysPanel
            keys={snapshot.sshKeys}
            onSave={async (input) => {
              await runAction("Saving key reference", async () => {
                await api.saveSshKey(input);
                await loadState();
              });
            }}
            onDelete={async (key) => {
              await runAction("Deleting key reference", async () => {
                await api.deleteSshKey(key.id);
                await loadState();
              });
            }}
          />
        ) : null}

        {activeSection === "settings" ? (
          <SettingsPanel
            settings={snapshot.settings}
            onSave={async (settings) => {
              await runAction("Saving settings", async () => {
                await api.saveSettings(settings);
                await loadState();
              });
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
        />
      ) : null}

      {showImport ? (
        <ImportDialog
          onClose={() => setShowImport(false)}
          onImported={async (firstServerId) => {
            setShowImport(false);
            await loadState(firstServerId);
          }}
        />
      ) : null}
    </div>
  );
}

function sectionTitle(section: Section, server: ServerProfile | null) {
  if (section === "servers") {
    return server?.name ?? "Servers";
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
  onEdit,
  onDelete,
  onCopyCommand,
  onLaunch,
  onOpenWebLink,
}: {
  server: ServerProfile | null;
  groups: Group[];
  onEdit: (server: ServerProfile) => void;
  onDelete: (server: ServerProfile) => void;
  onCopyCommand: (server: ServerProfile) => void;
  onLaunch: (server: ServerProfile) => void;
  onOpenWebLink: (server: ServerProfile, linkId: string) => void;
}) {
  if (!server) {
    return (
      <section className="empty-state">
        <Server size={42} />
        <h2>No server selected</h2>
        <p>Add a server or import entries from your OpenSSH config.</p>
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
            <p>{groupName(groups, server.groupId)} · Last connected {shortDate(server.lastConnectedAt)}</p>
          </div>
          <div className="hero-actions">
            <button className="icon-button" aria-label="Edit server" title="Edit server" onClick={() => onEdit(server)}>
              <Pencil size={18} />
            </button>
            <button className="icon-button danger" aria-label="Delete server" title="Delete server" onClick={() => onDelete(server)}>
              <Trash2 size={18} />
            </button>
          </div>
        </div>

        <div className="action-strip">
          <button className="button primary" onClick={() => onLaunch(server)}>
            <Terminal size={17} />
            Open SSH
          </button>
          <button className="button" onClick={() => onCopyCommand(server)}>
            <Copy size={17} />
            Copy command
          </button>
          {server.webLinks.map((link) => (
            <button className="button" key={link.id} onClick={() => onOpenWebLink(server, link.id)}>
              <ExternalLink size={17} />
              {link.label}
            </button>
          ))}
        </div>

        <div className="info-grid">
          <Info label="Host" value={server.host} />
          <Info label="Port" value={String(server.port)} />
          <Info label="Username" value={server.username || "Default OpenSSH user"} />
          <Info label="Identity file" value={server.identityFile || "OpenSSH default"} />
        </div>

        <section className="panel">
          <div className="panel-heading">
            <h3>Notes</h3>
            <span>No secrets here</span>
          </div>
          <p className={server.notes.trim() ? "notes" : "muted"}>{server.notes.trim() || "No notes saved for this server."}</p>
        </section>
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
            <h3>Actions</h3>
            <span>{server.actions.length}</span>
          </div>
          <div className="action-list">
            {server.actions.map((action) => (
              <div key={action.id} className="action-row">
                <span>{action.label}</span>
                <code>{action.type}</code>
              </div>
            ))}
          </div>
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

function ServerForm({
  form,
  groups,
  keyRefs,
  onChange,
  onCancel,
  onSave,
}: {
  form: ServerFormModel;
  groups: Group[];
  keyRefs: SshKeyRef[];
  onChange: (form: ServerFormModel) => void;
  onCancel: () => void;
  onSave: (form: ServerFormModel) => void;
}) {
  const update = <Key extends keyof ServerFormModel>(key: Key, value: ServerFormModel[Key]) => {
    onChange({ ...form, [key]: value });
  };

  const updateWebLink = (index: number, patch: Partial<WebLinkInput>) => {
    onChange({
      ...form,
      webLinks: form.webLinks.map((link, itemIndex) => (itemIndex === index ? { ...link, ...patch } : link)),
    });
  };

  return (
    <div className="modal-backdrop" role="presentation">
      <form
        className="modal"
        onSubmit={(event) => {
          event.preventDefault();
          onSave(form);
        }}
      >
        <div className="modal-heading">
          <div>
            <p className="eyebrow">Server profile</p>
            <h2>{form.id ? "Edit server" : "Add server"}</h2>
          </div>
          <button type="button" className="icon-button" aria-label="Close" onClick={onCancel}>
            <X size={18} />
          </button>
        </div>

        <div className="form-grid">
          <label>
            Name
            <input value={form.name} onChange={(event) => update("name", event.target.value)} required />
          </label>
          <label>
            Hostname or IP
            <input value={form.host} onChange={(event) => update("host", event.target.value)} required />
          </label>
          <label>
            Port
            <input
              type="number"
              min={1}
              max={65535}
              value={form.port}
              onChange={(event) => update("port", Number(event.target.value))}
              required
            />
          </label>
          <label>
            Username
            <input value={form.username} onChange={(event) => update("username", event.target.value)} placeholder="OpenSSH default" />
          </label>
          <label>
            Identity file
            <input
              value={form.identityFile}
              onChange={(event) => update("identityFile", event.target.value)}
              placeholder="~/.ssh/id_ed25519"
              list="known-keys"
            />
            <datalist id="known-keys">
              {keyRefs.map((key) => (
                <option key={key.id} value={key.path}>
                  {key.label}
                </option>
              ))}
            </datalist>
          </label>
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
          <label className="span-2">
            Tags
            <input value={form.tagText} onChange={(event) => update("tagText", event.target.value)} placeholder="linux, prod, nas" />
          </label>
          <label className="span-2">
            Notes
            <textarea value={form.notes} onChange={(event) => update("notes", event.target.value)} rows={5} />
          </label>
        </div>

        <div className="subform">
          <div className="panel-heading">
            <h3>Web admin links</h3>
            <button
              type="button"
              className="button compact"
              onClick={() => onChange({ ...form, webLinks: [...form.webLinks, defaultWebLink()] })}
            >
              <Plus size={15} />
              Add link
            </button>
          </div>
          {form.webLinks.map((link, index) => (
            <div className="web-link-row" key={index}>
              <input value={link.label} onChange={(event) => updateWebLink(index, { label: event.target.value })} placeholder="Proxmox" />
              <input value={link.url} onChange={(event) => updateWebLink(index, { url: event.target.value })} placeholder="https://host:8006" />
              <button
                type="button"
                className="icon-button"
                aria-label="Remove web link"
                onClick={() => onChange({ ...form, webLinks: form.webLinks.filter((_, itemIndex) => itemIndex !== index) })}
              >
                <Trash2 size={16} />
              </button>
            </div>
          ))}
        </div>

        <div className="modal-actions">
          <button type="button" className="button ghost" onClick={onCancel}>
            Cancel
          </button>
          <button type="submit" className="button primary">
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
          <input type="color" value={draft.color} onChange={(event) => setDraft({ ...draft, color: event.target.value })} />
        </label>
        <button className="button primary" type="submit">
          <Plus size={17} />
          Save group
        </button>
      </form>

      <div className="list-panel">
        {groups.map((group) => (
          <div className="list-row" key={group.id}>
            <span className="color-dot" style={{ backgroundColor: group.color }} />
            <strong>{group.name}</strong>
            <button className="icon-button danger" aria-label={`Delete ${group.name}`} onClick={() => onDelete(group)}>
              <Trash2 size={17} />
            </button>
          </div>
        ))}
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
  const [draft, setDraft] = useState<SshKeyInput>({ label: "", path: "" });

  return (
    <section className="management-grid">
      <form
        className="panel edit-panel"
        onSubmit={(event) => {
          event.preventDefault();
          onSave(draft);
          setDraft({ label: "", path: "" });
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
            </div>
            <button className="icon-button danger" aria-label={`Delete ${key.label}`} onClick={() => onDelete(key)}>
              <Trash2 size={17} />
            </button>
          </div>
        ))}
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

function ImportDialog({ onClose, onImported }: { onClose: () => void; onImported: (firstServerId: string | null) => void }) {
  const [candidates, setCandidates] = useState<ImportCandidate[]>([]);
  const [selectedAliases, setSelectedAliases] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .importSshConfigPreview()
      .then((items) => {
        setCandidates(items);
        setSelectedAliases(new Set(items.filter((item) => item.selected).map((item) => item.alias)));
      })
      .catch((cause: unknown) => {
        setError(cause instanceof Error ? cause.message : String(cause));
      })
      .finally(() => setLoading(false));
  }, []);

  async function importSelected() {
    setError(null);
    try {
      const result = await api.importSshConfig([...selectedAliases]);
      onImported(result.servers[0]?.id ?? null);
    } catch (cause: unknown) {
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
          <button type="button" className="icon-button" aria-label="Close" onClick={onClose}>
            <X size={18} />
          </button>
        </div>

        {loading ? <p className="muted">Scanning SSH config...</p> : null}
        {error ? <div className="status-banner danger">{error}</div> : null}

        <div className="import-list">
          {candidates.map((candidate) => (
            <label className="import-row" key={candidate.alias}>
              <input
                type="checkbox"
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
                <strong>{candidate.name}</strong>
                <span>
                  {candidate.username ? `${candidate.username}@` : ""}
                  {candidate.host}:{candidate.port}
                </span>
                {candidate.warnings.length ? <small>{candidate.warnings.join(" ")}</small> : null}
              </div>
            </label>
          ))}
        </div>

        {!loading && candidates.length === 0 ? (
          <div className="empty-state compact">
            <Download size={34} />
            <h3>No importable hosts found</h3>
            <p>Only concrete Host aliases are imported in the MVP.</p>
          </div>
        ) : null}

        <div className="modal-actions">
          <button type="button" className="button ghost" onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="button primary" disabled={selectedAliases.size === 0} onClick={importSelected}>
            Import selected
          </button>
        </div>
      </section>
    </div>
  );
}
