use crate::cancel::StopReason;
use crate::crash::crash_point;
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
            Ok(true) => break,
            Ok(false) => match still_waiting(store, job_id).await {
                Ok(true) => {}
                Ok(false) => break,
                Err(error) => note(&mut warned, &error),
            },
            Err(error) => note(&mut warned, &error),
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    crash_point!("stop-stored");
    stop.send_replace(true);
}

fn note(warned: &mut bool, error: &Error) {
    if *warned {
        return;
    }
    eprintln!(
        "warning: storing stop reason failed: {}",
        render::sanitize(&error.to_string())
    );
    *warned = true;
}

async fn still_waiting(store: &Store, job_id: JobId) -> Result<bool, Error> {
    store
        .call(move |conn| {
            let (state, reason, finished): (String, Option<String>, i64) = conn.query_row(
                "SELECT state, stop_reason,
                 (SELECT count(*) FROM runs WHERE job_id = ?1 AND finished_at IS NOT NULL)
                 FROM jobs WHERE id = ?1",
                [job_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
            let open =
                reason.is_none() && (state == "running" || (state == "queued" && finished == 0));
            Ok(open)
        })
        .await
}
