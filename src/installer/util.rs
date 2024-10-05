use super::types::{Branch, MoonlightBranch};
use std::path::PathBuf;

const DOWNLOAD_DIR: &str = "dist";

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
