use crate::paths::Paths;
use crate::store::Store;
use tempfile::TempDir;

pub async fn store() -> (TempDir, Paths, Store) {
    let tmp = TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let store = Store::open(&paths).await.unwrap();
    store
        .call(|conn| {
            conn.execute_batch(
        "INSERT INTO targets (id, owner, repo, number, created_at) VALUES (1, 'o', 'r', 1, 0);
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid,
            enqueued_by_started_at, created_at, updated_at)
         VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);"
    )
        })
        .await
        .unwrap();
    (tmp, paths, store)
}

pub async fn failed_run(store: &Store) {
    store
        .call(|conn| {
            conn.execute_batch(
                "INSERT INTO runs (target_id, job_id, state, failure_kind, interrupted_cause,
            head_sha, merge_base_sha, base_tip_sha, base_ref, created_at)
         VALUES (1, 1, 'failed', 'interrupted', 'crash', 'head', 'base', 'base', 'main', 0);",
            )
        })
        .await
        .unwrap();
}
