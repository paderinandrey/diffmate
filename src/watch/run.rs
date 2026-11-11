use super::{events, job, poll, timing};
use crate::agent::Agent;
use crate::app::AppConfig;
use crate::cancel::StopReason;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::review::signals::Signals;
use crate::runner::Deps;
use crate::store::{Store, now_ms, retry::QueuedJob};
use tokio::sync::watch;

pub(super) async fn jobs<G: Git, H: Gh, A: Agent>(
    config: &AppConfig,
    store: &Store,
    deps: &Deps<G, H, A>,
    signals: &mut Signals,
) -> Result<(), Error> {
    let mut next_poll_at = 0;
    loop {
        if shutdown(signals).await {
            return Ok(());
        }
        if now_ms() >= next_poll_at {
            let polled = tokio::select! {
                biased;
                _ = signals.next(StopReason::Shutdown) => {
                    stopped();
                    return Ok(());
                }
                result = poll::once(
                    &deps.gh,
                    store,
                    config.settings.debounce(),
                    &config.process,
                ) => result,
            };
            next_poll_at = now_ms().saturating_add(interval(config));
            if let Err(error) = polled {
                events::print(&format!("poll failed: {error}"));
            }
        }
        if let Some(queued) = store.next_runnable_job(now_ms()).await? {
            if alongside(config, store, deps, signals, &queued).await? {
                return Ok(());
            }
        } else if store.next_service_job(now_ms()).await?.is_some() {
            serve(config, store, deps).await?;
        } else if idle(store, signals, next_poll_at).await? {
            return Ok(());
        }
    }
}

async fn alongside<G: Git, H: Gh, A: Agent>(
    config: &AppConfig,
    store: &Store,
    deps: &Deps<G, H, A>,
    signals: &mut Signals,
    queued: &QueuedJob,
) -> Result<bool, Error> {
    let prefix = format!("job {} review {}", queued.job.job_id, queued.pr.label());
    events::print(&format!("{prefix} started"));
    let (stop, cancel) = watch::channel(false);
    let review = crate::runner::execute_review_job(
        &config.paths,
        store,
        deps,
        &queued.pr,
        &queued.job,
        &cancel,
    );
    let mut review = std::pin::pin!(review);
    let result = loop {
        tokio::select! {
            biased;
            result = &mut review => break result,
            _ = signals.next(StopReason::Shutdown) => {
                let job_id = queued.job.job_id;
                crate::review::signals::request_stop(&stop, store, job_id, StopReason::Shutdown)
                    .await;
                break review.await;
            }
            () = tokio::time::sleep(super::WATCH_LOCK_POLL) => serve(config, store, deps).await?,
        }
    };
    job::complete(config, store, queued, &result, &stop).await
}

async fn serve<G: Git, H: Gh, A: Agent>(
    config: &AppConfig,
    store: &Store,
    deps: &Deps<G, H, A>,
) -> Result<(), Error> {
    let Some(job) = store.next_service_job(now_ms()).await? else {
        return Ok(());
    };
    match job.kind.as_str() {
        "publish" => {
            store.start_job(job.id).await?;
            crate::publish::execute_publish(store, &deps.gh, job.id, config.settings.max_retries())
                .await
        }
        "reconcile" => {
            store.start_job(job.id).await?;
            crate::publish::execute_reconcile(store, &deps.gh, job.id).await
        }
        "sync" => {
            store.start_job(job.id).await?;
            crate::publish::execute_sync(store, &deps.gh, job.id).await
        }
        _ => {
            tokio::time::sleep(super::WATCH_LOCK_POLL).await;
            Ok(())
        }
    }
}

fn interval(config: &AppConfig) -> i64 {
    i64::try_from(config.settings.poll_interval().as_millis()).unwrap_or(i64::MAX)
}

async fn shutdown(signals: &mut Signals) -> bool {
    tokio::select! {
        biased;
        _ = signals.next(StopReason::Shutdown) => {
            stopped();
            true
        }
        () = std::future::ready(()) => false,
    }
}

async fn idle(store: &Store, signals: &mut Signals, next_poll_at: i64) -> Result<bool, Error> {
    let delay = timing::idle_delay(now_ms(), next_poll_at, store.earliest_next_attempt().await?);
    tokio::select! {
        () = tokio::time::sleep(delay) => Ok(false),
        _ = signals.next(StopReason::Shutdown) => {
            stopped();
            Ok(true)
        }
    }
}

pub(super) fn stopped() {
    events::print("watch stopped (shutdown)");
}
