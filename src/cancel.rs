#[cfg(test)]
mod tests;

use std::future::Future;
use tokio::sync::watch;

pub(crate) type CancelRx = watch::Receiver<bool>;

pub(crate) async fn cancelled(cancel: &mut CancelRx) {
    let closed = cancel.wait_for(|stop| *stop).await.is_err();
    if closed {
        std::future::pending::<()>().await;
    }
}

pub(crate) async fn raced<F: Future>(cancel: &mut CancelRx, work: F) -> Option<F::Output> {
    tokio::select! {
        biased;
        () = cancelled(cancel) => None,
        output = work => Some(output),
    }
}
