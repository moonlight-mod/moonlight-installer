use libmoonlight::types::{Branch, DetectedInstall, InstallInfo, MoonlightBranch};
use libmoonlight::{get_download_dir, Installer};
use std::collections::HashMap;
use std::path::PathBuf;

pub type Version = String;

pub enum LogicCommand {
    GetInstalls,
    GetDownloadedVersions,
    GetLatestVersion(MoonlightBranch),
    UpdateMoonlight(MoonlightBranch),
    PatchInstall {
        branch: MoonlightBranch,
        install: DetectedInstall,
    },
    UnpatchInstall(DetectedInstall),
    KillDiscord(Branch),
    ResetConfig(Branch),
}

pub enum LogicResponse {
    Installs(Vec<InstallInfo>),
    DownloadedVersions(HashMap<MoonlightBranch, Version>),
    LatestVersion(libmoonlight::Result<Version>),
    UpdateComplete(libmoonlight::Result<(MoonlightBranch, Version)>),
    PatchComplete(libmoonlight::Result<PathBuf>),
    UnpatchComplete(libmoonlight::Result<PathBuf>),
}

pub fn app_logic_thread(
    rx: &flume::Receiver<LogicCommand>,
    tx: &flume::Sender<LogicResponse>,
) -> Result<(), Box<dyn std::error::Error>> {
    let installer = Installer::new();

    loop {
        match rx.recv()? {
            LogicCommand::GetLatestVersion(branch) => {
                let latest_version = installer.get_latest_moonlight_version(branch);
                tx.send(LogicResponse::LatestVersion(latest_version))?;
            }

            LogicCommand::GetDownloadedVersions => {
                // TODO: handle errors
                let downloaded_versions = installer.get_downloaded_versions().unwrap_or_default();
                tx.send(LogicResponse::DownloadedVersions(downloaded_versions))?;
            }

            LogicCommand::GetInstalls => {
                let installs = installer.get_installs().unwrap_or_default();
                tx.send(LogicResponse::Installs(installs))?;
            }

            LogicCommand::UpdateMoonlight(branch) => {
                let result = installer.download_moonlight(branch);
                if let Ok(ref version) = result {
                    installer.set_downloaded_version(branch, version).ok();
                }
                tx.send(LogicResponse::UpdateComplete(result.map(|v| (branch, v))))?;
            }

            LogicCommand::PatchInstall { branch, install } => {
                let resp = installer
                    .patch_install(&install, get_download_dir(branch), branch)
                    .map(|()| install.path);
                tx.send(LogicResponse::PatchComplete(resp))?;
            }

            LogicCommand::UnpatchInstall(install) => {
                let resp = installer.unpatch_install(&install).map(|()| install.path);
                tx.send(LogicResponse::UnpatchComplete(resp))?;
            }

            LogicCommand::KillDiscord(branch) => {
                let _ = branch.kill_discord();
            }

            LogicCommand::ResetConfig(branch) => {
                installer.reset_config(branch);
            }
        }
    }
}
