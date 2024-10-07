use crate::types::{DetectedInstall, InstallInfo, InstallerResult};

use super::types::{Branch, MoonlightBranch};
use std::path::{Path, PathBuf};

const DOWNLOAD_DIR: &str = "dist";
pub const PATCHED_ASAR: &str = "_app.asar";

// Reimplementation of electron's appdata stuff
// Don't like to unwrap here but also if this is broken the entire app is broken
pub fn get_moonlight_dir() -> PathBuf {
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
        _ => unimplemented!("Unsupported OS"),
    };

    if !dir.exists() {
        std::fs::create_dir_all(&dir).ok();
    }

    dir
}

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

pub fn get_branch_config(branch: Branch) -> PathBuf {
    get_moonlight_dir().join(format!("{}.json", branch.to_string().to_lowercase()))
}

pub fn get_download_dir() -> PathBuf {
    get_moonlight_dir().join(DOWNLOAD_DIR)
}

pub fn kill_discord(branch: Branch) {
    let name = match branch {
        Branch::Stable => "Discord",
        Branch::PTB => "DiscordPTB",
        Branch::Canary => "DiscordCanary",
        Branch::Development => "DiscordDevelopment",
    };

    match std::env::consts::OS {
        "windows" => {
            std::process::Command::new("taskkill")
                .args(["/F", "/IM", &format!("{}.exe", name)])
                .output()
                .ok();
        }

        "macos" | "linux" => {
            std::process::Command::new("killall")
                .args([name])
                .output()
                .ok();
        }

        _ => {}
    }
}

pub fn branch_name(branch: MoonlightBranch) -> &'static str {
    match branch {
        MoonlightBranch::Stable => "Stable",
        MoonlightBranch::Nightly => "Nightly",
    }
}

pub fn branch_desc(branch: MoonlightBranch) -> &'static str {
    match branch {
        MoonlightBranch::Stable => {
            "Periodic updates and fixes when they're ready. Suggested for most users."
        }
        MoonlightBranch::Nightly => {
            "In-progress development snapshots while it's being worked on. May contain issues."
        }
    }
}
