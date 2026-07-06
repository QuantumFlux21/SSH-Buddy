use tauri::State;

use crate::{
    db::Database,
    domain::{
        default_settings, AppResult, AppSettings, AppStateSnapshot, Group, GroupInput,
        ImportCandidate, ImportResult, ServerInput, ServerProfile, SshKeyInput, SshKeyRef,
    },
};

#[tauri::command]
pub fn get_app_state(db: State<'_, Database>) -> AppResult<AppStateSnapshot> {
    Ok(AppStateSnapshot {
        servers: db.list_servers()?,
        groups: db.list_groups()?,
        tags: db.list_tags()?,
        ssh_keys: db.list_ssh_key_refs()?,
        settings: default_settings(),
    })
}

#[tauri::command]
pub fn list_servers(db: State<'_, Database>) -> AppResult<Vec<ServerProfile>> {
    db.list_servers()
}

#[tauri::command]
pub fn create_server(input: ServerInput, db: State<'_, Database>) -> AppResult<ServerProfile> {
    db.create_server(input)
}

#[tauri::command]
pub fn update_server(
    id: String,
    input: ServerInput,
    db: State<'_, Database>,
) -> AppResult<ServerProfile> {
    db.update_server(&id, input)
}

#[tauri::command]
pub fn delete_server(id: String, db: State<'_, Database>) -> AppResult<()> {
    db.delete_server(&id)
}

#[tauri::command]
pub fn list_groups(db: State<'_, Database>) -> AppResult<Vec<Group>> {
    db.list_groups()
}

#[tauri::command]
pub fn create_group(input: GroupInput, db: State<'_, Database>) -> AppResult<Group> {
    db.create_group(input)
}

#[tauri::command]
pub fn delete_group(id: String, db: State<'_, Database>) -> AppResult<()> {
    db.delete_group(&id)
}

#[tauri::command]
pub fn list_ssh_key_refs(db: State<'_, Database>) -> AppResult<Vec<SshKeyRef>> {
    db.list_ssh_key_refs()
}

#[tauri::command]
pub fn create_ssh_key_ref(input: SshKeyInput, db: State<'_, Database>) -> AppResult<SshKeyRef> {
    db.create_ssh_key_ref(input)
}

#[tauri::command]
pub fn delete_ssh_key_ref(id: String, db: State<'_, Database>) -> AppResult<()> {
    db.delete_ssh_key_ref(&id)
}

#[tauri::command]
pub fn save_settings(_input: AppSettings) -> AppResult<AppSettings> {
    Err("Settings persistence is not implemented yet".to_string())
}

#[tauri::command]
pub fn get_ssh_command(server_id: String, db: State<'_, Database>) -> AppResult<String> {
    let server = db
        .get_server(&server_id)?
        .ok_or_else(|| "Server not found".to_string())?;
    let identity_file = match &server.identity_file_id {
        Some(id) => db.key_path(id)?,
        None => None,
    };

    Ok(build_ssh_command(&server, identity_file.as_deref()))
}

#[tauri::command]
pub fn launch_ssh(_server_id: String) -> AppResult<()> {
    Err("SSH launching is not implemented yet".to_string())
}

#[tauri::command]
pub fn open_web_link(_server_id: String, _link_id: String) -> AppResult<()> {
    Err("Web admin links are not implemented yet".to_string())
}

#[tauri::command]
pub fn import_ssh_config_preview() -> AppResult<Vec<ImportCandidate>> {
    Err("SSH config import is not implemented yet".to_string())
}

#[tauri::command]
pub fn import_ssh_config(_aliases: Vec<String>) -> AppResult<ImportResult> {
    Err("SSH config import is not implemented yet".to_string())
}

fn build_ssh_command(server: &ServerProfile, identity_file: Option<&str>) -> String {
    let mut parts = vec!["ssh".to_string()];
    if server.port != 22 {
        parts.push("-p".to_string());
        parts.push(server.port.to_string());
    }
    if let Some(identity_file) = identity_file {
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
