#[cfg(unix)]
use nix::unistd::{Uid, User};

use crate::types::{
    Branch, DetectedInstall, FlatpakFilesystemOverride, FlatpakFilesystemOverridePermission,
    FlatpakOverrides, InstallInfo,
};
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
                "linux" => get_dot_config().join("moonlight-mod"),
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

// https://github.com/flatpak/flatpak/pull/6084
pub fn get_local_share_workaround() -> PathBuf {
    get_home_dir().join(".local").join("share")
}

fn get_flatpak_home() -> PathBuf {
    let a = get_local_share().join("flatpak");
    if a.exists() {
        a
    } else {
        let b = get_local_share_workaround().join("flatpak");
        if b.exists() {
            b
        } else {
            a
        }
    }
}

pub fn get_dot_config() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| get_home_dir().join(".config"))
}

fn get_flatpak_overrides(id: &str) -> crate::Result<Option<FlatpakOverrides>> {
    let overrides = get_flatpak_home().join("overrides");

    std::fs::create_dir_all(&overrides)?;

    let app_overrides = overrides.join(id);

    let file = match std::fs::OpenOptions::new().read(true).open(&app_overrides) {
        Ok(v) => v,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => return Ok(None),
            _ => return Err(err.into()),
        },
    };

    serde_ini::from_read(file).or(Ok(None))
}

pub fn ensure_flatpak_overrides(id: &str) -> crate::Result<()> {
    let overrides = get_flatpak_overrides(id)?;

    let has = overrides
        .as_ref()
        .and_then(|v| v.context.as_ref())
        .and_then(|v| v.filesystems.as_ref())
        .is_some_and(|v| {
            v.iter().any(|entry| {
                entry.path == "xdg-config/moonlight-mod"
                    && entry.permission == FlatpakFilesystemOverridePermission::ReadWrite
            })
        });

    if has {
        return Ok(());
    }

    let mut overrides = overrides.unwrap_or_default();

    if overrides.context.is_none() {
        overrides.context = Some(Default::default());
    }
    let context = overrides.context.as_mut().unwrap();

    if context.filesystems.is_none() {
        context.filesystems = Some(Default::default());
    }
    let filesystem = context.filesystems.as_mut().unwrap();

    filesystem.push(FlatpakFilesystemOverride {
        path: String::from("xdg-config/moonlight-mod"),
        permission: FlatpakFilesystemOverridePermission::ReadWrite,
    });

    // ensured that it exists in get_flatpak_overrides
    let app_overrides = get_flatpak_home().join("overrides").join(id);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .append(false)
        .open(&app_overrides)?;

    serde_ini::to_writer(&mut file, &overrides).expect("ini serialization to succeed");

    Ok(())
}
