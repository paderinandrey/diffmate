#[cfg(test)]
mod tests;

use std::future::Future;
use tokio::sync::watch;

pub(crate) type CancelRx = watch::Receiver<bool>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StopReason {
    Timeout,
    Shutdown,
    UserCancel,
}

impl StopReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Shutdown => "shutdown",
            Self::UserCancel => "user_cancel",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "timeout" => Some(Self::Timeout),
            "shutdown" => Some(Self::Shutdown),
            "user_cancel" => Some(Self::UserCancel),
            _ => None,
        }
    }
}

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
