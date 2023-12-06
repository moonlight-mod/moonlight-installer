use std::path::PathBuf;
use tauri::AppHandle;

pub mod patch;
pub mod update;

pub fn get_data_dir(app_handle: &AppHandle) -> PathBuf {
    app_handle.path_resolver().app_data_dir().unwrap()
}

pub fn get_moonlight_dir(app_handle: &AppHandle) -> PathBuf {
    // Reimplementation of electron's appdata stuff
    let dir = match std::env::consts::OS {
        "windows" => {
            let appdata = std::env::var("APPDATA").unwrap();
            PathBuf::from(appdata).join("moonlight-mod")
        }
        // TODO
        _ => get_data_dir(app_handle).join("moonlight"),
    };

    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }

    dir
}
