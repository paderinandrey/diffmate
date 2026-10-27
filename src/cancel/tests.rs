use super::raced;
use std::future::pending;
use tokio::sync::watch;

#[tokio::test]
async fn work_completes_while_nothing_is_cancelled() {
    let (_stop, mut cancel) = watch::channel(false);

    assert_eq!(raced(&mut cancel, async { 7 }).await, Some(7));
}

#[tokio::test]
async fn an_already_cancelled_channel_never_polls_the_work() {
    let (stop, mut cancel) = watch::channel(false);
    stop.send_replace(true);

    assert_eq!(raced(&mut cancel, async { 7 }).await, None);
}

#[tokio::test]
async fn cancelling_drops_work_that_is_still_running() {
    let (stop, mut cancel) = watch::channel(false);
    let work = raced(&mut cancel, pending::<()>());
    let trigger = async { stop.send_replace(true) };

    let (output, _) = tokio::join!(work, trigger);

    assert_eq!(output, None);
}

#[tokio::test]
async fn a_dropped_sender_is_not_a_cancellation() {
    let (stop, mut cancel) = watch::channel(false);
    drop(stop);

    assert_eq!(raced(&mut cancel, async { 7 }).await, Some(7));
}
