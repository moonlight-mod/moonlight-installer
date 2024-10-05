use super::{
    types::*,
    util::{get_branch_config, get_download_dir},
};
use std::path::PathBuf;

const USER_AGENT: &str =
    "moonlight-installer (https://github.com/moonlight-mod/moonlight-installer)";

const GITHUB_REPO: &str = "moonlight-mod/moonlight";
const ARTIFACT_NAME: &str = "dist.tar.gz";
const NIGHTLY_REF_URL: &str = "https://moonlight-mod.github.io/moonlight/ref";
const NIGHTLY_DIST_URL: &str = "https://moonlight-mod.github.io/moonlight/dist.tar.gz";

const PATCHED_ASAR: &str = "_app.asar";

pub struct Installer;

impl Installer {
    pub fn new() -> Self {
        Installer {}
    }

    pub fn download_moonlight(&self, branch: MoonlightBranch) -> InstallerResult<String> {
        let dir = get_download_dir();

        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }

        std::fs::create_dir_all(&dir)?;

        Ok(match branch {
            MoonlightBranch::Stable => self.download_stable(dir)?,
            MoonlightBranch::Nightly => self.download_nightly(dir)?,
        })
    }

    fn download_stable(&self, dir: PathBuf) -> InstallerResult<String> {
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

    fn download_nightly(&self, dir: PathBuf) -> InstallerResult<String> {
        let version = self.get_nightly_version()?;
        let resp = reqwest::blocking::get(NIGHTLY_DIST_URL)?;
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(resp));
        archive.unpack(dir)?;
        Ok(version)
    }

    pub fn get_latest_moonlight_version(&self, branch: MoonlightBranch) -> InstallerResult<String> {
        match branch {
            MoonlightBranch::Stable => self.get_stable_release().map(|x| x.name),
            MoonlightBranch::Nightly => self.get_nightly_version(),
        }
    }

    fn get_stable_release(&self) -> InstallerResult<GitHubRelease> {
        let url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            GITHUB_REPO
        );
        let resp = reqwest::blocking::Client::new()
            .get(url)
            .header("User-Agent", USER_AGENT)
            .send()?
            .json()?;
        Ok(resp)
    }

    fn get_nightly_version(&self) -> InstallerResult<String> {
        let resp = reqwest::blocking::get(NIGHTLY_REF_URL)?.text()?;
        Ok(resp
            .lines()
            .next()
            .map(|x| x.to_string())
            .unwrap_or_default())
    }

    pub fn get_installs(&self) -> InstallerResult<Vec<InstallInfo>> {
        self.detect_installs().map(|installs| {
            installs
                .into_iter()
                .map(|install| {
                    let patched = self.is_install_patched(install.clone()).unwrap_or(false);
                    let has_config = get_branch_config(install.branch).exists();

                    InstallInfo {
                        install,
                        patched,
                        has_config,
                    }
                })
                .collect()
        })
    }

    fn detect_installs(&self) -> InstallerResult<Vec<DetectedInstall>> {
        match std::env::consts::OS {
            "windows" => {
                let appdata = std::env::var("LocalAppData").unwrap();
                let dirs = vec![
                    "Discord",
                    "DiscordPTB",
                    "DiscordCanary",
                    "DiscordDevelopment",
                ];
                let mut installs = vec![];

                for dir in dirs {
                    let branch = match dir {
                        "Discord" => Branch::Stable,
                        "DiscordPTB" => Branch::PTB,
                        "DiscordCanary" => Branch::Canary,
                        "DiscordDevelopment" => Branch::Development,
                        _ => unreachable!(),
                    };

                    let path = PathBuf::from(appdata.clone()).join(dir);
                    if path.exists() {
                        // app-(version)
                        let mut app_dirs: Vec<_> = std::fs::read_dir(&path)?
                            .filter_map(|x| x.ok())
                            .filter(|x| x.file_name().to_string_lossy().starts_with("app-"))
                            .collect();

                        app_dirs.sort_by(|a, b| {
                            let a_file_name = a.file_name();
                            let b_file_name = b.file_name();
                            a_file_name.cmp(&b_file_name)
                        });

                        if let Some(most_recent_install) = app_dirs.last() {
                            installs.push(DetectedInstall {
                                install_type: InstallType::Windows,
                                branch,
                                path: most_recent_install.path(),
                            });
                        }
                    }
                }

                Ok(installs)
            }

            "macos" => {
                let apps_dirs = vec![
                    PathBuf::from("/Applications"),
                    PathBuf::from(std::env::var("HOME").unwrap()).join("Applications"),
                ];

                let branch_names = vec![
                    "Discord",
                    "Discord PTB",
                    "Discord Canary",
                    "Discord Development",
                ];

                let mut installs = vec![];

                for apps_dir in apps_dirs {
                    for branch_name in branch_names.clone() {
                        let branch = match branch_name {
                            "Discord" => Branch::Stable,
                            "Discord PTB" => Branch::PTB,
                            "Discord Canary" => Branch::Canary,
                            "Discord Development" => Branch::Development,
                            _ => unreachable!(),
                        };

                        let macos_app_dir = apps_dir.join(format!("{}.app", branch_name));

                        if !macos_app_dir.exists() {
                            continue;
                        }

                        let app_dir = macos_app_dir.join("Contents/Resources");

                        installs.push(DetectedInstall {
                            install_type: InstallType::MacOS,
                            branch,
                            path: app_dir,
                        })
                    }
                }

                Ok(installs)
            }

            "linux" => {
                let home = std::env::var("HOME").unwrap();
                let local_share = PathBuf::from(home).join(".local/share");

                let dirs = vec![
                    "Discord",
                    "DiscordPTB",
                    "DiscordCanary",
                    "DiscordDevelopment",
                ];

                let mut installs = vec![];
                for dir in dirs {
                    let branch = match dir {
                        "Discord" => Branch::Stable,
                        "DiscordPTB" => Branch::PTB,
                        "DiscordCanary" => Branch::Canary,
                        "DiscordDevelopment" => Branch::Development,
                        _ => unreachable!(),
                    };

                    let path = local_share.join(dir);
                    if path.exists() {
                        installs.push(DetectedInstall {
                            install_type: InstallType::Linux,
                            branch,
                            path,
                        });
                    }
                }

                Ok(installs)
            }

            _ => Ok(Vec::new()),
        }
    }

    fn get_app_dir(&self, install: DetectedInstall) -> InstallerResult<PathBuf> {
        match std::env::consts::OS {
            "windows" | "linux" => Ok(install.path.join("resources")),
            "macos" => Ok(install.path),
            _ => unimplemented!("Unsupported OS"),
        }
    }

    // This will probably match other client mods that replace app.asar, but it
    // will just prompt them to unpatch, so I think it's fine
    fn is_install_patched(&self, install: DetectedInstall) -> InstallerResult<bool> {
        Ok(!self.get_app_dir(install)?.join("app.asar").exists())
    }

    pub fn patch_install(&self, install: DetectedInstall) -> InstallerResult<()> {
        // TODO: flatpak and stuff
        let app_dir = self.get_app_dir(install)?;
        let asar = app_dir.join("app.asar");
        std::fs::rename(&asar, asar.with_file_name(PATCHED_ASAR))?;
        std::fs::create_dir(app_dir.join("app"))?;

        let json = serde_json::json!({
          "name": "discord",
          "main": "./injector.js",
          "private": true
        });
        std::fs::write(app_dir.join("app/package.json"), json.to_string())?;

        let moonlight_injector = get_download_dir().join("injector.js");
        let moonlight_injector_str = serde_json::to_string(&moonlight_injector).unwrap();
        let injector = format!(
            r#"require({}).inject(
  require("path").resolve(__dirname, "../{}")
);
"#,
            moonlight_injector_str, PATCHED_ASAR
        );
        std::fs::write(app_dir.join("app/injector.js"), injector)?;
        Ok(())
    }

    pub fn unpatch_install(&self, install: DetectedInstall) -> InstallerResult<()> {
        let app_dir = self.get_app_dir(install)?;
        let asar = app_dir.join(PATCHED_ASAR);
        std::fs::rename(&asar, asar.with_file_name("app.asar"))?;
        std::fs::remove_dir_all(app_dir.join("app"))?;
        Ok(())
    }

    pub fn reset_config(&self, branch: Branch) {
        let config = get_branch_config(branch);
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
