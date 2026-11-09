use super::{events, job, poll, timing};
use crate::agent::Agent;
use crate::app::AppConfig;
use crate::cancel::StopReason;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::review::signals::Signals;
use crate::runner::Deps;
use crate::store::{Store, now_ms};

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
            let polled =
                poll::once(&deps.gh, store, config.settings.debounce(), &config.process).await;
            next_poll_at = now_ms().saturating_add(interval(config));
            if let Err(error) = polled {
                events::print(&format!("poll failed: {error}"));
            }
        }
        if let Some(queued) = store.next_runnable_job(now_ms()).await? {
            if job::run(config, store, deps, signals, &queued).await? {
                return Ok(());
            }
        } else if idle(store, signals, next_poll_at).await? {
            return Ok(());
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
