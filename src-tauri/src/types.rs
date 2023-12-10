use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum MoonlightBranch {
    Stable,
    Nightly,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum InstallType {
    Windows,
    MacOS,
    Linux,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum Branch {
    Stable,
    PTB,
    Canary,
    Development,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DetectedInstall {
    pub install_type: InstallType,
    pub branch: Branch,
    pub path: PathBuf,
}

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
    MacOSNoPermission
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Error {
    pub message: String,
    pub code: ErrorCode,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error {
            message: value.to_string(),
            code: match (value.raw_os_error(), std::env::consts::OS) {
                (Some(32), "windows") => ErrorCode::WindowsFileLock,
                (Some(1), "macos") => ErrorCode::MacOSNoPermission,
                _ => ErrorCode::Unknown,
            },
        }
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Error {
            message: value.to_string(),
            code: ErrorCode::Unknown,
        }
    }
}
