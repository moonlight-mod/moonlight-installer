use libmoonlight::types::{Branch, DetectedInstall, InstallInfo, InstallerResult, MoonlightBranch};
use libmoonlight::Installer;
use std::path::PathBuf;

pub enum LogicCommand {
	GetInstalls,
	GetDownloadedVersion,
	GetLatestVersion(MoonlightBranch),
	UpdateMoonlight(MoonlightBranch),
	PatchInstall(DetectedInstall),
	UnpatchInstall(DetectedInstall),
	KillDiscord(Branch),
	ResetConfig(Branch),
}

pub enum LogicResponse {
	Installs(Vec<InstallInfo>),
	DownloadedVersion(Option<String>),
	LatestVersion(InstallerResult<String>),
	UpdateComplete(InstallerResult<String>),
	PatchComplete(InstallerResult<PathBuf>),
	UnpatchComplete(InstallerResult<PathBuf>),
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

			LogicCommand::GetDownloadedVersion => {
				let downloaded_version = installer.get_downloaded_version().unwrap_or(None);
				tx.send(LogicResponse::DownloadedVersion(downloaded_version))?;
			}

			LogicCommand::GetInstalls => {
				let installs = installer.get_installs().unwrap_or_default();
				tx.send(LogicResponse::Installs(installs))?;
			}

			LogicCommand::UpdateMoonlight(branch) => {
				let err = installer.download_moonlight(branch);
				if let Ok(ref version) = err {
					installer.set_downloaded_version(version).ok();
				}
				tx.send(LogicResponse::UpdateComplete(err))?;
			}

			LogicCommand::PatchInstall(install) => {
				let resp = installer
					.patch_install(&install, None)
					.map(|()| install.path);
				tx.send(LogicResponse::PatchComplete(resp))?;
			}

			LogicCommand::UnpatchInstall(install) => {
				let resp = installer
					.unpatch_install(&install)
					.map(|()| install.path);
				tx.send(LogicResponse::UnpatchComplete(resp))?;
			}

			LogicCommand::KillDiscord(branch) => {
				branch.kill_discord();
			}

			LogicCommand::ResetConfig(branch) => {
				installer.reset_config(branch);
			}
		}
	}
}
