use std::path::PathBuf;
use clap::{CommandFactory, Parser};
use libmoonlight::{detect_install, types::MoonlightBranch};

#[derive(Parser)]
#[clap(version)]
pub enum Args {
    /// Install or update moonlight
    Install { branch: MoonlightBranch },

    /// Patch a Discord install
    Patch {
        exe: PathBuf,

        /// Path to a custom moonlight build
        #[clap(long, short)]
        moonlight: Option<PathBuf>,
    },

    /// Unpatch a Discord install
    Unpatch { exe: PathBuf },

    /// Generate shell completions
    Completions { shell: clap_complete::Shell },
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    env_logger::init_from_env(env_logger::Env::new().filter_or("MOONLIGHT_LOG", "info"));
    let args = Args::parse();
    let installer = libmoonlight::Installer::new();

    match args {
        Args::Install { branch } => {
            log::info!("Downloading moonlight branch {}", branch);
            let ver = installer.download_moonlight(branch)?;
            installer.set_downloaded_version(&ver)?;
            log::info!("Downloaded version {}", ver);
        }

        Args::Patch { exe, moonlight } => {
            log::info!("Patching install at {:?}", exe);
            let install = detect_install(&exe);
            if let Some(install) = install {
                if install.patched {
                    log::warn!("Install already patched");
                    std::process::exit(0);
                }

                installer.patch_install(install.install, moonlight)?;
                log::info!("Patched install at {:?}", exe);
            } else {
                log::error!("Failed to detect install at {:?}", exe);
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

        Args::Completions { shell } => {
            let mut app = Args::command();
            let bin_name = app.get_name().to_string();
            clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());
        }
    }

    Ok(())
}
