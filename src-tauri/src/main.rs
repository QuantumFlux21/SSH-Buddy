#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Group {
    id: String,
    name: String,
    color: String,
    sort_order: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Tag {
    id: String,
    name: String,
    color: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshKeyRef {
    id: String,
    label: String,
    path: String,
    fingerprint: Option<String>,
    comment: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebLink {
    id: String,
    server_id: String,
    label: String,
    url: String,
    sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionAction {
    id: String,
    server_id: String,
    #[serde(rename = "type")]
    action_type: String,
    label: String,
    enabled: bool,
    sort_order: i64,
    config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerProfile {
    id: String,
    name: String,
    host: String,
    port: u16,
    username: String,
    identity_file: Option<String>,
    group_id: Option<String>,
    notes: String,
    tags: Vec<Tag>,
    web_links: Vec<WebLink>,
    actions: Vec<ConnectionAction>,
    created_at: String,
    updated_at: String,
    last_connected_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    terminal_preference: String,
    safety_warnings_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppStateSnapshot {
    servers: Vec<ServerProfile>,
    groups: Vec<Group>,
    tags: Vec<Tag>,
    ssh_keys: Vec<SshKeyRef>,
    settings: AppSettings,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebLinkInput {
    id: Option<String>,
    label: String,
    url: String,
    sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerInput {
    id: Option<String>,
    name: String,
    host: String,
    port: u16,
    username: String,
    identity_file: Option<String>,
    group_id: Option<String>,
    notes: String,
    tag_names: Vec<String>,
    web_links: Vec<WebLinkInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GroupInput {
    id: Option<String>,
    name: String,
    color: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshKeyInput {
    id: Option<String>,
    label: String,
    path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportCandidate {
    alias: String,
    name: String,
    host: String,
    port: u16,
    username: String,
    identity_file: Option<String>,
    warnings: Vec<String>,
    selected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportResult {
    imported: usize,
    skipped: usize,
    servers: Vec<ServerProfile>,
}

#[derive(Debug)]
struct AppState {
    snapshot: Mutex<AppStateSnapshot>,
}

impl AppState {
    fn new() -> Self {
        Self {
            snapshot: Mutex::new(AppStateSnapshot {
                servers: Vec::new(),
                groups: vec![Group {
                    id: "grp_homelab".to_string(),
                    name: "Homelab".to_string(),
                    color: "#3aa675".to_string(),
                    sort_order: 0,
                    created_at: timestamp(),
                    updated_at: timestamp(),
                }],
                tags: Vec::new(),
                ssh_keys: vec![SshKeyRef {
                    id: "key_default".to_string(),
                    label: "Default OpenSSH key".to_string(),
                    path: "~/.ssh/id_ed25519".to_string(),
                    fingerprint: None,
                    comment: None,
                    created_at: timestamp(),
                    updated_at: timestamp(),
                }],
                settings: AppSettings {
                    terminal_preference: "auto".to_string(),
                    safety_warnings_enabled: true,
                },
            }),
        }
    }
}

#[tauri::command]
fn get_app_state(state: State<'_, AppState>) -> Result<AppStateSnapshot, String> {
    state
        .snapshot
        .lock()
        .map(|snapshot| snapshot.clone())
        .map_err(|_| "App state lock was poisoned".to_string())
}

#[tauri::command]
fn save_server(input: ServerInput, state: State<'_, AppState>) -> Result<ServerProfile, String> {
    validate_server(&input)?;

    let mut snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    let server_id = input.id.clone().unwrap_or_else(|| make_id("srv"));
    let existing = snapshot
        .servers
        .iter()
        .find(|server| server.id == server_id)
        .cloned();
    let now = timestamp();
    let tags = input
        .tag_names
        .iter()
        .filter_map(|name| normalize_name(name))
        .map(|name| ensure_tag(&mut snapshot.tags, name))
        .collect::<Vec<_>>();
    let web_links = input
        .web_links
        .iter()
        .filter(|link| !link.label.trim().is_empty() || !link.url.trim().is_empty())
        .enumerate()
        .map(|(index, link)| WebLink {
            id: link.id.clone().unwrap_or_else(|| make_id("web")),
            server_id: server_id.clone(),
            label: fallback_label(&link.label, "Web admin"),
            url: link.url.trim().to_string(),
            sort_order: if link.sort_order >= 0 {
                link.sort_order
            } else {
                index as i64
            },
        })
        .collect::<Vec<_>>();
    let mut actions = vec![
        ConnectionAction {
            id: make_id("act"),
            server_id: server_id.clone(),
            action_type: "ssh".to_string(),
            label: "Open SSH session".to_string(),
            enabled: true,
            sort_order: 0,
            config: serde_json::json!({}),
        },
        ConnectionAction {
            id: make_id("act"),
            server_id: server_id.clone(),
            action_type: "copy-command".to_string(),
            label: "Copy SSH command".to_string(),
            enabled: true,
            sort_order: 1,
            config: serde_json::json!({}),
        },
    ];
    actions.extend(
        web_links
            .iter()
            .enumerate()
            .map(|(index, link)| ConnectionAction {
                id: make_id("act"),
                server_id: server_id.clone(),
                action_type: "web".to_string(),
                label: link.label.clone(),
                enabled: true,
                sort_order: index as i64 + 2,
                config: serde_json::json!({ "webLinkId": link.id }),
            }),
    );

    let server = ServerProfile {
        id: server_id,
        name: input.name.trim().to_string(),
        host: input.host.trim().to_string(),
        port: input.port,
        username: input.username.trim().to_string(),
        identity_file: input.identity_file.filter(|value| !value.trim().is_empty()),
        group_id: input.group_id.filter(|value| !value.trim().is_empty()),
        notes: input.notes,
        tags,
        web_links,
        actions,
        created_at: existing
            .as_ref()
            .map(|server| server.created_at.clone())
            .unwrap_or_else(|| now.clone()),
        updated_at: now,
        last_connected_at: existing.and_then(|server| server.last_connected_at),
    };

    if let Some(index) = snapshot
        .servers
        .iter()
        .position(|existing_server| existing_server.id == server.id)
    {
        snapshot.servers[index] = server.clone();
    } else {
        snapshot.servers.push(server.clone());
    }

    Ok(server)
}

#[tauri::command]
fn delete_server(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    snapshot.servers.retain(|server| server.id != id);
    Ok(())
}

#[tauri::command]
fn save_group(input: GroupInput, state: State<'_, AppState>) -> Result<Group, String> {
    let name = normalize_name(&input.name).ok_or_else(|| "Group name is required".to_string())?;
    let mut snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    let group_id = input.id.unwrap_or_else(|| make_id("grp"));
    let now = timestamp();
    let group = Group {
        id: group_id,
        name,
        color: input.color,
        sort_order: snapshot.groups.len() as i64,
        created_at: now.clone(),
        updated_at: now,
    };

    if let Some(index) = snapshot
        .groups
        .iter()
        .position(|existing_group| existing_group.id == group.id)
    {
        snapshot.groups[index] = group.clone();
    } else {
        snapshot.groups.push(group.clone());
    }

    Ok(group)
}

#[tauri::command]
fn delete_group(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    snapshot.groups.retain(|group| group.id != id);
    for server in &mut snapshot.servers {
        if server.group_id.as_deref() == Some(&id) {
            server.group_id = None;
        }
    }
    Ok(())
}

#[tauri::command]
fn save_ssh_key(input: SshKeyInput, state: State<'_, AppState>) -> Result<SshKeyRef, String> {
    let label = normalize_name(&input.label).ok_or_else(|| "Key label is required".to_string())?;
    let path = normalize_name(&input.path).ok_or_else(|| "Key path is required".to_string())?;
    let mut snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    let key_id = input.id.unwrap_or_else(|| make_id("key"));
    let now = timestamp();
    let key = SshKeyRef {
        id: key_id,
        label,
        path,
        fingerprint: None,
        comment: None,
        created_at: now.clone(),
        updated_at: now,
    };

    if let Some(index) = snapshot
        .ssh_keys
        .iter()
        .position(|existing_key| existing_key.id == key.id)
    {
        snapshot.ssh_keys[index] = key.clone();
    } else {
        snapshot.ssh_keys.push(key.clone());
    }

    Ok(key)
}

#[tauri::command]
fn delete_ssh_key(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    snapshot.ssh_keys.retain(|key| key.id != id);
    Ok(())
}

#[tauri::command]
fn save_settings(input: AppSettings, state: State<'_, AppState>) -> Result<AppSettings, String> {
    let mut snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    snapshot.settings = input;
    Ok(snapshot.settings.clone())
}

#[tauri::command]
fn get_ssh_command(server_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    let server = snapshot
        .servers
        .iter()
        .find(|server| server.id == server_id)
        .ok_or_else(|| "Server not found".to_string())?;
    Ok(build_ssh_command(server))
}

#[tauri::command]
fn launch_ssh(server_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    let server = snapshot
        .servers
        .iter_mut()
        .find(|server| server.id == server_id)
        .ok_or_else(|| "Server not found".to_string())?;
    server.last_connected_at = Some(timestamp());
    Ok(())
}

#[tauri::command]
fn open_web_link(
    server_id: String,
    link_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let snapshot = state
        .snapshot
        .lock()
        .map_err(|_| "App state lock was poisoned".to_string())?;
    let server = snapshot
        .servers
        .iter()
        .find(|server| server.id == server_id)
        .ok_or_else(|| "Server not found".to_string())?;
    let link = server
        .web_links
        .iter()
        .find(|link| link.id == link_id)
        .ok_or_else(|| "Web link not found".to_string())?;
    validate_web_url(&link.url)?;
    Ok(())
}

#[tauri::command]
fn import_ssh_config_preview() -> Result<Vec<ImportCandidate>, String> {
    Ok(Vec::new())
}

#[tauri::command]
fn import_ssh_config(_aliases: Vec<String>) -> Result<ImportResult, String> {
    Ok(ImportResult {
        imported: 0,
        skipped: 0,
        servers: Vec::new(),
    })
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            save_server,
            delete_server,
            save_group,
            delete_group,
            save_ssh_key,
            delete_ssh_key,
            save_settings,
            get_ssh_command,
            launch_ssh,
            open_web_link,
            import_ssh_config_preview,
            import_ssh_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn validate_server(input: &ServerInput) -> Result<(), String> {
    if input.name.trim().is_empty() {
        return Err("Server name is required".to_string());
    }
    if input.host.trim().is_empty() {
        return Err("Host is required".to_string());
    }
    if input.port == 0 {
        return Err("Port must be between 1 and 65535".to_string());
    }
    for link in &input.web_links {
        if !link.url.trim().is_empty() {
            validate_web_url(&link.url)?;
        }
    }
    Ok(())
}

fn validate_web_url(url: &str) -> Result<(), String> {
    let value = url.trim();
    if value.starts_with("http://") || value.starts_with("https://") {
        Ok(())
    } else {
        Err("Web links must start with http:// or https://".to_string())
    }
}

fn normalize_name(value: &str) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn fallback_label(value: &str, fallback: &str) -> String {
    normalize_name(value).unwrap_or_else(|| fallback.to_string())
}

fn ensure_tag(tags: &mut Vec<Tag>, name: String) -> Tag {
    if let Some(tag) = tags
        .iter()
        .find(|tag| tag.name.eq_ignore_ascii_case(&name))
        .cloned()
    {
        return tag;
    }

    let now = timestamp();
    let tag = Tag {
        id: make_id("tag"),
        name,
        color: "#4da3ff".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    tags.push(tag.clone());
    tag
}

fn build_ssh_command(server: &ServerProfile) -> String {
    let mut parts = vec!["ssh".to_string()];
    if server.port != 22 {
        parts.push("-p".to_string());
        parts.push(server.port.to_string());
    }
    if let Some(identity_file) = &server.identity_file {
        parts.push("-i".to_string());
        parts.push(shell_quote(identity_file));
    }
    let destination = if server.username.trim().is_empty() {
        server.host.clone()
    } else {
        format!("{}@{}", server.username, server.host)
    };
    parts.push(shell_quote(&destination));
    parts.join(" ")
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "@%_+=:,./-".contains(character))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn make_id(prefix: &str) -> String {
    format!(
        "{}_{}",
        prefix,
        timestamp().replace(['-', ':', '.', 'Z'], "")
    )
}

fn timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis()))
        .unwrap_or_else(|_| "0.000Z".to_string())
}
