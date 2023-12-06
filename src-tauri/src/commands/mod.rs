use crate::types::Error;
use std::path::PathBuf;
use tauri::AppHandle;

pub mod patch;
pub mod update;

pub fn get_data_dir(app_handle: &AppHandle) -> Result<PathBuf, Error> {
    app_handle.path_resolver().app_data_dir().ok_or(Error {
        message: "could not get data dir".to_string(),
    })
}

pub fn get_moonlight_dir() -> Result<PathBuf, Error> {
    // Reimplementation of electron's appdata stuff
    let dir = match std::env::consts::OS {
        "windows" => {
            let appdata = std::env::var("APPDATA").unwrap();
            PathBuf::from(appdata).join("moonlight-mod")
        }
        "macos" => {
            let home = std::env::var("HOME").unwrap();
            PathBuf::from(home).join("Library/Application Support/moonlight-mod")
        }
        "linux" => {
            let home = std::env::var("HOME").unwrap();
            PathBuf::from(home).join(".config/moonlight-mod")
        }
        _ => panic!("unsupported platform"),
    };

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}
