use super::{get_data_dir, get_moonlight_dir};
use crate::{
    types::MoonlightBranch,
    version::{download_nightly, get_nightly_version},
};
use tauri::AppHandle;

#[tauri::command]
pub fn get_moonlight_branch(app_handle: AppHandle) -> MoonlightBranch {
    let data_dir = get_data_dir(&app_handle);
    let branch_txt = data_dir.join("branch.txt");

    if !branch_txt.exists() {
        return MoonlightBranch::Stable;
    }

    let branch = std::fs::read_to_string(branch_txt).unwrap();
    match branch.as_str() {
        "stable" => MoonlightBranch::Stable,
        "nightly" => MoonlightBranch::Nightly,
        _ => MoonlightBranch::Stable,
    }
}

#[tauri::command]
pub fn set_moonlight_branch(app_handle: AppHandle, branch: MoonlightBranch) {
    let data_dir = get_data_dir(&app_handle);
    let branch_txt = data_dir.join("branch.txt");

    let branch = match branch {
        MoonlightBranch::Stable => "stable",
        MoonlightBranch::Nightly => "nightly",
    };
    std::fs::write(branch_txt, branch).unwrap();
}

#[tauri::command]
pub fn get_downloaded_moonlight(app_handle: AppHandle) -> Option<String> {
    let data_dir = get_data_dir(&app_handle);
    let version = data_dir.join("version.txt");

    if !version.exists() {
        return None;
    }

    std::fs::read_to_string(version).ok()
}

#[tauri::command]
pub fn get_latest_moonlight_version(branch: MoonlightBranch) -> Option<String> {
    match branch {
        // TODO
        MoonlightBranch::Stable => None,
        MoonlightBranch::Nightly => get_nightly_version().ok(),
    }
}

#[tauri::command]
pub fn download_moonlight(app_handle: AppHandle, branch: MoonlightBranch) {
    let dir = get_moonlight_dir(&app_handle).join("dist");
    let version_txt = get_data_dir(&app_handle).join("version.txt");

    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }

    std::fs::create_dir_all(&dir).unwrap();

    match branch {
        // TODO
        MoonlightBranch::Stable => (),
        MoonlightBranch::Nightly => download_nightly(version_txt, dir).unwrap(),
    }
}
