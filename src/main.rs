#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
pub mod config;
pub mod installer;
pub mod logic;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 250.0])
            .with_min_inner_size([150.0, 150.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "moonlight installer",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!(format!("Error running app: {:?}", e)))?;

    Ok(())
}
