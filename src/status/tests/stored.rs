use super::{LiveHead, fixture, freshness};
use crate::store::Store;

#[tokio::test]
async fn a_queued_crash_retry_appends_its_persisted_budget_to_latest() {
    let (_tmp, paths, store) = fixture::store().await;
    fixture::failed_run(&store).await;
    store
        .call(|conn| {
            conn.execute_batch(
                "UPDATE jobs SET state = 'queued', origin = 'automatic', retry_count = 1,
            max_retries = 2, next_attempt_at = 70000;",
            )
        })
        .await
        .unwrap();
    drop(store);
    let reader = Store::open_read_only(&paths).await.unwrap().unwrap();
    let rows = reader.target_statuses().await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        super::super::latest(&rows[0], 10_000),
        "failed (interrupted: crash); retry 1 of 2 in 1m"
    );
    assert!(!paths.lock().exists());
}

#[tokio::test]
async fn fallback_uses_the_stored_head_and_latest_successful_run() {
    let (_tmp, paths, store) = fixture::store().await;
    store
        .call(|conn| {
            conn.execute_batch(
                "UPDATE targets SET head_sha = 'new-head', head_checked_at = 20000;
         INSERT INTO runs (target_id, job_id, state, result,
            head_sha, merge_base_sha, base_tip_sha, base_ref, created_at)
         VALUES (1, 1, 'succeeded', 'no_findings', 'old-head', 'base', 'base', 'main', 0),
                (1, 1, 'succeeded', 'no_findings', 'draft-head', 'base', 'base', 'main', 1);",
            )
        })
        .await
        .unwrap();
    fixture::failed_run(&store).await;
    drop(store);
    let reader = Store::open_read_only(&paths).await.unwrap().unwrap();
    let rows = reader.target_statuses().await.unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.latest, "failed (interrupted: crash)");
    assert_eq!(row.draft_head.as_deref(), Some("draft-head"));
    assert_eq!(row.known_head, Some(("new-head".into(), 20_000)));
    let (head, checked) = row.known_head.as_ref().unwrap();
    assert_eq!(
        freshness::describe(
            row.draft_head.as_deref().unwrap(),
            &LiveHead::Failed,
            Some((head, *checked)),
            200_000
        ),
        "stale: head new-head, checked 3m ago (live check failed)"
    );
    assert!(!paths.lock().exists());
}
