use clap::Parser;

mod cli;
mod desktop;
mod install;
mod uninstall;
mod upgrade;
mod utils;

fn main() {
    let cli: cli::Cli = cli::Cli::parse();

    match &cli.command {
        cli::Commands::Install { path } => {
            if let Err(err) = install::install_app(&path) {
                eprintln!("Install failed: {}", err);
                std::process::exit(1);
            }
        }
        cli::Commands::Uninstall { name } => {
            if let Err(err) = uninstall::uninstall_app(&name) {
                eprintln!("Uninstall failed: {}", err);
                std::process::exit(1);
            }
        }
        cli::Commands::Upgrade { name, path } => {
            if let Err(err) = upgrade::upgrade_app(&name, &path) {
                eprint!("Upgrade failed: {}", err);
                std::process::exit(1);
            }
        }
    }
}
