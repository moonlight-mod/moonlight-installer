use crate::types::{Branch, DetectedInstall, InstallInfo, InstallerResult};
use std::path::{Path, PathBuf};

const DOWNLOAD_DIR: &str = "dist";
pub const PATCHED_ASAR: &str = "_app.asar";

pub fn get_moonlight_dir() -> PathBuf {
    let dir = std::env::var_os("MOONLIGHT_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::config_dir().map(|d| d.join("moonlight-mod")))
        .unwrap();

    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }

    dir
}

#[must_use]
pub fn detect_install_type(exe: &Path) -> Option<Branch> {
    let name = exe.file_name()?.to_string_lossy();

    // lmfao
    if name.contains("PTB") {
        Some(Branch::PTB)
    } else if name.contains("Canary") {
        Some(Branch::Canary)
    } else if name.contains("Development") {
        Some(Branch::Development)
    } else {
        Some(Branch::Stable)
    }
}

#[must_use]
pub fn detect_install(exe: &Path) -> Option<InstallInfo> {
    let folder = exe.parent()?;
    let install_type = detect_install_type(exe)?;
    let app_dir = get_app_dir(folder).ok()?;

    Some(InstallInfo {
        install: DetectedInstall {
            branch: install_type,
            path: folder.to_path_buf(),
        },
        patched: app_dir.join(PATCHED_ASAR).exists(),
        has_config: false,
    })
}

pub fn get_app_dir(path: &Path) -> InstallerResult<PathBuf> {
    match std::env::consts::OS {
        "windows" | "linux" => Ok(path.join("resources")),
        "macos" => Ok(path.to_path_buf()),
        _ => unimplemented!("Unsupported OS"),
    }
}

#[must_use]
pub fn get_download_dir() -> PathBuf {
    get_moonlight_dir().join(DOWNLOAD_DIR)
}
