#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
