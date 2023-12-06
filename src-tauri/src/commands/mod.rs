use std::path::PathBuf;
use tauri::{AppHandle, api::path::data_dir};

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
        _ => data_dir().unwrap().join("moonlight-mod"),
    };

    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }

    dir
}
