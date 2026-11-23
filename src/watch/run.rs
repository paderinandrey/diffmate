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
use std::future::Future;
use tokio::sync::watch;

pub(super) async fn jobs<G: Git, H: Gh, A: Agent>(
    config: &AppConfig,
    store: &Store,
    deps: &Deps<G, H, A>,
    signals: &mut Signals,
) -> Result<(), Error> {
    let mut polling = poll::Schedule::new(config, store, &deps.gh);
    loop {
        if shutdown(signals).await {
            return Ok(());
        }
        if polling.delay().is_zero() {
            tokio::select! {
                biased;
                _ = signals.next(StopReason::Shutdown) => {
                    stopped();
                    return Ok(());
                }
                () = polling.poll() => {},
            }
            clean_refs(config, store, &deps.git).await?;
        }
        if let Some(queued) = store.next_runnable_job(now_ms()).await? {
            let work = alongside(config, store, deps, signals, &queued);
            let stop = polling.during(work).await?;
            clean_refs(config, store, &deps.git).await?;
            if stop {
                return Ok(());
            }
        } else if store.next_service_job(now_ms()).await?.is_some() {
            tokio::select! {
                biased;
                _ = signals.next(StopReason::Shutdown) => {
                    stopped();
                    return Ok(());
                }
                result = polling.during(serve(config, store, deps)) => result?,
            }
            clean_refs(config, store, &deps.git).await?;
        } else if idle(store, signals, polling.delay()).await? {
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
    let mut service: std::pin::Pin<Box<dyn Future<Output = Result<(), Error>> + '_>> =
        Box::pin(service_slot(config, store, deps));
    let mut review_result = None;
    let result = loop {
        let review_done = review_result.is_some();
        tokio::select! {
            biased;
            result = &mut review, if !review_done => {
                review_result = Some(result);
            }
            _ = signals.next(StopReason::Shutdown) => {
                drop(service);
                let job_id = queued.job.job_id;
                crate::review::signals::request_stop(&stop, store, job_id, StopReason::Shutdown)
                    .await;
                break match review_result {
                    Some(result) => result,
                    None => review.await,
                };
            }
            outcome = &mut service => {
                outcome?;
                if let Some(result) = review_result.take() {
                    break result;
                }
                service = Box::pin(service_slot(config, store, deps));
            }
        }
    };
    job::complete(config, store, queued, &result, &stop).await
}

async fn service_slot<G: Git, H: Gh, A: Agent>(
    config: &AppConfig,
    store: &Store,
    deps: &Deps<G, H, A>,
) -> Result<(), Error> {
    tokio::time::sleep(super::WATCH_LOCK_POLL).await;
    serve(config, store, deps).await
}

async fn serve<G: Git, H: Gh, A: Agent>(
    config: &AppConfig,
    store: &Store,
    deps: &Deps<G, H, A>,
) -> Result<(), Error> {
    crate::publish::serve_next(store, &deps.gh, config.settings.max_retries())
        .await
        .map(drop)
}

async fn clean_refs(config: &AppConfig, store: &Store, git: &impl Git) -> Result<(), Error> {
    for warning in crate::recovery::clean_closed_refs(&config.paths, store, git).await? {
        events::print(&format!("warning: {warning}"));
    }
    Ok(())
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

async fn idle(
    store: &Store,
    signals: &mut Signals,
    poll_delay: std::time::Duration,
) -> Result<bool, Error> {
    let delay = timing::idle_delay(now_ms(), poll_delay, store.earliest_next_attempt().await?);
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
