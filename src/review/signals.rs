use crate::cancel::StopReason;
use crate::error::Error;
use crate::render;
use crate::store::{JobId, Store};
use std::future::Future;
use std::pin::pin;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::watch;

pub(super) async fn until_done<F: Future>(
    stop: watch::Sender<bool>,
    job: F,
    store: &Store,
    job_id: JobId,
    interrupt: StopReason,
) -> Result<F::Output, Error> {
    let mut interrupts = signal(SignalKind::interrupt()).map_err(Error::io("listen for SIGINT"))?;
    let mut terminate = signal(SignalKind::terminate()).map_err(Error::io("listen for SIGTERM"))?;
    let mut job = pin!(job);
    loop {
        let reason = tokio::select! {
            output = &mut job => return Ok(output),
            _ = interrupts.recv() => interrupt,
            _ = terminate.recv() => StopReason::Shutdown,
        };
        if let Err(error) = store.set_stop_reason(job_id, reason).await {
            eprintln!(
                "warning: storing stop reason failed: {}",
                render::sanitize(&error.to_string())
            );
        }
        if !*stop.borrow() {
            stop.send_replace(true);
        }
    }
}
