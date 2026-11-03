use super::deadline::with_deadline;
use crate::cancel;
use std::cell::Cell;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::{Instant, sleep};

const TIMEOUT: Duration = Duration::from_secs(60);

#[tokio::test(start_paused = true)]
async fn work_finishing_a_second_early_keeps_its_output() {
    let (_tx, cancel) = watch::channel(false);
    let result = with_deadline(
        &cancel,
        TIMEOUT,
        || async { panic!("late timeout") },
        |_| async {
            sleep(TIMEOUT.checked_sub(Duration::from_secs(1)).unwrap()).await;
            42
        },
    )
    .await;
    assert_eq!(result, 42);
}

#[tokio::test(start_paused = true)]
async fn the_timeout_is_persisted_before_cancellation_at_the_exact_deadline() {
    let (_tx, cancel) = watch::channel(false);
    let persisted = &Cell::new(false);
    let started = Instant::now();
    with_deadline(
        &cancel,
        TIMEOUT,
        || async {
            persisted.set(true);
        },
        |mut merged| async move {
            cancel::cancelled(&mut merged).await;
            assert!(persisted.get());
            assert_eq!(started.elapsed(), TIMEOUT);
        },
    )
    .await;
}

#[tokio::test(start_paused = true)]
async fn an_outer_stop_disables_the_deadline_during_graceful_cleanup() {
    let (stop, cancel) = watch::channel(false);
    let started = Instant::now();
    let signal = async {
        sleep(TIMEOUT / 2).await;
        stop.send_replace(true);
    };
    let work = with_deadline(
        &cancel,
        TIMEOUT,
        || async { panic!("second stop") },
        |mut merged| async move {
            cancel::cancelled(&mut merged).await;
            assert_eq!(started.elapsed(), TIMEOUT / 2);
            sleep(TIMEOUT).await;
        },
    );
    tokio::join!(signal, work);
    assert_eq!(started.elapsed(), TIMEOUT + TIMEOUT / 2);
}

#[tokio::test(start_paused = true)]
async fn a_ready_outer_stop_wins_over_a_ready_deadline() {
    let (stop, cancel) = watch::channel(false);
    let signal = async {
        sleep(TIMEOUT).await;
        stop.send_replace(true);
    };
    let work = with_deadline(
        &cancel,
        TIMEOUT,
        || async { panic!("overwrote signal") },
        |mut merged| async move {
            cancel::cancelled(&mut merged).await;
        },
    );
    tokio::join!(biased; signal, work);
}
