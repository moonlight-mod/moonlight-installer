#[cfg(unix)]
use nix::unistd::{Uid, User};

use crate::types::{Branch, DetectedInstall, InstallInfo};
use std::path::{Path, PathBuf};

pub const DOWNLOAD_DIR: &str = "dist";
pub const PATCHED_ASAR: &str = "_app.asar";

pub fn get_moonlight_dir() -> PathBuf {
    let dir = std::env::var_os("MOONLIGHT_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            Some(match std::env::consts::OS {
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
                _ => unimplemented!("Unsupported OS"),
            })
        })
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
            flatpak_id: None,
        },
        patched: app_dir.join(PATCHED_ASAR).exists(),
        has_config: false,
    })
}

pub fn get_app_dir(path: &Path) -> crate::Result<PathBuf> {
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

pub fn get_home_dir() -> PathBuf {
    #[cfg(windows)]
    unimplemented!();
    #[cfg(unix)]
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| {
            User::from_uid(Uid::effective())
                .ok()
                .flatten()
                .map(|u| u.dir)
        })
        .expect("$HOME to be set or user to be in /etc/passwd")
}

pub fn get_local_share() -> PathBuf {
    std::env::var_os("MOONLIGHT_DISCORD_SHARE_LINUX")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| get_home_dir().join(".local/share"))
}
