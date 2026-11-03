mod events;
mod run;

use crate::agent::claude::ClaudeCli;
use crate::app::AppConfig;
use crate::error::Error;
use crate::gh::GhCli;
use crate::git::GitCli;
use crate::runner::Deps;
use crate::{lock, recovery, store::Store};
use std::process::ExitCode;
use std::time::Duration;

pub(crate) const WATCH_LOCK_POLL: Duration = Duration::from_secs(1);
pub(crate) const WATCH_IDLE_POLL: Duration = Duration::from_secs(1);

pub(crate) async fn run(config: &AppConfig) -> ExitCode {
    match execute(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            events::print(&format!("error: {error}"));
            ExitCode::FAILURE
        }
    }
}

async fn execute(config: &AppConfig) -> Result<(), Error> {
    let store = Store::open(&config.paths).await?;
    let deps = Deps {
        git: GitCli::new(&config.env),
        gh: GhCli::new(&config.env),
        agent: ClaudeCli::new(&config.env)?,
        settings: config.settings.clone(),
    };
    let _lock = lock::acquire(&config.paths.lock(), WATCH_LOCK_POLL, || {
        events::print("waiting for the runner lock");
    })
    .await?;
    let report = recovery::run(&config.paths, &store, &deps.git, &config.settings).await?;
    for line in report.lines() {
        events::print(&line);
    }
    run::jobs(config, &store, &deps).await
}
