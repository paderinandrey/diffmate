mod agent;
mod app;
mod cli;
mod context;
mod diff;
mod error;
mod gh;
mod git;
mod lock;
mod normalize;
mod paths;
mod pr_url;
mod proc;
mod procenv;
mod recovery;
mod render;
mod review;
mod runner;
mod status;
mod store;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    let config = match app::AppConfig::from_process() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut builder = tokio::runtime::Builder::new_current_thread();
    let runtime = match builder.enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("error: cannot start the async runtime: {error}");
            return ExitCode::FAILURE;
        }
    };
    match cli.command {
        cli::Command::Review { url } => runtime.block_on(review::run(&config, &url)),
        cli::Command::Status => runtime.block_on(status::run(&config)),
    }
}
