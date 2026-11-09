use crate::app::ProcessInfo;
use crate::cancel::StopReason;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::recovery::rules::JobNext;
use crate::store::jobs::EnqueueRequest;
use crate::store::records::JobOrigin;
use crate::store::{JobId, Store};

async fn queued() -> (tempfile::TempDir, Store, JobId) {
    let temp = tempfile::TempDir::new().unwrap();
    let paths = Paths::new(temp.path().join("state"), temp.path().join("cache"));
    let store = Store::open(&paths).await.unwrap();
    let job = store
        .enqueue_review(
            &PrRef {
                owner: "o".into(),
                repo: "r".into(),
                number: 1,
            },
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
        .unwrap();
    (temp, store, job.job.job_id)
}

#[tokio::test]
async fn a_late_stop_does_not_mark_the_requeued_attempt() {
    let (_temp, store, job) = queued().await;
    store.start_job(job).await.unwrap();
    assert!(
        store
            .set_stop_reason(job, StopReason::Timeout)
            .await
            .unwrap()
    );
    store
        .resolve_job(
            job,
            &JobNext::Requeue {
                retry_count: 1,
                max_retries: Some(2),
                next_attempt_at: Some(1),
            },
        )
        .await
        .unwrap();
    assert!(
        !store
            .set_stop_reason(job, StopReason::Shutdown)
            .await
            .unwrap()
    );
    assert!(
        !store
            .set_stop_reason(job, StopReason::UserCancel)
            .await
            .unwrap()
    );
    assert_eq!(store.stop_reason(job).await.unwrap(), None);
    let state = store
        .call(move |conn| {
            conn.query_row("SELECT state FROM jobs WHERE id = ?1", [job], |row| {
                row.get::<_, String>(0)
            })
        })
        .await
        .unwrap();
    assert_eq!(state, "queued");
}
