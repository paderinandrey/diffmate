use super::events;
use crate::agent::Agent;
use crate::app::AppConfig;
use crate::cancel::StopReason;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::review::signals::{self, Signals};
use crate::runner::{self, Deps, ReviewResult};
use crate::store::records::JobOrigin;
use crate::store::{Store, now_ms, retry::QueuedJob};
use tokio::sync::watch;

pub(super) async fn run<G: Git, H: Gh, A: Agent>(
    config: &AppConfig,
    store: &Store,
    deps: &Deps<G, H, A>,
    signals: &mut Signals,
    queued: &QueuedJob,
) -> Result<bool, Error> {
    let prefix = format!("job {} review {}", queued.job.job_id, queued.pr.label());
    events::print(&format!("{prefix} started"));
    let (stop, cancel) = watch::channel(false);
    let review =
        runner::execute_review_job(&config.paths, store, deps, &queued.pr, &queued.job, &cancel);
    let result = signals::until_done(
        stop,
        review,
        store,
        queued.job.job_id,
        StopReason::Shutdown,
        signals,
    )
    .await?;
    follow(config, store, queued, &result).await?;
    events::print(&format!("{prefix} {}", events::outcome(&result)));
    let halt = *cancel.borrow();
    if halt {
        super::run::stopped();
    }
    Ok(halt)
}

async fn follow(
    config: &AppConfig,
    store: &Store,
    queued: &QueuedJob,
    result: &ReviewResult,
) -> Result<(), Error> {
    if queued.origin != JobOrigin::Automatic || !matches!(result, ReviewResult::Succeeded { .. }) {
        return Ok(());
    }
    let debounce = i64::try_from(config.settings.debounce().as_millis()).unwrap_or(i64::MAX);
    store
        .schedule_after_run(queued.job.target_id, now_ms(), debounce, &config.process)
        .await
}
