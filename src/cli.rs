use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "diffmate", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Review {
        url: String,
        #[arg(long)]
        force: bool,
    },
    Status,
    Watch,
    Tui,
    #[command(hide = true)]
    Debug {
        #[command(subcommand)]
        command: DebugCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum DebugCommand {
    Enqueue {
        url: String,
        #[arg(long)]
        automatic: bool,
        #[arg(long)]
        force: bool,
    },
    Discard {
        url: String,
    },
}
