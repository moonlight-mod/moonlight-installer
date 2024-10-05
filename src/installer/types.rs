use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonlightBranch {
    Stable,
    Nightly,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub enum InstallType {
    Windows,
    MacOS,
    Linux,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum Branch {
    Stable,
    PTB,
    Canary,
    Development,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DetectedInstall {
    pub install_type: InstallType,
    pub branch: Branch,
    pub path: PathBuf,
}

// Just DetectedInstall but tracking patched for the UI
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct InstallInfo {
    pub install: DetectedInstall,
    pub patched: bool,
}

// Lot more in here but idc
#[derive(serde::Deserialize, Debug)]
pub struct GitHubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct GitHubRelease {
    pub name: String,
    pub assets: Vec<GitHubReleaseAsset>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ErrorCode {
    Unknown,
    WindowsFileLock,
    MacOSNoPermission,
    NetworkFailed,
}

pub type InstallerResult<T> = std::result::Result<T, InstallerError>;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InstallerError {
    pub message: String,
    pub code: ErrorCode,
}

impl From<std::io::Error> for InstallerError {
    fn from(value: std::io::Error) -> Self {
        InstallerError {
            message: value.to_string(),
            code: match (value.raw_os_error(), std::env::consts::OS) {
                (Some(32), "windows") => ErrorCode::WindowsFileLock,
                (Some(1), "macos") => ErrorCode::MacOSNoPermission,
                _ => ErrorCode::Unknown,
            },
        }
    }
}

impl From<Box<dyn std::error::Error>> for InstallerError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        InstallerError {
            message: value.to_string(),
            code: ErrorCode::Unknown,
        }
    }
}

impl From<reqwest::Error> for InstallerError {
    fn from(value: reqwest::Error) -> Self {
        InstallerError {
            message: value.to_string(),
            code: ErrorCode::NetworkFailed,
        }
    }
}
