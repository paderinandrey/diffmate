use crate::app::ProcessInfo;
use crate::paths::Paths;
use crate::store::Store;

pub(super) const KEPT: &str = "\
INSERT INTO targets (id, owner, repo, number, tracking, created_at) VALUES (1, 'o', 'r', 1, 'tracked', 1);
INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', 'abc123', 'base', 'base', 'main', 0);
INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{}', 0);
INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('f1', 1, 1, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'Title', 'Body text', 'kept');";

pub(super) fn process() -> ProcessInfo {
    ProcessInfo {
        pid: 4,
        started_at_ms: 5,
    }
}

pub(super) async fn open() -> (tempfile::TempDir, Store) {
    let tmp = tempfile::TempDir::new().unwrap();
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    (tmp, Store::open(&paths).await.unwrap())
}

pub(super) async fn batch(store: &Store, sql: &str) {
    let sql = sql.to_string();
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
}

pub(super) async fn cell<T>(store: &Store, sql: &str) -> T
where
    T: rusqlite::types::FromSql + Send + 'static,
{
    let sql = sql.to_string();
    store
        .call(move |conn| conn.query_row(&sql, [], |row| row.get(0)))
        .await
        .unwrap()
}
