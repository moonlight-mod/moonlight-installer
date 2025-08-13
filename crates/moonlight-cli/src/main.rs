use clap::{CommandFactory, Parser, Subcommand};
use libmoonlight::types::MoonlightBranch;
use libmoonlight::{detect_install, get_download_dir};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    author=clap::crate_authors!(),
    version=clap::crate_version!(),
    long_version=clap::crate_version!(),
    about="moonlight installer",
    subcommand_required=true,
    arg_required_else_help=true,
)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install or update moonlight
    Install { branch: MoonlightBranch },

    /// Patch a Discord install
    Patch {
        exe: PathBuf,

        branch: MoonlightBranch,

        /// Path to a custom moonlight build
        #[clap(long, short)]
        moonlight: Option<PathBuf>,
    },

    /// Unpatch a Discord install
    Unpatch { exe: PathBuf },

    /// Generate shell completions
    Completions {
        #[clap(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    env_logger::init_from_env(env_logger::Env::new().filter_or("MOONLIGHT_LOG", "info"));
    let cli = Cli::parse();
    let installer = libmoonlight::Installer::new();

    match cli.command {
        Commands::Install { branch } => {
            log::info!("Downloading moonlight branch {}", branch);
            let info = installer.download_moonlight(branch)?;
            log::info!("Downloaded version {}", info.version);
            installer.set_downloaded_version(branch, info)?;
        }

        Commands::Patch {
            exe,
            branch,
            moonlight,
        } => {
            log::info!("Patching install at {:?}", exe);
            let install = detect_install(&exe);
            if let Some(install) = install {
                if install.patched {
                    log::warn!("Client mod appears to already be patched - unpatching first");
                    installer.unpatch_install(&install.install)?;
                }

                installer.patch_install(
                    &install.install,
                    moonlight.unwrap_or_else(|| get_download_dir(branch)),
                    branch,
                )?;
                log::info!("Patched install at {:?}", exe);
            } else {
                log::error!("Failed to detect install at {:?}", exe);
                std::process::exit(1);
            }
        }

        Commands::Unpatch { exe } => {
            log::info!("Unpatching install at {:?}", exe);
            let install = detect_install(&exe);
            if let Some(install) = install {
                if !install.patched {
                    log::warn!("Install already unpatched");
                    std::process::exit(0);
                }

                installer.unpatch_install(&install.install)?;
                log::info!("Unpatched install at {:?}", exe);
            } else {
                log::error!("Failed to detect install at {:?}", exe);
                std::process::exit(1);
            }
        }

        Commands::Completions { shell } => clap_complete::generate(
            shell,
            &mut Cli::command(),
            "moonlight-cli",
            &mut std::io::stdout().lock(),
        ),
    }

    Ok(())
}
