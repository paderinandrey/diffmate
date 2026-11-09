use super::*;
use crate::cancel::{self, StopReason};
use crate::runner::agent_run::deadline::{persist_timeout, with_deadline};
use std::cell::RefCell;

#[tokio::test(start_paused = true)]
async fn a_failed_stop_write_never_cancels_before_a_durable_retry() {
    let fixture = Fixture::new(JobOrigin::Manual).await;
    let run = fixture.seed_run().await;
    fixture
        .store
        .call(|conn| {
            conn.execute_batch(
                "CREATE TRIGGER reject_stop BEFORE UPDATE OF stop_reason ON jobs
         BEGIN SELECT RAISE(ABORT, 'injected write failure'); END;",
            )
        })
        .await
        .unwrap();
    let (_tx, outer) = watch::channel(false);
    let receiver = RefCell::new(None);
    let mut warning = None;
    let work = with_deadline(
        &outer,
        Duration::from_secs(60),
        || async {
            warning = persist_timeout(&fixture.store, run).await;
        },
        |mut merged| {
            receiver.replace(Some(merged.clone()));
            async move {
                cancel::cancelled(&mut merged).await;
            }
        },
    );
    let repair = async {
        tokio::time::sleep(Duration::from_secs(62)).await;
        assert!(!*receiver.borrow().as_ref().unwrap().borrow());
        assert_eq!(
            fixture.store.stop_reason(fixture.job.job_id).await.unwrap(),
            None
        );
        fixture
            .store
            .call(|conn| conn.execute_batch("DROP TRIGGER reject_stop"))
            .await
            .unwrap();
    };
    tokio::join!(work, repair);
    assert!(warning.unwrap().contains("injected write failure"));
    assert_eq!(
        fixture.store.stop_reason(fixture.job.job_id).await.unwrap(),
        Some(StopReason::Timeout)
    );
}

#[tokio::test(start_paused = true)]
async fn concurrent_timeout_and_signal_keep_exactly_the_first_written_reason() {
    let fixture = Fixture::new(JobOrigin::Manual).await;
    let run = fixture.seed_run().await;
    let (timeout, signal) = tokio::join!(
        fixture
            .store
            .set_stop_reason_for_run(run, StopReason::Timeout),
        fixture
            .store
            .set_stop_reason(fixture.job.job_id, StopReason::UserCancel),
    );
    let (timeout, signal) = (timeout.unwrap(), signal.unwrap());
    assert_ne!(timeout, signal);
    let expected = if timeout {
        StopReason::Timeout
    } else {
        StopReason::UserCancel
    };
    assert_eq!(
        fixture.store.stop_reason(fixture.job.job_id).await.unwrap(),
        Some(expected)
    );
}
