use tauri::State;
use tauri_plugin_opener::OpenerExt;

use crate::{
    db::Database,
    domain::{
        AppResult, AppSettings, AppStateSnapshot, Group, GroupInput, ImportCandidate, ImportResult,
        RdpSettings, RdpSettingsInput, ServerInput, ServerProfile, SshKeyInput, SshKeyRef, Tunnel,
        TunnelInput, WebLink, WebLinkInput,
    },
    launcher::{
        build_rdp_launch_command, build_sftp_argv, build_ssh_argv, build_tunnel_argv,
        format_argv_for_display, launch_rdp as launch_rdp_client, launch_sftp_in_terminal,
        launch_ssh_in_terminal, launch_tunnel_in_terminal,
    },
};

#[tauri::command]
pub fn get_app_state(db: State<'_, Database>) -> AppResult<AppStateSnapshot> {
    Ok(AppStateSnapshot {
        servers: db.list_servers()?,
        groups: db.list_groups()?,
        tags: db.list_tags()?,
        ssh_keys: db.list_ssh_key_refs()?,
        settings: db.get_settings()?,
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
pub fn save_settings(input: AppSettings, db: State<'_, Database>) -> AppResult<AppSettings> {
    db.save_settings(input)
}

#[tauri::command]
pub fn get_ssh_command(server_id: String, db: State<'_, Database>) -> AppResult<String> {
    let server = db
        .get_server(&server_id)?
        .ok_or_else(|| "Server not found".to_string())?;
    let identity_file = identity_file_path(&db, &server)?;

    let argv = build_ssh_argv(&server, identity_file.as_deref())?;
    Ok(format_argv_for_display(&argv))
}

#[tauri::command]
pub fn launch_ssh(server_id: String, db: State<'_, Database>) -> AppResult<()> {
    let server = db
        .get_server(&server_id)?
        .ok_or_else(|| "Server not found".to_string())?;
    let identity_file = identity_file_path(&db, &server)?;
    let settings = db.get_settings()?;

    launch_ssh_in_terminal(&server, identity_file.as_deref(), &settings)
}

#[tauri::command]
pub fn get_sftp_command(server_id: String, db: State<'_, Database>) -> AppResult<String> {
    let server = db
        .get_server(&server_id)?
        .ok_or_else(|| "Server not found".to_string())?;
    let identity_file = identity_file_path(&db, &server)?;

    let argv = build_sftp_argv(&server, identity_file.as_deref())?;
    Ok(format_argv_for_display(&argv))
}

#[tauri::command]
pub fn launch_sftp(server_id: String, db: State<'_, Database>) -> AppResult<()> {
    let server = db
        .get_server(&server_id)?
        .ok_or_else(|| "Server not found".to_string())?;
    let identity_file = identity_file_path(&db, &server)?;
    let settings = db.get_settings()?;

    launch_sftp_in_terminal(&server, identity_file.as_deref(), &settings)
}

#[tauri::command]
pub fn get_rdp_settings(
    server_id: String,
    db: State<'_, Database>,
) -> AppResult<Option<RdpSettings>> {
    db.get_rdp_settings(&server_id)
}

#[tauri::command]
pub fn save_rdp_settings(
    server_id: String,
    input: RdpSettingsInput,
    db: State<'_, Database>,
) -> AppResult<RdpSettings> {
    db.save_rdp_settings(&server_id, input)
}

#[tauri::command]
pub fn delete_rdp_settings(server_id: String, db: State<'_, Database>) -> AppResult<()> {
    db.delete_rdp_settings(&server_id)
}

#[tauri::command]
pub fn get_rdp_command(server_id: String, db: State<'_, Database>) -> AppResult<String> {
    let server = db
        .get_server(&server_id)?
        .ok_or_else(|| "Server not found".to_string())?;
    let settings = db
        .get_rdp_settings(&server_id)?
        .ok_or_else(|| "RDP is not configured for this server".to_string())?;
    let command = build_rdp_launch_command(&server, &settings, crate::launcher::command_in_path)?;
    let mut argv = vec![command.program];
    argv.extend(command.args);
    Ok(format_argv_for_display(&argv))
}

#[tauri::command]
pub fn launch_rdp(server_id: String, db: State<'_, Database>) -> AppResult<()> {
    let server = db
        .get_server(&server_id)?
        .ok_or_else(|| "Server not found".to_string())?;
    let settings = db
        .get_rdp_settings(&server_id)?
        .ok_or_else(|| "RDP is not configured for this server".to_string())?;

    launch_rdp_client(&server, &settings)
}

#[tauri::command]
pub fn list_tunnels(server_id: String, db: State<'_, Database>) -> AppResult<Vec<Tunnel>> {
    db.list_tunnels(&server_id)
}

#[tauri::command]
pub fn create_tunnel(
    server_id: String,
    input: TunnelInput,
    db: State<'_, Database>,
) -> AppResult<Tunnel> {
    db.create_tunnel(&server_id, input)
}

#[tauri::command]
pub fn update_tunnel(id: String, input: TunnelInput, db: State<'_, Database>) -> AppResult<Tunnel> {
    db.update_tunnel(&id, input)
}

#[tauri::command]
pub fn delete_tunnel(id: String, db: State<'_, Database>) -> AppResult<()> {
    db.delete_tunnel(&id)
}

#[tauri::command]
pub fn get_tunnel_command(
    server_id: String,
    tunnel_id: String,
    db: State<'_, Database>,
) -> AppResult<String> {
    let server = db
        .get_server(&server_id)?
        .ok_or_else(|| "Server not found".to_string())?;
    let tunnel = db.get_tunnel_for_server(&server_id, &tunnel_id)?;
    let identity_file = identity_file_path(&db, &server)?;

    let argv = build_tunnel_argv(&server, identity_file.as_deref(), &tunnel)?;
    Ok(format_argv_for_display(&argv))
}

#[tauri::command]
pub fn launch_tunnel(
    server_id: String,
    tunnel_id: String,
    db: State<'_, Database>,
) -> AppResult<()> {
    let server = db
        .get_server(&server_id)?
        .ok_or_else(|| "Server not found".to_string())?;
    let tunnel = db.get_tunnel_for_server(&server_id, &tunnel_id)?;
    let identity_file = identity_file_path(&db, &server)?;
    let settings = db.get_settings()?;

    launch_tunnel_in_terminal(&server, identity_file.as_deref(), &tunnel, &settings)
}

#[tauri::command]
pub fn list_web_links(server_id: String, db: State<'_, Database>) -> AppResult<Vec<WebLink>> {
    db.list_web_links(&server_id)
}

#[tauri::command]
pub fn create_web_link(
    server_id: String,
    input: WebLinkInput,
    db: State<'_, Database>,
) -> AppResult<WebLink> {
    db.create_web_link(&server_id, input)
}

#[tauri::command]
pub fn update_web_link(
    id: String,
    input: WebLinkInput,
    db: State<'_, Database>,
) -> AppResult<WebLink> {
    db.update_web_link(&id, input)
}

#[tauri::command]
pub fn delete_web_link(id: String, db: State<'_, Database>) -> AppResult<()> {
    db.delete_web_link(&id)
}

#[tauri::command]
pub fn open_web_link(
    server_id: String,
    link_id: String,
    db: State<'_, Database>,
    app: tauri::AppHandle,
) -> AppResult<()> {
    let link = db.get_web_link_for_server(&server_id, &link_id)?;
    crate::domain::validate_web_link_url(&link.url)?;
    app.opener()
        .open_url(link.url, None::<&str>)
        .map_err(|error| format!("Failed to open web link: {error}"))
}

#[tauri::command]
pub fn import_ssh_config_preview(db: State<'_, Database>) -> AppResult<Vec<ImportCandidate>> {
    crate::ssh_config::import_preview(&db)
}

#[tauri::command]
pub fn import_ssh_config(aliases: Vec<String>, db: State<'_, Database>) -> AppResult<ImportResult> {
    crate::ssh_config::import_selected(&db, aliases)
}

fn identity_file_path(db: &Database, server: &ServerProfile) -> AppResult<Option<String>> {
    match &server.identity_file_id {
        Some(id) => db
            .key_path(id)?
            .map(Some)
            .ok_or_else(|| "Selected SSH key reference was not found".to_string()),
        None => Ok(None),
    }
}
