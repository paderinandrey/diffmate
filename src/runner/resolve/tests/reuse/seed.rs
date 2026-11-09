use crate::store::Store;

pub(super) async fn runs(store: &Store) {
    store.call(|conn| {
        conn.execute_batch(
            "INSERT INTO targets (id, owner, repo, number, created_at) VALUES (1, 'o', 'r', 1, 0);
             INSERT INTO jobs (id, target_id, kind, state, origin, enqueued_by_pid,
                 enqueued_by_started_at, created_at, updated_at)
             VALUES (1, 1, 'review', 'queued', 'automatic', 0, 0, 0, 0),
                    (2, 1, 'review', 'succeeded', 'manual', 0, 0, 0, 0),
                    (3, 1, 'review', 'succeeded', 'manual', 0, 0, 0, 0);
             INSERT INTO runs (id, target_id, job_id, state, result, failure_kind,
                 head_sha, merge_base_sha, base_tip_sha, base_ref, created_at)
             VALUES (1, 1, 1, 'failed', NULL, 'timeout', 'old-head', 'base', 'tip', 'main', 0),
                    (2, 1, 2, 'succeeded', 'no_findings', NULL, 'old-head', 'base', 'tip', 'main', 0),
                    (3, 1, 3, 'succeeded', 'no_findings', NULL, 'new-head', 'base', 'tip', 'main', 0);
             INSERT INTO drafts (run_id, original_output, created_at) VALUES (2, '{}', 0), (3, '{}', 0);"
        )
    }).await.unwrap();
}
