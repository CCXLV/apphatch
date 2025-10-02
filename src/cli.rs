use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "AppHatch")]
#[command(version = "0.1.0")]
#[command(author = "CCXLV")]
#[command(about = "Installs AppImages easily")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Install {
        #[arg(short, long)]
        path: String,
    },
    Uninstall {
        #[arg(short, long)]
        name: String,
    },
}
