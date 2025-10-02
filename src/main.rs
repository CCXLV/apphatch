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
            if let Err(err) = install::install_app(&path) {
                eprintln!("install failed: {}", err);
                std::process::exit(1);
            }
        }
        cli::Commands::Uninstall { name: _ } => {}
    }
}
