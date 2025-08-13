use super::types::{Branch, DetectedInstall, GitHubRelease, InstallInfo, MoonlightBranch};
use super::util::{get_download_dir, get_home_dir};
use crate::types::{
    DownloadedBranchInfo, DownloadedMap, MoonlightMeta, TemplatedPathBuf, TemplatedPathBufBase,
};
use crate::{
    ensure_flatpak_overrides, get_app_dir, get_local_share, get_local_share_workaround,
    get_moonlight_dir, PATCHED_ASAR,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const USER_AGENT: &str =
    "moonlight-installer (https://github.com/moonlight-mod/moonlight-installer)";
const LEGACY_INSTALLED_VERSION_FILE: &str = ".moonlight-installed-version";
const INSTALLED_VERSIONS_FILE: &str = "moonlight-installed-versions.json";

const GITHUB_REPO: &str = "moonlight-mod/moonlight";
const ARTIFACT_NAME: &str = "dist.tar.gz";
const NIGHTLY_REF_URL: &str = "https://moonlight-mod.github.io/moonlight/ref";
const NIGHTLY_DIST_URL: &str = "https://moonlight-mod.github.io/moonlight/dist.tar.gz";

pub struct Installer;

impl Default for Installer {
    fn default() -> Self {
        Self::new()
    }
}

impl Installer {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    pub fn download_moonlight(
        &self,
        branch: MoonlightBranch,
    ) -> crate::Result<DownloadedBranchInfo> {
        let dir = get_download_dir(branch);

        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }

        std::fs::create_dir_all(&dir)?;

        let version = match branch {
            MoonlightBranch::Stable => self.download_stable(&dir)?,
            MoonlightBranch::Nightly => self.download_nightly(&dir)?,
        };

        let path = pathdiff::diff_paths(&dir, get_moonlight_dir()).unwrap_or(dir);

        Ok(DownloadedBranchInfo { version, path })
    }

    fn download_stable(&self, dir: impl AsRef<Path>) -> crate::Result<String> {
        let release = self.get_stable_release()?;
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == ARTIFACT_NAME)
            .unwrap();

        let resp = reqwest::blocking::Client::new()
            .get(&asset.browser_download_url)
            .header("User-Agent", USER_AGENT)
            .send()?;
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(resp));

        archive.unpack(dir)?;
        Ok(release.name)
    }

    fn download_nightly(&self, dir: impl AsRef<Path>) -> crate::Result<String> {
        let version = self.get_nightly_version()?;
        let resp = reqwest::blocking::get(NIGHTLY_DIST_URL)?;
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(resp));
        archive.unpack(dir)?;
        Ok(version)
    }

    pub fn get_latest_moonlight_version(&self, branch: MoonlightBranch) -> crate::Result<String> {
        match branch {
            MoonlightBranch::Stable => self.get_stable_release().map(|x| x.name),
            MoonlightBranch::Nightly => self.get_nightly_version(),
        }
    }

    pub fn get_downloaded_versions(&self) -> crate::Result<DownloadedMap> {
        let dir = get_moonlight_dir();
        let legacy_file = dir.join(LEGACY_INSTALLED_VERSION_FILE);
        let file = dir.join(INSTALLED_VERSIONS_FILE);

        if std::fs::exists(&legacy_file)? {
            // special case: migration from <=v0.2.5
            let serialized_version = std::fs::read_to_string(&legacy_file)?;

            let installed_version = if serialized_version.is_empty() {
                // No versions installed…I guess?
                None
            } else if serialized_version.starts_with('v') {
                Some((MoonlightBranch::Stable, serialized_version))
            } else {
                Some((MoonlightBranch::Nightly, serialized_version))
            };

            let versions = if get_moonlight_dir().join("dist").exists() {
                installed_version.map(|(branch, version)| {
                    HashMap::from([(
                        branch,
                        DownloadedBranchInfo {
                            version,
                            path: PathBuf::from("dist"),
                        },
                    )])
                })
            } else {
                None
            }
            .unwrap_or_default();

            let serialized = serde_json::to_string_pretty(&versions)?;

            std::fs::write(&file, serialized)?;
            std::fs::remove_file(&legacy_file)?;

            return Ok(versions);
        }

        let serialized_versions = match std::fs::read_to_string(&file) {
            Ok(v) => v,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    // No versions installed
                    return Ok(Default::default());
                }
                _ => return Err(err.into()),
            },
        };

        let mut versions: DownloadedMap = serde_json::from_str(&serialized_versions)?;

        // filter out missing versions
        for key in MoonlightBranch::ALL {
            let Some(info) = versions.get(&key) else {
                continue;
            };

            let path = if info.path.is_relative() {
                &get_moonlight_dir().join(&info.path)
            } else {
                &info.path
            };

            if !path.exists() {
                versions.remove(&key);
            }
        }

        Ok(versions)
    }

    pub fn set_downloaded_version(
        &self,
        branch: MoonlightBranch,
        version: String,
        path: PathBuf,
    ) -> crate::Result<()> {
        let dir = get_moonlight_dir();

        let mut current = self.get_downloaded_versions()?;
        let info = DownloadedBranchInfo { version, path };
        current.insert(branch, info);

        let serialized = serde_json::to_string_pretty(&current)?;

        std::fs::write(dir.join(INSTALLED_VERSIONS_FILE), serialized)?;

        Ok(())
    }

    fn get_stable_release(&self) -> crate::Result<GitHubRelease> {
        let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
        let resp = reqwest::blocking::Client::new()
            .get(url)
            .header("User-Agent", USER_AGENT)
            .send()?
            .json()?;
        Ok(resp)
    }

    fn get_nightly_version(&self) -> crate::Result<String> {
        let resp = reqwest::blocking::get(NIGHTLY_REF_URL)?.text()?;
        Ok(resp
            .lines()
            .next()
            .map(ToString::to_string)
            .unwrap_or_default())
    }

    pub fn get_installs(&self) -> crate::Result<Vec<InstallInfo>> {
        self.detect_installs().map(|installs| {
            installs
                .into_iter()
                .map(|install| {
                    let patched = self.is_install_patched(&install).unwrap_or(false);
                    let has_config = install.branch.config().exists();

                    InstallInfo {
                        install,
                        patched,
                        has_config,
                    }
                })
                .collect()
        })
    }

    fn detect_installs(&self) -> crate::Result<Vec<DetectedInstall>> {
        match std::env::consts::OS {
            "windows" => {
                let appdata = std::env::var("LocalAppData").unwrap();
                let dirs = [
                    ("Discord", Branch::Stable),
                    ("DiscordPTB", Branch::PTB),
                    ("DiscordCanary", Branch::Canary),
                    ("DiscordDevelopment", Branch::Development),
                ];
                let mut installs = vec![];

                for (dir, branch) in dirs {
                    let path = PathBuf::from(appdata.clone()).join(dir);
                    if path.exists() {
                        // app-(version)
                        let mut app_dirs: Vec<_> = std::fs::read_dir(&path)?
                            .filter_map(Result::ok)
                            .filter(|x| x.file_name().to_string_lossy().starts_with("app-"))
                            .collect();

                        app_dirs.sort_by(|a, b| {
                            let a_file_name = a.file_name();
                            let b_file_name = b.file_name();
                            a_file_name.cmp(&b_file_name)
                        });

                        if let Some(most_recent_install) = app_dirs.last() {
                            let path = most_recent_install.path();

                            let res_dir = get_app_dir(&path)?;
                            let app_dir = res_dir.join("app");

                            let moonlight_info =
                                std::fs::read_to_string(app_dir.join("moonlight.json"))
                                    .ok()
                                    .and_then(|s| serde_json::from_str(&s).ok());

                            installs.push(DetectedInstall {
                                branch,
                                path,
                                moonlight_info,
                                flatpak_id: None,
                            });
                        }
                    }
                }

                Ok(installs)
            }

            "macos" => {
                let apps_dirs = vec![
                    PathBuf::from("/Applications"),
                    get_home_dir().join("Applications"),
                ];

                let branches = [
                    ("Discord", Branch::Stable),
                    ("Discord PTB", Branch::PTB),
                    ("Discord Canary", Branch::Canary),
                    ("Discord Development", Branch::Development),
                ];

                let mut installs = vec![];

                for apps_dir in apps_dirs {
                    for (branch_name, branch) in branches {
                        let macos_app_dir = apps_dir.join(format!("{branch_name}.app"));

                        if !macos_app_dir.exists() {
                            continue;
                        }

                        let path = macos_app_dir.join("Contents/Resources");

                        let res_dir = get_app_dir(&path)?;
                        let app_dir = res_dir.join("app");

                        let moonlight_info =
                            std::fs::read_to_string(app_dir.join("moonlight.json"))
                                .ok()
                                .and_then(|s| serde_json::from_str(&s).ok());

                        installs.push(DetectedInstall {
                            branch,
                            path,
                            moonlight_info,
                            flatpak_id: None,
                        });
                    }
                }

                Ok(installs)
            }

            "linux" => {
                // this is a crime but it has to be done...
                // please merge pr flatpak devs
                let local_shares = [get_local_share(), get_local_share_workaround()];

                let dirs = [
                    ("Discord", Branch::Stable, None),
                    ("DiscordPTB", Branch::PTB, None),
                    ("DiscordCanary", Branch::Canary, None),
                    ("DiscordDevelopment", Branch::Development, None),
                    // flatpak user installations
                    ("flatpak/app/com.discordapp.Discord/current/active/files/discord", Branch::Stable, Some("com.discordapp.Discord")),
                    ("flatpak/app/com.discordapp.DiscordCanary/current/active/files/discord-canary", Branch::Canary, Some("com.discordapp.DiscordCanary")),
                ];

                let mut installs = vec![];
                for (dir, branch, id) in dirs {
                    for local_share in &local_shares {
                        let path = local_share.join(dir);
                        if path.join(branch.name()).exists() {
                            let res_dir = get_app_dir(&path)?;
                            let app_dir = res_dir.join("app");

                            let moonlight_info =
                                std::fs::read_to_string(app_dir.join("moonlight.json"))
                                    .ok()
                                    .and_then(|s| serde_json::from_str(&s).ok());

                            installs.push(DetectedInstall {
                                branch,
                                path,
                                moonlight_info,
                                flatpak_id: id.map(Into::into),
                            });
                            break;
                        }
                    }
                }

                Ok(installs)
            }

            _ => Ok(Vec::new()),
        }
    }

    // This will probably match other client mods that replace app.asar, but it
    // will just prompt them to unpatch, so I think it's fine
    fn is_install_patched(&self, install: &DetectedInstall) -> crate::Result<bool> {
        Ok(!get_app_dir(&install.path)?.join("app.asar").exists())
    }

    pub fn patch_install(
        &self,
        install: &DetectedInstall,
        download_dir: PathBuf,
        branch: MoonlightBranch,
    ) -> crate::Result<()> {
        // TODO: atomic patching

        let res_dir = get_app_dir(&install.path)?;
        let app_dir = res_dir.join("app");
        let asar = res_dir.join("app.asar");
        std::fs::rename(&asar, asar.with_file_name(PATCHED_ASAR))?;
        std::fs::create_dir(&app_dir)?;

        let json = serde_json::json!({
          "name": install.branch.dashed_name(),
          "main": "./injector.js",
          "private": true
        });
        std::fs::write(app_dir.join("package.json"), json.to_string())?;

        let injector_path = download_dir.join("injector.js");
        let moonlight_injector = pathdiff::diff_paths(&injector_path, get_moonlight_dir())
            .map(|path_str| TemplatedPathBuf {
                relative_to: Some(TemplatedPathBufBase::Moonlight),
                path_str,
            })
            .unwrap_or(TemplatedPathBuf {
                relative_to: None,
                path_str: injector_path,
            });

        let moonlight_info = MoonlightMeta {
            moonlight_injector,
            patched_asar: PATCHED_ASAR.to_owned(),
            branch,
        };
        std::fs::write(
            app_dir.join("moonlight.json"),
            serde_json::to_string_pretty(&moonlight_info)
                .expect("MoonlightMeta's Serialize implementation should not fail"),
        )?;
        std::fs::write(app_dir.join("injector.js"), include_str!("injector.js"))?;

        if let Some(flatpak_id) = install.flatpak_id.as_deref() {
            ensure_flatpak_overrides(flatpak_id)?;
        }

        Ok(())
    }

    pub fn unpatch_install(&self, install: &DetectedInstall) -> crate::Result<()> {
        let app_dir = get_app_dir(&install.path)?;
        let asar = app_dir.join(PATCHED_ASAR);
        std::fs::rename(&asar, asar.with_file_name("app.asar"))?;
        std::fs::remove_dir_all(app_dir.join("app"))?;
        Ok(())
    }

    pub fn reset_config(&self, branch: Branch) {
        let config = branch.config();
        let new_name = format!(
            "{}-backup-{}.json",
            config.file_stem().unwrap().to_string_lossy(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        std::fs::rename(&config, config.with_file_name(new_name)).ok();
    }
}
