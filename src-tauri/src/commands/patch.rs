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

            installs
        }

        // TODO: linux support
        "linux" => vec![],
        _ => vec![],
    }
}

fn get_app_dir(install: DetectedInstall) -> PathBuf {
    match std::env::consts::OS {
        "windows" => install.path.join("resources"),
        "macos" => install.path,
        _ => todo!(),
    }
}

#[tauri::command]
pub fn is_install_patched(install: DetectedInstall) -> bool {
    !get_app_dir(install).join("app.asar").exists()
}

#[tauri::command]
pub fn patch_install(install: DetectedInstall) -> Result<(), Error> {
    // TODO: flatpak, etc whatever the fuck
    let app_dir = get_app_dir(install);
    let asar = app_dir.join("app.asar");
    std::fs::rename(&asar, asar.with_file_name("_app.asar"))?;
    std::fs::create_dir(app_dir.join("app"))?;

    let json = serde_json::json!({
      "name": "discord",
      "main": "./injector.js",
      "private": true
    });
    std::fs::write(app_dir.join("app/package.json"), json.to_string())?;

    let moonlight_injector = get_moonlight_dir()?.join("dist").join("injector.js");
    let moonlight_injector_str = serde_json::to_string(&moonlight_injector).unwrap();
    let injector = format!(
        r#"require({}).inject(
  require("path").resolve(__dirname, "../_app.asar")
);
"#,
        moonlight_injector_str
    );
    std::fs::write(app_dir.join("app/injector.js"), injector)?;
    Ok(())
}

#[tauri::command]
pub fn unpatch_install(install: DetectedInstall) -> Result<(), Error> {
    let app_dir = get_app_dir(install);
    let asar = app_dir.join("_app.asar");
    std::fs::rename(&asar, asar.with_file_name("app.asar"))?;
    std::fs::remove_dir_all(app_dir.join("app"))?;
    Ok(())
}
