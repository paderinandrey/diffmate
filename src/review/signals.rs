use crate::cancel::StopReason;
use crate::error::Error;
use crate::render;
use crate::store::{JobId, Store};
use std::future::Future;
use std::pin::pin;
use tokio::signal::unix::{Signal, SignalKind, signal};
use tokio::sync::watch;
#[cfg(test)]
mod tests;

pub(crate) struct Signals {
    interrupts: Signal,
    terminate: Signal,
}

impl Signals {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            interrupts: signal(SignalKind::interrupt()).map_err(Error::io("listen for SIGINT"))?,
            terminate: signal(SignalKind::terminate()).map_err(Error::io("listen for SIGTERM"))?,
        })
    }

    pub async fn next(&mut self, interrupt: StopReason) -> StopReason {
        tokio::select! {
            _ = self.interrupts.recv() => interrupt,
            _ = self.terminate.recv() => StopReason::Shutdown,
        }
    }
}

pub(crate) async fn until_done<F: Future>(
    stop: watch::Sender<bool>,
    job: F,
    store: &Store,
    job_id: JobId,
    interrupt: StopReason,
    signals: &mut Signals,
) -> Result<F::Output, Error> {
    let mut job = pin!(job);
    let reason = tokio::select! {
        biased;
        output = &mut job => return Ok(output),
        reason = signals.next(interrupt) => reason,
    };
    request_stop(&stop, store, job_id, reason).await;
    Ok(job.await)
}

async fn request_stop(
    stop: &watch::Sender<bool>,
    store: &Store,
    job_id: JobId,
    reason: StopReason,
) {
    let mut warned = false;
    loop {
        match store.set_stop_reason(job_id, reason).await {
            Ok(_) => break,
            Err(error) => {
                if !warned {
                    eprintln!(
                        "warning: storing stop reason failed: {}",
                        render::sanitize(&error.to_string())
                    );
                    warned = true;
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
    stop.send_replace(true);
}
