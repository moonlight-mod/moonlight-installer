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

#[derive(serde::Serialize, serde::Deserialize)]
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
