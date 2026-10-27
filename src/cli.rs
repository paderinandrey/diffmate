use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "diffmate", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Review { url: String },
    Status,
}
