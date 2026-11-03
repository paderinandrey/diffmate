use super::{WATCH_IDLE_POLL, events};
use crate::agent::Agent;
use crate::app::AppConfig;
use crate::cancel::StopReason;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::review::signals::{self, Signals};
use crate::runner::{self, Deps};
use crate::store::{Store, now_ms};
use std::time::Duration;
use tokio::sync::watch;

pub(super) async fn jobs<G: Git, H: Gh, A: Agent>(
    config: &AppConfig,
    store: &Store,
    deps: &Deps<G, H, A>,
    signals: &mut Signals,
) -> Result<(), Error> {
    loop {
        let queued = tokio::select! {
            biased;
            _ = signals.next(StopReason::Shutdown) => {
                stopped();
                return Ok(());
            },
            queued = store.next_runnable_job(now_ms()) => queued?,
        };
        if let Some(queued) = queued {
            let prefix = format!("job {} review {}", queued.job.job_id, queued.pr.label());
            events::print(&format!("{prefix} started"));
            let (stop, cancel) = watch::channel(false);
            let job = runner::execute_review_job(
                &config.paths,
                store,
                deps,
                &queued.pr,
                &queued.job,
                &cancel,
            );
            let result = signals::until_done(
                stop,
                job,
                store,
                queued.job.job_id,
                StopReason::Shutdown,
                signals,
            )
            .await?;
            events::print(&format!("{prefix} {}", events::outcome(&result)));
            if *cancel.borrow() {
                stopped();
                return Ok(());
            }
        } else {
            let delay = store
                .earliest_next_attempt()
                .await?
                .map_or(WATCH_IDLE_POLL, |at| {
                    Duration::from_millis(at.saturating_sub(now_ms()).max(0).unsigned_abs())
                        .min(WATCH_IDLE_POLL)
                });
            tokio::select! {
                () = tokio::time::sleep(delay) => continue,
                _ = signals.next(StopReason::Shutdown) => {},
            }
            stopped();
            return Ok(());
        }
    }
}

pub(super) fn stopped() {
    events::print("watch stopped (shutdown)");
}
