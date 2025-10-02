use clap::Parser;

mod cli;
mod desktop;
mod install;
mod uninstall;
mod utils;

fn main() {
    let cli: cli::Cli = cli::Cli::parse();

    match &cli.command {
        cli::Commands::Install { path } => {
            install::install_app(&path).unwrap();
        }
        cli::Commands::Uninstall { name } => {}
    }
}
