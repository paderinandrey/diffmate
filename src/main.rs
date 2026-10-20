mod agent;
mod app;
mod cli;
mod context;
#[cfg_attr(not(test), expect(dead_code))]
mod diff;
mod error;
mod gh;
mod git;
mod paths;
mod pr_url;
mod proc;
mod procenv;
mod render;
mod review;
mod runner;
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
    }
}
