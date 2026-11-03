use super::{WATCH_IDLE_POLL, events};
use crate::agent::Agent;
use crate::app::AppConfig;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::runner::{self, Deps};
use crate::store::{Store, now_ms};
use std::time::Duration;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::watch;

pub(super) async fn jobs<G: Git, H: Gh, A: Agent>(
    config: &AppConfig,
    store: &Store,
    deps: &Deps<G, H, A>,
) -> Result<(), Error> {
    let mut interrupts = signal(SignalKind::interrupt()).map_err(Error::io("listen for SIGINT"))?;
    let mut terminate = signal(SignalKind::terminate()).map_err(Error::io("listen for SIGTERM"))?;
    loop {
        if let Some(queued) = store.next_runnable_job(now_ms()).await? {
            let prefix = format!("job {} review {}", queued.job.job_id, queued.pr.label());
            events::print(&format!("{prefix} started"));
            let (_stop, cancel) = watch::channel(false);
            let result = runner::execute_review_job(
                &config.paths,
                store,
                deps,
                &queued.pr,
                &queued.job,
                &cancel,
            )
            .await;
            events::print(&format!("{prefix} {}", events::outcome(&result)));
        } else {
            let delay = store
                .earliest_next_attempt()
                .await?
                .map_or(WATCH_IDLE_POLL, |at| {
                    Duration::from_millis(at.saturating_sub(now_ms()).unsigned_abs())
                        .min(WATCH_IDLE_POLL)
                });
            tokio::select! {
                () = tokio::time::sleep(delay) => continue,
                _ = interrupts.recv() => {},
                _ = terminate.recv() => {},
            }
            events::print("watch stopped (shutdown)");
            return Ok(());
        }
    }
}
