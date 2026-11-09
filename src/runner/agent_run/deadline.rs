use crate::cancel::{self, CancelRx, StopReason};
use crate::store::{RunId, Store};
use std::future::Future;
use std::pin::pin;
use std::time::Duration;
use tokio::sync::watch;

pub(super) async fn with_deadline<T, W, F, O, G>(
    outer: &CancelRx,
    timeout: Duration,
    on_timeout: O,
    work: W,
) -> T
where
    W: FnOnce(CancelRx) -> F,
    F: Future<Output = T>,
    O: FnOnce() -> G,
    G: Future<Output = ()>,
{
    let (stop, merged) = watch::channel(*outer.borrow());
    let mut outer = outer.clone();
    let mut work = pin!(work(merged));
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);
    tokio::select! {
        biased;
        output = &mut work => return output,
        () = cancel::cancelled(&mut outer) => {}
        () = &mut deadline => on_timeout().await,
    }
    stop.send_replace(true);
    work.await
}

pub(super) async fn persist_timeout(store: &Store, run: RunId) -> Option<String> {
    let mut warning = None;
    loop {
        match store
            .set_stop_reason_for_run(run, StopReason::Timeout)
            .await
        {
            Ok(_) => return warning,
            Err(error) => {
                if warning.is_none() {
                    let message = format!("storing timeout reason failed: {error}");
                    eprintln!("warning: {}", crate::render::sanitize(&message));
                    warning = Some(message);
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
