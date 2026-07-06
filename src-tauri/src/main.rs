#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod domain;
mod launcher;

use std::{fs, io};

use commands::{
    create_group, create_server, create_ssh_key_ref, delete_group, delete_server,
    delete_ssh_key_ref, get_app_state, get_ssh_command, import_ssh_config,
    import_ssh_config_preview, launch_ssh, list_groups, list_servers, list_ssh_key_refs,
    open_web_link, save_settings, update_server,
};
use db::Database;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
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
            open_web_link,
            import_ssh_config_preview,
            import_ssh_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
