#[derive(thiserror::Error, serde::Serialize, serde::Deserialize, Debug)]
pub enum MoonlightError {
    #[error("failed to get windows file lock: {0}")]
    WindowsFileLock(String),
    #[error("failed to get macos file permission: {0}")]
    MacOSNoPermission(String),
    #[error("network request failed: {0}")]
    NetworkFailed(String),
    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, MoonlightError>;

impl From<std::io::Error> for MoonlightError {
    fn from(value: std::io::Error) -> Self {
        match (value.raw_os_error(), std::env::consts::OS) {
            (Some(32), "windows") => Self::WindowsFileLock(value.to_string()),
            (Some(1), "macos") => Self::MacOSNoPermission(value.to_string()),
            _ => Self::Unknown(value.to_string()),
        }
    }
}

impl From<Box<dyn std::error::Error>> for MoonlightError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self::Unknown(value.to_string())
    }
}

impl From<reqwest::Error> for MoonlightError {
    fn from(value: reqwest::Error) -> Self {
        Self::NetworkFailed(value.to_string())
    }
}
