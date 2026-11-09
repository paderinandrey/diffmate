mod events;
mod job;
mod poll;
mod run;
mod timing;
mod wait;

use crate::agent::claude::ClaudeCli;
use crate::app::AppConfig;
use crate::cancel::StopReason;
use crate::crash::crash_point;
use crate::error::Error;
use crate::gh::GhCli;
use crate::git::GitCli;
use crate::notify;
use crate::review::signals::Signals;
use crate::runner::Deps;
use crate::{recovery, store::Store};
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
    let mut signals = Signals::new()?;
    let store = Store::open(&config.paths).await?;
    let deps = Deps {
        git: GitCli::new(&config.env),
        gh: GhCli::new(&config.env),
        agent: ClaudeCli::new(&config.env)?,
        settings: config.settings.clone(),
    };
    let path = config.paths.lock();
    let acquire = wait::lock(&path);
    let runner_lock = tokio::select! {
        biased;
        _ = signals.next(StopReason::Shutdown) => {
            run::stopped();
            return Ok(());
        },
        lock = acquire => lock?,
    };
    runner_lock.announce(crate::lock::Role::Watch).await?;
    crash_point!("locked");
    let report = recovery::run(&config.paths, &store, &deps.git, &config.settings).await?;
    crash_point!("recovered");
    for line in report.lines() {
        events::print(&line);
    }
    for notice in notify::recovery::notices(&store, &report.jobs).await? {
        if let Err(error) = notify::deliver(&notice).await {
            events::print(&format!("notification-failed: {error}"));
        }
    }
    run::jobs(config, &store, &deps, &mut signals).await
}
