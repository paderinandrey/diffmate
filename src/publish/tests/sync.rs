mod harness;
mod link;

use harness::{Remote, cell, seeded};

#[tokio::test(start_paused = true)]
async fn sync_failure_stays_submitted() {
    let (_tmp, store, job) = seeded().await;
    let gh = Remote::comments_fail();
    crate::publish::execute_sync(&store, &gh, job)
        .await
        .unwrap();
    assert_eq!(
        cell::<String>(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT thread_status FROM findings WHERE id = 'f1'").await,
        "unknown"
    );
    assert_eq!(
        cell::<String>(
            &store,
            "SELECT COALESCE(github_comment_id, '') FROM findings WHERE id = 'f1'"
        )
        .await,
        ""
    );
    assert_eq!(gh.posts(), 0);
    let state = format!("SELECT state FROM jobs WHERE id = {job}");
    assert_eq!(cell::<String>(&store, &state).await, "queued");
    let gap =
        format!("SELECT COALESCE(next_attempt_at, 0) - updated_at FROM jobs WHERE id = {job}");
    assert_eq!(cell::<i64>(&store, &gap).await, 60_000);
}

#[tokio::test(start_paused = true)]
async fn sync_does_not_enqueue_twice() {
    let (_tmp, store) = published().await;
    let sql = "SELECT count(*) FROM jobs WHERE kind = 'sync' AND state IN ('queued', 'running')";
    assert_eq!(cell::<i64>(&store, sql).await, 1);
    let id = cell::<String>(&store, "SELECT id FROM publications").await;
    store.enqueue_sync(&id).await.unwrap();
    assert_eq!(cell::<i64>(&store, sql).await, 1);
}

async fn published() -> (tempfile::TempDir, crate::store::Store) {
    let (tmp, store) = super::support::open().await;
    super::support::batch(&store, super::support::KEPT).await;
    let script = super::script::Script::new(store.clone());
    let process = super::support::process();
    let job = store
        .confirm_publication(1, "COMMENT", "abc123", &process)
        .await
        .unwrap();
    crate::publish::execute_publish(&store, &script, job, 2)
        .await
        .unwrap();
    (tmp, store)
}
