use crate::config::Config;
use crate::logic::{app_logic_thread, LogicCommand, LogicResponse, Version};
use libmoonlight::types::{
    Branch, DownloadedBranchInfo, DownloadedMap, InstallInfo, MoonlightBranch,
};
use libmoonlight::MoonlightError;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct AppState {
    downloaded_versions: Option<DownloadedMap>,
    latest_version: HashMap<MoonlightBranch, Version>,
    installs: Option<Vec<InstallInfo>>,

    downloading: bool,
    downloading_error: Option<MoonlightError>,

    patching: bool,
    patching_error: Option<MoonlightError>,
    patching_branch: Option<Branch>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct App {
    config: Config,

    // Don't know how to clean up these skips lol
    #[serde(skip)]
    tx: Option<flume::Sender<LogicCommand>>,
    #[serde(skip)]
    rx: Option<flume::Receiver<LogicResponse>>,
    #[serde(skip)]
    state: AppState,
}

// https://github.com/rust-lang/rustfmt/issues/3863
const PATCH_TOOLIP: &str = "Download moonlight first to patch a Discord installation.";
const RESET_CONFIG_TOOLTIP: &str =
    "Backs up and removes the moonlight config file for this Discord installation.";
const WINDOWS_FILE_LOCK: &str = "Discord is currently open, which locks moonlight's ability to modify its files. Please completely close Discord and make sure it does not appear in the taskbar.\nAlternatively, click the button below to attempt to close Discord forcefully. This will disconnect you from any voice calls you are in and may cause issues.";
const MACOS_NO_PERMISSION: &str = "moonlight is unable to modify your Discord installation. This is because your MacOS system privacy settings doesn't allow us to do so.\nYou can fix this via a pop-up you should've gotten, or by going to System Settings > Privacy & Security > App Management and allowing moonlight installer.";
const NETWORK_FAILED: &str = "moonlight is unable to download required files, likely due to a network issue. Please check your internet connection and try again.";

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        let (main_tx, logic_rx) = flume::unbounded::<LogicCommand>();
        let (logic_tx, main_rx) = flume::unbounded::<LogicResponse>();
        std::thread::spawn(move || {
            if let Err(err) = app_logic_thread(&logic_rx, &logic_tx) {
                log::error!("Logic thread error: {:?}", err);
            }
        });

        app.tx = Some(main_tx);
        app.rx = Some(main_rx);

        app.send(LogicCommand::GetDownloadedVersions);

        let mut seen = HashSet::new();
        for branch in app
            .config
            .install_selected_branches
            .values()
            .copied()
            .chain([app.config.selected_branch].into_iter())
        {
            if !seen.contains(&branch) {
                app.send(LogicCommand::GetLatestVersion(branch));
                seen.insert(branch);
            }
        }
        drop(seen);

        app.send(LogicCommand::GetInstalls);

        app
    }

    fn handle_messages(&mut self) {
        // This is always filled, we just need to mark it as an Option for serde
        let rx = self.rx.as_ref().unwrap();
        while let Ok(msg) = rx.try_recv() {
            match msg {
                LogicResponse::Installs(installs) => {
                    log::info!("Installs: {:?}", installs);
                    self.state.installs = Some(installs);
                }

                LogicResponse::DownloadedVersions(versions) => {
                    log::info!("Downloaded versions: {:#?}", versions);
                    self.state.downloaded_versions = Some(versions);
                }

                LogicResponse::LatestVersion(version) => {
                    log::info!("Latest version: {:?}", version);
                    if let Ok((branch, version)) = version {
                        self.state.latest_version.insert(branch, version);
                        self.state.downloading_error = None;
                    } else {
                        self.state.downloading_error = version.err();
                    }
                }

                LogicResponse::UpdateComplete(info) => {
                    log::info!("Update complete: {:?}", info);
                    if let Ok((branch, info)) = info {
                        if let Some(map) = self.state.downloaded_versions.as_mut() {
                            map.insert(branch, info);
                        }
                        self.state.downloading_error = None;
                    } else {
                        self.state.downloading_error = info.err();
                    }
                    self.state.downloading = false;
                }

                LogicResponse::PatchComplete(install_path) => {
                    log::info!("Patch complete: {:?}", install_path);
                    if let Ok(install_path) = install_path {
                        if let Some(installs) = &mut self.state.installs {
                            for i in installs.iter_mut() {
                                if i.install.path == install_path {
                                    i.patched = true;
                                    break;
                                }
                            }
                        }
                        self.state.patching_error = None;
                    } else {
                        self.state.patching_error = install_path.err();
                    }

                    self.state.patching = false;
                }

                LogicResponse::UnpatchComplete(install_path) => {
                    log::info!("Unpatch complete: {:?}", install_path);
                    if let Ok(install_path) = install_path {
                        if let Some(installs) = &mut self.state.installs {
                            for i in installs.iter_mut() {
                                if i.install.path == install_path {
                                    i.patched = false;
                                    break;
                                }
                            }
                        }
                        self.state.patching_error = None;
                    } else {
                        self.state.patching_error = install_path.err();
                    }

                    self.state.patching = false;
                }
            }
        }
    }

    fn send(&self, cmd: LogicCommand) {
        // Same with above, always exists by this point
        let tx = self.tx.as_ref().unwrap();
        tx.send(cmd).unwrap();
    }

    fn draw_error(&self, ui: &mut egui::Ui, err: &MoonlightError) {
        ui.heading(egui::RichText::new("Error").color(egui::Color32::RED));

        match err {
            MoonlightError::WindowsFileLock(_) => {
                ui.label(WINDOWS_FILE_LOCK);

                if ui.button("Force close Discord").clicked() {
                    if let Some(branch) = self.state.patching_branch {
                        self.send(LogicCommand::KillDiscord(branch));
                    }
                }
            }

            MoonlightError::MacOSNoPermission(_) => {
                ui.label(MACOS_NO_PERMISSION);
            }

            MoonlightError::NetworkFailed(_) => {
                ui.label(NETWORK_FAILED);
            }

            MoonlightError::Unknown(msg) => {
                ui.label("An unknown error occurred. Please report this.");
                ui.label(msg);
            }
        }

        ui.separator();
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.heading("moonlight installer");

                    egui::CollapsingHeader::new("Download moonlight")
                        .default_open(true)
                        .show(ui, |ui| {
                            if let Some(err) = &self.state.downloading_error {
                                self.draw_error(ui, err);
                            }

                            ui.vertical(|ui| {
                                egui::ComboBox::from_label("Selected branch")
                                    .selected_text(self.config.selected_branch.name())
                                    .show_ui(ui, |ui| {
                                        for branch in MoonlightBranch::ALL {
                                            let str = format!(
                                                "{}\n  {}",
                                                branch.name(),
                                                branch.description()
                                            );
                                            if ui
                                                .selectable_value(
                                                    &mut self.config.selected_branch,
                                                    branch,
                                                    str,
                                                )
                                                .changed()
                                            {
                                                let branch = self.config.selected_branch;
                                                self.state.latest_version.remove(&branch);
                                                self.send(LogicCommand::GetLatestVersion(branch));
                                            }
                                        }
                                    });

                                ui.horizontal(|ui| {
                                    ui.label("Latest version:");
                                    if let Some(version) =
                                        self.state.latest_version.get(&self.config.selected_branch)
                                    {
                                        ui.label(version);
                                    } else {
                                        ui.spinner();
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Downloaded version:");
                                    if let Some(map) = &self.state.downloaded_versions {
                                        ui.label(
                                            map.get(&self.config.selected_branch)
                                                .map_or("None", |v| &v.version),
                                        );
                                    } else {
                                        ui.spinner();
                                    }
                                });

                                ui.horizontal(|ui| {
                                    let latest_version =
                                        self.state.latest_version.get(&self.config.selected_branch);
                                    let can_download = !self.state.downloading
                                        && latest_version.is_some()
                                        && self
                                            .state
                                            .downloaded_versions
                                            .as_ref()
                                            .and_then(|map| map.get(&self.config.selected_branch))
                                            .is_none_or(
                                                |DownloadedBranchInfo { version, .. }| {
                                                    latest_version
                                                        .is_some_and(|latest| version != latest)
                                                },
                                            );

                                    if ui
                                        .add_enabled(can_download, egui::Button::new("Download"))
                                        .clicked()
                                    {
                                        self.state.downloading = true;
                                        self.state.downloading_error = None;
                                        self.send(LogicCommand::UpdateMoonlight(
                                            self.config.selected_branch,
                                        ));
                                    }

                                    if self.state.downloading {
                                        ui.spinner();
                                    }
                                });
                            });
                        });

                    egui::CollapsingHeader::new("Discord installations")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                if let Some(err) = &self.state.patching_error {
                                    self.draw_error(ui, err);
                                }

                                if self.state.installs.is_none() {
                                    ui.spinner();
                                    return;
                                }

                                // lmao this is so jank I hate the borrow checker
                                let mut should_patch = Vec::new();
                                let mut should_unpatch = Vec::new();
                                let mut should_reset_config = Vec::new();

                                egui::Grid::new("install_grid").show(ui, |ui| {
                                    for install in self.state.installs.as_ref().unwrap() {
                                        ui.label(format!("{:?}", install.install.branch))
                                            .on_hover_text(install.install.path.to_string_lossy());

                                        ui.label("w/ moonlight");

                                        let selected_moonlight_branch = self
                                            .config
                                            .install_selected_branches
                                            .entry(install.install.path.clone())
                                            .or_insert(
                                                install
                                                    .install
                                                    .moonlight_info
                                                    .as_ref()
                                                    .map(|m| m.branch)
                                                    .unwrap_or(
                                                        install
                                                            .install
                                                            .branch
                                                            .preferred_moonlight_branch(),
                                                    ),
                                            );

                                        struct ComboBoxId<'a>(&'a Path);

                                        impl<'a> Hash for ComboBoxId<'a> {
                                            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                                                "moonlight_branch".hash(state);
                                                self.0.hash(state);
                                            }
                                        }

                                        egui::ComboBox::new(
                                            ComboBoxId(install.install.path.as_path()),
                                            "",
                                        )
                                        .selected_text(selected_moonlight_branch.name())
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                for branch in MoonlightBranch::ALL {
                                                    ui.selectable_value(
                                                        selected_moonlight_branch,
                                                        branch,
                                                        branch.name(),
                                                    );
                                                }
                                            },
                                        );

                                        let patch_button = egui::Button::new(if install.patched {
                                            "Unpatch"
                                        } else {
                                            "Patch"
                                        });
                                        let can_click_patch = !self.state.patching
                                            && (!install.patched
                                                && self
                                                    .state
                                                    .downloaded_versions
                                                    .as_ref()
                                                    .is_some_and(|map| {
                                                        map.contains_key(selected_moonlight_branch)
                                                    }))
                                            || (install.patched
                                                && install
                                                    .install
                                                    .moonlight_info
                                                    .as_ref()
                                                    .is_none_or(|m| {
                                                        m.branch == *selected_moonlight_branch
                                                    }));

                                        let reset_config_button = egui::Button::new("Reset config");
                                        let can_reset_config = install.has_config;

                                        let patch_clicked = ui
                                            .add_enabled(can_click_patch, patch_button)
                                            .on_disabled_hover_text(PATCH_TOOLIP)
                                            .clicked();

                                        if patch_clicked {
                                            if install.patched {
                                                should_unpatch.push(install.install.clone());
                                            } else {
                                                should_patch.push((
                                                    install.install.clone(),
                                                    *selected_moonlight_branch,
                                                ));
                                            }
                                        }

                                        let reset_config_clicked = ui
                                            .add_enabled(can_reset_config, reset_config_button)
                                            .on_hover_text(RESET_CONFIG_TOOLTIP)
                                            .clicked();
                                        if reset_config_clicked {
                                            should_reset_config.push(install.install.branch);
                                        }

                                        ui.end_row();
                                    }
                                });

                                for (install, branch) in should_patch {
                                    self.state.patching = true;
                                    self.state.patching_branch = Some(install.branch);
                                    self.state.patching_error = None;
                                    self.send(LogicCommand::PatchInstall { install, branch });
                                    self.send(LogicCommand::GetInstalls);
                                }
                                for install in should_unpatch {
                                    self.state.patching = true;
                                    self.state.patching_branch = Some(install.branch);
                                    self.state.patching_error = None;
                                    self.send(LogicCommand::UnpatchInstall(install));
                                    self.send(LogicCommand::GetInstalls);
                                }
                                for branch in should_reset_config {
                                    self.send(LogicCommand::ResetConfig(branch));
                                    self.state
                                        .installs
                                        .as_mut()
                                        .unwrap()
                                        .iter_mut()
                                        .find(|i| i.install.branch == branch)
                                        .unwrap()
                                        .has_config = false;
                                }
                            });
                        });
                });
        });

        // Since we're receiving messages on the UI thread, we need to be
        // repainting at least sometimes so the UI can update
        self.handle_messages();
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
