#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod domain;
mod launcher;
mod ssh_config;

use std::{fs, io};

use commands::{
    create_group, create_server, create_ssh_key_ref, create_tunnel, create_web_link, delete_group,
    delete_rdp_settings, delete_server, delete_ssh_key_ref, delete_tunnel, delete_web_link,
    get_app_state, get_install_public_key_command, get_rdp_command, get_rdp_settings,
    get_sftp_command, get_ssh_command, get_tunnel_command, import_ssh_config,
    import_ssh_config_preview, launch_install_public_key, launch_rdp, launch_sftp, launch_ssh,
    launch_tunnel, list_groups, list_servers, list_ssh_key_refs, list_tunnels, list_web_links,
    open_web_link, save_rdp_settings, save_settings, update_server, update_tunnel, update_web_link,
};
use db::Database;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("ssh-buddy.sqlite3");
            let database = Database::open(&db_path).map_err(io::Error::other)?;
            database.migrate().map_err(io::Error::other)?;
            app.manage(database);
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
