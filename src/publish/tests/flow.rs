use super::execute_publish;
use super::script::Script;
use super::support::{batch, open, process};
use crate::gh::ReviewRequest;

pub(super) const BASE: &str = "\
INSERT INTO targets (id, owner, repo, number, tracking, created_at) VALUES (1, 'o', 'r', 1, 'tracked', 1);
INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', 'abc123', 'base', 'base', 'main', 0);";

pub(super) async fn publish(
    sql: &str,
    event: &str,
) -> (tempfile::TempDir, crate::store::Store, ReviewRequest) {
    let (tmp, store) = open().await;
    batch(&store, sql).await;
    let script = Script::new(store.clone());
    let job = store
        .confirm_publication(1, event, "other", &process())
        .await
        .unwrap();
    execute_publish(&store, &script, job, 2).await.unwrap();
    let review = script.seen().into_iter().next().unwrap();
    (tmp, store, review)
}

pub(super) fn marker(id: &str) -> String {
    format!("<!-- diffmate:publication={id} -->")
}
