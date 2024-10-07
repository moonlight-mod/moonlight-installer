use std::path::PathBuf;

use clap::Parser;
use libmoonlight::{detect_install, types::MoonlightBranch};

#[derive(Parser)]
#[clap(version)]
pub enum Args {
    /// Install or update moonlight
    Install { branch: MoonlightBranch },

    /// Patch a Discord install
    Patch { exe: PathBuf },

    /// Unpatch a Discord install
    Unpatch { exe: PathBuf },
}

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().filter_or("MOONLIGHT_LOG", "info"));
    let args = Args::parse();
    let installer = libmoonlight::Installer::new();

    match args {
        Args::Install { branch } => {
            log::info!("Downloading moonlight branch {}", branch);
            let ver = installer.download_moonlight(branch)?;
            log::info!("Downloaded version {}", ver);
        }

        Args::Patch { exe: dir } => {
            log::info!("Patching install at {:?}", dir);
            let install = detect_install(&dir);
            if let Some(install) = install {
                if install.patched {
                    log::warn!("Install already patched");
                    std::process::exit(0);
                }

                installer.patch_install(install.install)?;
                log::info!("Patched install at {:?}", dir);
            } else {
                log::error!("Failed to detect install at {:?}", dir);
                std::process::exit(1);
            }
        }

        Args::Unpatch { exe: dir } => {
            log::info!("Unpatching install at {:?}", dir);
            let install = detect_install(&dir);
            if let Some(install) = install {
                if !install.patched {
                    log::warn!("Install already unpatched");
                    std::process::exit(0);
                }

                installer.unpatch_install(install.install)?;
                log::info!("Unpatched install at {:?}", dir);
            } else {
                log::error!("Failed to detect install at {:?}", dir);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
