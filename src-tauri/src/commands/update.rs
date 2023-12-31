use super::{get_data_dir, get_moonlight_dir};
use crate::{
    types::{Error, MoonlightBranch},
    version::{download_nightly, download_stable, get_nightly_version, get_stable_version},
};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn get_moonlight_branch(app_handle: AppHandle) -> MoonlightBranch {
    let data_dir = get_data_dir(&app_handle);
    if data_dir.is_err() {
        return MoonlightBranch::Stable;
    }

    let branch_txt = data_dir.unwrap().join("branch.txt");
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
pub fn set_moonlight_branch(app_handle: AppHandle, branch: MoonlightBranch) -> Result<(), Error> {
    let data_dir = get_data_dir(&app_handle)?;
    let branch_txt = data_dir.join("branch.txt");

    let branch = match branch {
        MoonlightBranch::Stable => "stable",
        MoonlightBranch::Nightly => "nightly",
    };
    std::fs::write(branch_txt, branch)?;

    Ok(())
}

fn installed_version_changed(app_handle: &AppHandle, version: Option<String>) {
    app_handle
        .emit_all("installed_version_changed", version)
        .unwrap();
}

#[tauri::command]
pub fn get_downloaded_moonlight(app_handle: AppHandle) -> Option<String> {
    let data_dir = get_data_dir(&app_handle);
    if data_dir.is_err() {
        installed_version_changed(&app_handle, None);
        return None;
    }

    let version = data_dir.unwrap().join("version.txt");
    if !version.exists() {
        installed_version_changed(&app_handle, None);
        return None;
    }

    let version = std::fs::read_to_string(version).ok();
    installed_version_changed(&app_handle, version.clone());
    version
}

#[tauri::command]
pub fn get_latest_moonlight_version(branch: MoonlightBranch) -> Option<String> {
    match branch {
        // TODO
        MoonlightBranch::Stable => get_stable_version().ok(),
        MoonlightBranch::Nightly => get_nightly_version().ok(),
    }
}

#[tauri::command]
pub fn download_moonlight(app_handle: AppHandle, branch: MoonlightBranch) -> Result<(), Error> {
    let dir = get_moonlight_dir()?.join("dist");
    let version_txt = get_data_dir(&app_handle)?.join("version.txt");

    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }

    std::fs::create_dir_all(&dir)?;

    match branch {
        MoonlightBranch::Stable => download_stable(version_txt.clone(), dir)?,
        MoonlightBranch::Nightly => download_nightly(version_txt.clone(), dir)?,
    }

    let installed_version = std::fs::read_to_string(version_txt)?;
    installed_version_changed(&app_handle, Some(installed_version));

    Ok(())
}
