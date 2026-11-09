use super::*;
use crate::app::ProcessInfo;
use crate::paths::Paths;
use crate::pr_url::parse;
use crate::store::{jobs::EnqueueRequest, records::JobOrigin};
use std::time::Duration;

async fn fixture() -> (tempfile::TempDir, Store, JobId) {
    let temp = tempfile::TempDir::new().unwrap();
    let paths = Paths::new(temp.path().join("state"), temp.path().join("cache"));
    let store = Store::open(&paths).await.unwrap();
    let job = store
        .enqueue_review(
            &parse("https://github.com/o/r/pull/1").unwrap(),
            &ProcessInfo {
                pid: 1,
                started_at_ms: 0,
            },
            EnqueueRequest {
                origin: JobOrigin::Manual,
                force: false,
            },
        )
        .await
        .unwrap();
    (temp, store, job.job.job_id)
}

#[tokio::test(start_paused = true)]
async fn a_failed_signal_write_does_not_cancel_until_persisted() {
    let (_temp, store, job) = fixture().await;
    store.start_job(job).await.unwrap();
    store.call(|conn| conn.execute_batch("CREATE TRIGGER reject_signal BEFORE UPDATE OF stop_reason ON jobs BEGIN SELECT RAISE(ABORT, 'injected signal write failure'); END;")).await.unwrap();
    let (stop, cancel) = watch::channel(false);
    let repair = async {
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(
            !*cancel.borrow(),
            "signal must not cancel without a durable cause"
        );
        assert_eq!(store.stop_reason(job).await.unwrap(), None);
        store
            .call(|conn| conn.execute_batch("DROP TRIGGER reject_signal"))
            .await
            .unwrap();
    };
    tokio::join!(
        request_stop(&stop, &store, job, StopReason::Shutdown),
        repair
    );
    assert!(*cancel.borrow());
    assert_eq!(
        store.stop_reason(job).await.unwrap(),
        Some(StopReason::Shutdown)
    );
}

#[tokio::test(start_paused = true)]
async fn an_existing_timeout_remains_the_first_cause() {
    let (_temp, store, job) = fixture().await;
    store.start_job(job).await.unwrap();
    assert!(
        store
            .set_stop_reason(job, StopReason::Timeout)
            .await
            .unwrap()
    );
    let (stop, cancel) = watch::channel(false);
    request_stop(&stop, &store, job, StopReason::Shutdown).await;
    assert!(*cancel.borrow());
    assert_eq!(
        store.stop_reason(job).await.unwrap(),
        Some(StopReason::Timeout)
    );
}

#[tokio::test(start_paused = true)]
async fn a_signal_before_start_waits_until_the_attempt_is_running() {
    let (_temp, store, job) = fixture().await;
    let (stop, cancel) = watch::channel(false);
    let start = async {
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(!*cancel.borrow());
        assert_eq!(store.stop_reason(job).await.unwrap(), None);
        store.start_job(job).await.unwrap();
    };
    tokio::join!(
        request_stop(&stop, &store, job, StopReason::Shutdown),
        start
    );
    assert!(*cancel.borrow());
    assert_eq!(
        store.stop_reason(job).await.unwrap(),
        Some(StopReason::Shutdown)
    );
}
