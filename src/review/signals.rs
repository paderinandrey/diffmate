use crate::error::Error;
use std::future::Future;
use std::pin::pin;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::watch;

pub(super) async fn until_done<F: Future>(
    stop: watch::Sender<bool>,
    job: F,
) -> Result<F::Output, Error> {
    let mut interrupt = signal(SignalKind::interrupt()).map_err(Error::io("listen for SIGINT"))?;
    let mut terminate = signal(SignalKind::terminate()).map_err(Error::io("listen for SIGTERM"))?;
    let mut job = pin!(job);
    loop {
        tokio::select! {
            output = &mut job => return Ok(output),
            _ = interrupt.recv() => { stop.send_replace(true); }
            _ = terminate.recv() => { stop.send_replace(true); }
        }
    }
}
