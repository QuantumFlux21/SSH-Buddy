#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod desktop;
mod domain;
mod launcher;
mod ssh_config;
mod status;

use std::{fs, io, path::Path};

use commands::{
    check_server_status, create_group, create_server, create_ssh_key_ref, create_tunnel,
    create_web_link, delete_group, delete_rdp_settings, delete_server, delete_ssh_key_ref,
    delete_tunnel, delete_web_link, get_app_state, get_install_public_key_command, get_rdp_command,
    get_rdp_settings, get_sftp_command, get_ssh_command, get_terminal_availability,
    get_tunnel_command, import_ssh_config, import_ssh_config_preview, launch_install_public_key,
    launch_rdp, launch_sftp, launch_ssh, launch_tunnel, list_groups, list_servers,
    list_ssh_key_refs, list_tunnels, list_web_links, open_web_link, save_rdp_settings,
    save_settings, scan_server_ports, test_terminal, update_server, update_tunnel, update_web_link,
};
use db::Database;
use desktop::{get_desktop_behavior_status, set_autostart_enabled, DesktopState};
use domain::default_settings;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            desktop::handle_second_instance(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .setup(|app| {
            let database_result = app
                .path()
                .app_data_dir()
                .map_err(|error| {
                    format!("Could not resolve the application data directory: {error}")
                })
                .and_then(|app_data_dir| initialize_database(&app_data_dir));
            let (database, database_ready, settings) = match database_result {
                Ok(database) => match database.get_settings() {
                    Ok(settings) => (database, true, settings),
                    Err(error) => (
                        Database::unavailable(database_recovery_message(&format!(
                            "Could not load application settings: {error}"
                        )))
                        .map_err(io::Error::other)?,
                        false,
                        default_settings(),
                    ),
                },
                Err(error) => (
                    Database::unavailable(database_recovery_message(&error))
                        .map_err(io::Error::other)?,
                    false,
                    default_settings(),
                ),
            };
            app.manage(database);
            app.manage(DesktopState::new(settings.close_to_tray));
            desktop::initialize(app.handle(), database_ready, &settings);

            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                let close_window = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        desktop::handle_close_request(&app_handle, &close_window, api);
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            list_servers,
            create_server,
            update_server,
            delete_server,
            list_groups,
            create_group,
            delete_group,
            list_ssh_key_refs,
            create_ssh_key_ref,
            delete_ssh_key_ref,
            save_settings,
            get_desktop_behavior_status,
            set_autostart_enabled,
            get_terminal_availability,
            test_terminal,
            get_ssh_command,
            launch_ssh,
            get_sftp_command,
            launch_sftp,
            get_install_public_key_command,
            launch_install_public_key,
            get_rdp_settings,
            save_rdp_settings,
            delete_rdp_settings,
            get_rdp_command,
            launch_rdp,
            list_tunnels,
            create_tunnel,
            update_tunnel,
            delete_tunnel,
            get_tunnel_command,
            launch_tunnel,
            check_server_status,
            scan_server_ports,
            list_web_links,
            create_web_link,
            update_web_link,
            delete_web_link,
            open_web_link,
            import_ssh_config_preview,
            import_ssh_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn initialize_database(app_data_dir: &Path) -> Result<Database, String> {
    fs::create_dir_all(app_data_dir).map_err(|error| {
        format!("Could not create or access the application data directory: {error}")
    })?;
    let db_path = app_data_dir.join("ssh-buddy.sqlite3");
    let database = Database::open(&db_path)
        .map_err(|error| format!("Could not open the local SQLite database: {error}"))?;
    database.migrate().map_err(|error| {
        format!("Could not verify or migrate the local SQLite database: {error}")
    })?;
    Ok(database)
}

fn database_recovery_message(error: &str) -> String {
    format!(
        "SSH-Buddy could not safely open its local data: {error} Database access is disabled for this session to prevent further changes. Close SSH-Buddy, verify app-data permissions and free space, then reopen it. If integrity or migration failed, preserve the database and restore a documented pre-migration backup before retrying."
    )
}
