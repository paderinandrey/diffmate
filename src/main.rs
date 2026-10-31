mod agent;
mod app;
mod cancel;
mod cli;
mod config;
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
#[cfg(test)]
mod tests;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    let mut config = match app::AppConfig::from_process() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    if matches!(cli.command, cli::Command::Review { .. }) {
        match config::load(&config.config_file) {
            Ok(settings) => config.settings = settings,
            Err(error) => {
                eprintln!(
                    "error: {}",
                    render::sanitize(&error.to_string()).replace('\n', " ")
                );
                return ExitCode::FAILURE;
            }
        }
    }
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
