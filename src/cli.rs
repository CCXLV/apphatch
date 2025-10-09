use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "apphatch")]
#[command(version = "0.1.0")]
#[command(author = "CCXLV")]
#[command(about = "Installs AppImages easily")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Install an app")]
    Install {
        #[arg(help = "The path to the AppImage file")]
        path: String,
    },

    #[command(about = "Uninstall an app")]
    Uninstall {
        #[arg(help = "The name of the app to uninstall, it is case sensitive.")]
        name: String,
    },

    #[command(
        about = "Upgrade an already installed app",
        override_usage = "apphatch upgrade <NAME> --path <FILE>"
    )]
    Upgrade {
        #[arg(help = "The name of the app to upgrade, it is case sensitive.")]
        name: String,

        #[arg(
            short,
            long,
            value_name = "FILE",
            help = "The path to the AppImage file"
        )]
        path: String,
    },
}
