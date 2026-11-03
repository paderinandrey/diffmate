use crate::paths::Paths;
use crate::store::{Store, now_ms};

#[tokio::test]
async fn queue_selects_only_due_reviews_in_id_order() {
    let tmp = tempfile::TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let store = Store::open(&paths).await.unwrap();
    let now = now_ms();
    store
        .call(move |conn| {
            conn.execute(
                "INSERT INTO targets (owner, repo, number, created_at) VALUES ('o', 'r', 7, 0)",
                [],
            )?;
            for (id, kind, state, due) in [
                (1, "review", "queued", Some(now + 60_000)),
                (2, "review", "queued", None),
                (3, "review", "queued", Some(now - 1)),
                (4, "publish", "queued", Some(now + 1)),
                (5, "review", "failed", Some(now + 2)),
            ] {
                conn.execute(
                    "INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid,
                enqueued_by_started_at, created_at, updated_at, next_attempt_at)
                VALUES (?1, ?2, 1, ?3, 0, 0, 0, 0, ?4)",
                    rusqlite::params![id, kind, state, due],
                )?;
            }
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(
        store.earliest_next_attempt().await.unwrap(),
        Some(now + 60_000)
    );
    let next = store.next_runnable_job(now).await.unwrap().unwrap();
    assert_eq!(next.job.job_id, 2);
    assert_eq!(next.job.target_id, 1);
    assert_eq!(next.pr.label(), "o/r#7");
    store.start_job(2).await.unwrap();
    assert_eq!(
        store
            .next_runnable_job(now)
            .await
            .unwrap()
            .unwrap()
            .job
            .job_id,
        3
    );
    store.start_job(3).await.unwrap();
    assert!(store.next_runnable_job(now).await.unwrap().is_none());
    assert!(
        store
            .next_runnable_job(now + 59_999)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .next_runnable_job(now + 60_000)
            .await
            .unwrap()
            .unwrap()
            .job
            .job_id,
        1
    );
    store.start_job(1).await.unwrap();
    assert_eq!(store.earliest_next_attempt().await.unwrap(), None);
    assert!(
        store
            .next_runnable_job(now + 90_000)
            .await
            .unwrap()
            .is_none()
    );
}
