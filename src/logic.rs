use crate::installer::{installer::Installer, types::*, util::kill_discord};
use std::path::PathBuf;

pub enum LogicCommand {
    GetInstalls,
    GetLatestVersion(MoonlightBranch),
    UpdateMoonlight(MoonlightBranch),
    PatchInstall(DetectedInstall),
    UnpatchInstall(DetectedInstall),
    KillDiscord(Branch),
    ResetConfig(Branch),
}

pub enum LogicResponse {
    Installs(Vec<InstallInfo>),
    LatestVersion(InstallerResult<String>),
    UpdateComplete(InstallerResult<String>),
    PatchComplete(InstallerResult<PathBuf>),
    UnpatchComplete(InstallerResult<PathBuf>),
}

pub fn app_logic_thread(
    rx: flume::Receiver<LogicCommand>,
    tx: flume::Sender<LogicResponse>,
) -> anyhow::Result<()> {
    let installer = Installer::new();

    loop {
        match rx.recv()? {
            LogicCommand::GetLatestVersion(branch) => {
                let latest_version = installer.get_latest_moonlight_version(branch);
                tx.send(LogicResponse::LatestVersion(latest_version))?;
            }

            LogicCommand::GetInstalls => {
                let installs = installer.get_installs().unwrap_or_default();
                tx.send(LogicResponse::Installs(installs))?;
            }

            LogicCommand::UpdateMoonlight(branch) => {
                let err = installer.download_moonlight(branch);
                tx.send(LogicResponse::UpdateComplete(err))?;
            }

            LogicCommand::PatchInstall(install) => {
                let resp = installer
                    .patch_install(install.clone())
                    .map(|_| install.path);
                tx.send(LogicResponse::PatchComplete(resp))?;
            }

            LogicCommand::UnpatchInstall(install) => {
                let resp = installer
                    .unpatch_install(install.clone())
                    .map(|_| install.path);
                tx.send(LogicResponse::UnpatchComplete(resp))?;
            }

            LogicCommand::KillDiscord(branch) => {
                kill_discord(branch);
            }

            LogicCommand::ResetConfig(branch) => {
                installer.reset_config(branch);
            }
        }
    }
}
