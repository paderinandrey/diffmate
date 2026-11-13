use super::events;
use crate::app::AppConfig;
use crate::error::Error;
use crate::notify;
use crate::runner::{Follow, ReviewResult};
use crate::store::records::JobOrigin;
use crate::store::{Store, now_ms, retry::QueuedJob};
use tokio::sync::watch;

pub(super) async fn complete(
    config: &AppConfig,
    store: &Store,
    queued: &QueuedJob,
    result: &ReviewResult,
    stop: &watch::Sender<bool>,
) -> Result<bool, Error> {
    follow(config, store, queued, result).await?;
    let prefix = format!("job {} review {}", queued.job.job_id, queued.pr.label());
    events::print(&format!("{prefix} {}", events::outcome(result)));
    announce(store, queued, result).await?;
    let halt = *stop.borrow();
    if halt {
        super::run::stopped();
    }
    Ok(halt)
}

async fn announce(store: &Store, queued: &QueuedJob, result: &ReviewResult) -> Result<(), Error> {
    let notice = notify::for_result(
        store,
        queued.job.job_id,
        queued.origin,
        result,
        &queued.pr.label(),
    )
    .await?;
    let Some(notice) = notice else {
        return Ok(());
    };
    if let Err(error) = notify::deliver(&notice).await {
        events::print(&format!("notification-failed: {error}"));
    }
    Ok(())
}

async fn follow(
    config: &AppConfig,
    store: &Store,
    queued: &QueuedJob,
    result: &ReviewResult,
) -> Result<(), Error> {
    let ReviewResult::Succeeded { follow, .. } = result else {
        return Ok(());
    };
    if queued.origin != JobOrigin::Automatic || *follow == Follow::Stop {
        return Ok(());
    }
    let debounce = i64::try_from(config.settings.debounce().as_millis()).unwrap_or(i64::MAX);
    store
        .schedule_after_run(
            queued.job.target_id,
            now_ms(),
            debounce,
            &config.process,
            *follow == Follow::Hold,
        )
        .await
}
