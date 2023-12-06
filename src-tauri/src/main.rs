#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod commands;
mod types;
mod version;


fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::patch::detect_installs,
            commands::patch::is_install_patched,
            commands::patch::patch_install,
            commands::patch::unpatch_install,
            commands::update::get_moonlight_branch,
            commands::update::set_moonlight_branch,
            commands::update::get_downloaded_moonlight,
            commands::update::get_latest_moonlight_version,
            commands::update::download_moonlight,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
              let window = app.get_window("main").unwrap();
              window.open_devtools();
              window.close_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
