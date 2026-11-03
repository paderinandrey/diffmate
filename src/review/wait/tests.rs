use super::*;
use crate::app::ProcessInfo;
use crate::paths::Paths;
use crate::pr_url::parse;
use crate::store::{jobs::EnqueueRequest, records::JobOrigin};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn a_delayed_job_waits_without_holding_the_lock_then_becomes_runnable() {
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
                origin: JobOrigin::Automatic,
                force: false,
            },
        )
        .await
        .unwrap()
        .job;
    store
        .call(|conn| {
            conn.execute("UPDATE jobs SET next_attempt_at = 4102444800000", [])
                .map(drop)
        })
        .await
        .unwrap();
    assert!(
        store
            .job_delayed(job.job_id, 4_102_444_799_999)
            .await
            .unwrap()
    );
    assert!(
        !store
            .job_delayed(job.job_id, 4_102_444_800_000)
            .await
            .unwrap()
    );
    let mut signals = Signals::new().unwrap();
    let path = paths.lock();
    let waiting = until_lock_or_end(&path, &store, job.job_id, "o/r#1", &mut signals);
    tokio::pin!(waiting);
    assert!(
        tokio::time::timeout(Duration::from_secs(2), &mut waiting)
            .await
            .is_err()
    );
    let lock = lock::try_acquire(&path).await.unwrap();
    assert!(lock.is_some());
    drop(lock);
    store
        .call(|conn| {
            conn.execute("UPDATE jobs SET next_attempt_at = 0", [])
                .map(drop)
        })
        .await
        .unwrap();
    assert!(matches!(waiting.await.unwrap(), Waited::Locked(_)));
}
