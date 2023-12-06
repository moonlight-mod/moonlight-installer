use tauri::AppHandle;

use crate::types::*;
use std::path::PathBuf;

use super::get_moonlight_dir;

#[tauri::command]
pub fn detect_installs() -> Vec<DetectedInstall> {
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
                    let mut app_dirs: Vec<_> = std::fs::read_dir(&path)
                        .unwrap()
                        .map(|x| x.unwrap())
                        .filter(|x| x.file_name().to_str().unwrap().starts_with("app-"))
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

            installs
        }
        // TODO
        "macos" => vec![],
        "linux" => vec![],
        _ => vec![],
    }
}

#[tauri::command]
pub fn is_install_patched(install: DetectedInstall) -> bool {
    !install.path.join("resources/app.asar").exists()
}

#[tauri::command]
pub fn patch_install(app_handle: AppHandle, install: DetectedInstall) {
    // TODO: macos, flatpak, etc whatever the fuck
    let asar = install.path.join("resources/app.asar");
    std::fs::rename(&asar, asar.with_file_name("_app.asar")).unwrap();
    std::fs::create_dir(install.path.join("resources/app")).unwrap();

    let json = serde_json::json!({
      "name": "discord",
      "main": "./injector.js",
      "private": true
    });
    std::fs::write(
        install.path.join("resources/app/package.json"),
        json.to_string(),
    )
    .unwrap();

    let moonlight_injector = get_moonlight_dir(&app_handle)
        .join("dist")
        .join("injector.js");
    let moonlight_injector_str = serde_json::to_string(&moonlight_injector).unwrap();
    let injector = format!(
        r#"require({}).inject(
  require("path").resolve(__dirname, "../_app.asar")
);
"#,
        moonlight_injector_str
    );
    std::fs::write(install.path.join("resources/app/injector.js"), injector).unwrap();
}

#[tauri::command]
pub fn unpatch_install(install: DetectedInstall) {
    let asar = install.path.join("resources/_app.asar");
    std::fs::rename(&asar, asar.with_file_name("app.asar")).unwrap();
    std::fs::remove_dir_all(install.path.join("resources/app")).unwrap();
}
