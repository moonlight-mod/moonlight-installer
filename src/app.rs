use std::time::Duration;

use crate::{
    config::Config,
    installer::{
        types::*,
        util::{branch_desc, branch_name},
    },
    logic::{app_logic_thread, LogicCommand, LogicResponse},
};

#[derive(Debug, Default)]
pub struct AppState {
    downloaded_version: Option<Option<String>>,
    latest_version: Option<String>,
    installs: Option<Vec<InstallInfo>>,

    downloading: bool,
    downloading_error: Option<InstallerError>,

    patching: bool,
    patching_error: Option<InstallerError>,
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

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        let (main_tx, logic_rx) = flume::unbounded::<LogicCommand>();
        let (logic_tx, main_rx) = flume::unbounded::<LogicResponse>();
        std::thread::spawn(move || {
            if let Err(err) = app_logic_thread(logic_rx, logic_tx) {
                log::error!("Logic thread error: {:?}", err);
            }
        });

        app.tx = Some(main_tx);
        app.rx = Some(main_rx);

        app.send(LogicCommand::GetDownloadedVersion);
        app.send(LogicCommand::GetLatestVersion(app.config.branch));
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

                LogicResponse::DownloadedVersion(version) => {
                    log::info!("Downloaded version: {:?}", version);
                    self.state.downloaded_version = Some(version);
                }

                LogicResponse::LatestVersion(version) => {
                    log::info!("Latest version: {:?}", version);
                    if let Ok(version) = version {
                        self.state.latest_version = Some(version);
                        self.state.downloading_error = None;
                    } else {
                        self.state.latest_version = None;
                        self.state.downloading_error = version.err();
                    }
                }

                LogicResponse::UpdateComplete(version) => {
                    log::info!("Update complete: {:?}", version);
                    if let Ok(version) = version {
                        self.state.downloaded_version = Some(Some(version));
                        self.state.downloading_error = None;
                    } else {
                        self.state.downloaded_version = Some(None);
                        self.state.downloading_error = version.err();
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

    fn draw_error(&self, ui: &mut egui::Ui, err: &InstallerError) {
        ui.heading(egui::RichText::new("Error").color(egui::Color32::RED));

        match err.code {
            ErrorCode::WindowsFileLock => {
                ui.label(
                    "Discord is currently open, which locks moonlight's ability to modify its files. Please completely close Discord and make sure it does not appear in the taskbar."
                );
                ui.label(
                    "Alternatively, click the button below to attempt to close Discord forcefully. This will disconnect you from any voice calls you are in and may cause issues."
                );

                if ui.button("Force close Discord").clicked() {
                    if let Some(branch) = self.state.patching_branch {
                        self.send(LogicCommand::KillDiscord(branch));
                    }
                }
            }

            ErrorCode::MacOSNoPermission => {
                ui.label("moonlight is unable to modify your Discord installation. This is because your MacOS system privacy settings doesn't allow us to do so.");
                ui.label("You can fix this via a pop-up you should've gotten, or by going to System Settings > Privacy & Security > App Management and allowing moonlight installer.");
            }

            ErrorCode::NetworkFailed => {
                ui.label("moonlight is unable to download required files, likely due to a network issue.");
                ui.label("Please check your internet connection and try again.");
            }

            _ => {
                ui.label("An unknown error occurred. Please report this.");
                ui.label(err.message.clone());
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
            ui.heading("moonlight installer");

            egui::CollapsingHeader::new("Download moonlight")
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(err) = &self.state.downloading_error {
                        self.draw_error(ui, err);
                    }

                    ui.vertical(|ui| {
                        egui::ComboBox::from_label("Selected branch")
                            .selected_text(branch_name(self.config.branch))
                            .show_ui(ui, |ui| {
                                for &branch in &[MoonlightBranch::Stable, MoonlightBranch::Nightly]
                                {
                                    let str = format!(
                                        "{}\n  {}",
                                        branch_name(branch),
                                        branch_desc(branch)
                                    );
                                    if ui
                                        .selectable_value(&mut self.config.branch, branch, str)
                                        .changed()
                                    {
                                        self.state.latest_version = None;
                                        self.send(LogicCommand::GetLatestVersion(
                                            self.config.branch,
                                        ));
                                    }
                                }
                            });

                        ui.horizontal(|ui| {
                            ui.label("Latest version:");
                            if let Some(version) = &self.state.latest_version {
                                ui.label(version);
                            } else {
                                ui.spinner();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Downloaded version:");
                            if let Some(version) = &self.state.downloaded_version {
                                ui.label(version.as_deref().unwrap_or("None"));
                            } else {
                                ui.spinner();
                            }
                        });

                        if self.state.downloading
                            || self.state.downloaded_version.is_none()
                            || self.state.latest_version.is_none()
                            || self.state.downloaded_version
                                == Some(self.state.latest_version.clone())
                        {
                            ui.disable();
                        }

                        ui.horizontal(|ui| {
                            if ui.button("Download").clicked() {
                                self.state.downloading = true;
                                self.state.downloading_error = None;
                                self.send(LogicCommand::UpdateMoonlight(self.config.branch));
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

                        if self.state.patching {
                            ui.disable();
                        }

                        if self.state.installs.is_none() {
                            ui.spinner();
                            return;
                        }

                        // lmao this is so jank I hate the borrow checker
                        let mut should_patch = Vec::new();
                        let mut should_unpatch = Vec::new();

                        for install in self.state.installs.as_ref().unwrap() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{:?}", install.install.branch))
                                    .on_hover_text(install.install.path.to_string_lossy());

                                if install.patched {
                                    if ui.button("Unpatch").clicked() {
                                        should_unpatch.push(install.install.clone());
                                    }
                                } else {
                                    if ui.button("Patch").clicked() {
                                        should_patch.push(install.install.clone());
                                    }
                                }
                            });
                        }

                        for install in should_patch {
                            self.state.patching = true;
                            self.state.patching_branch = Some(install.branch);
                            self.state.patching_error = None;
                            self.send(LogicCommand::PatchInstall(install));
                        }
                        for install in should_unpatch {
                            self.state.patching = true;
                            self.state.patching_branch = Some(install.branch);
                            self.state.patching_error = None;
                            self.send(LogicCommand::UnpatchInstall(install));
                        }
                    });
                })
        });

        // Since we're receiving messages on the UI thread, we need to be
        // repainting at least sometimes so the UI can update
        self.handle_messages();
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
