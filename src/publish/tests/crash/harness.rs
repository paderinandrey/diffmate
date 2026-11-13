use std::sync::Arc;

use super::super::support::{self, batch, cell, open, process};
use super::gh::Remote;
use crate::paths::Paths;
use crate::store::Store;

pub(super) const PUBLICATION: &str = "0123456789abcdef0123456789abcdef";

pub(super) async fn ready() -> (tempfile::TempDir, Store, Arc<Remote>, i64) {
    let (tmp, store) = open().await;
    batch(&store, support::KEPT).await;
    let remote = Arc::new(Remote::new(store.clone()));
    let job = store
        .confirm_publication(1, "COMMENT", "abc123", &process())
        .await
        .unwrap();
    store.start_job(job).await.unwrap();
    (tmp, store, remote, job)
}

pub(super) async fn planted(state: &str) -> (tempfile::TempDir, Store, Arc<Remote>) {
    let (tmp, store) = open().await;
    batch(&store, support::KEPT).await;
    let sql = format!(
        "INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, created_at, updated_at) VALUES ('{PUBLICATION}', 1, 1, 1, '{state}', 'COMMENT', 'abc123', '{{}}', 1, 1);
         INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, origin, publication_id, failure_reason) VALUES ('publish', 1, 'failed', 1, 0, 2, 2, 'manual', '{PUBLICATION}', 'outcome unknown');"
    );
    batch(&store, &sql).await;
    let remote = Arc::new(Remote::new(store.clone()));
    (tmp, store, remote)
}

pub(super) async fn unknown() -> (tempfile::TempDir, Store, Arc<Remote>, i64) {
    let (tmp, store) = open().await;
    batch(&store, support::KEPT).await;
    let sql = format!(
        "INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, created_at, updated_at) VALUES ('{PUBLICATION}', 1, 1, 1, 'unknown', 'COMMENT', 'abc123', '{{}}', 1, 1);
         INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, origin, publication_id) VALUES ('reconcile', 1, 'queued', 1, 0, 2, 2, 'manual', '{PUBLICATION}');"
    );
    batch(&store, &sql).await;
    let job = cell::<i64>(&store, "SELECT id FROM jobs WHERE kind = 'reconcile'").await;
    let remote = Arc::new(Remote::new(store.clone()));
    (tmp, store, remote, job)
}

pub(super) fn paths(tmp: &tempfile::TempDir) -> Paths {
    Paths::new(tmp.path().join("state"), tmp.path().join("cache"))
}

pub(super) async fn text(store: &Store, sql: &str) -> String {
    cell(store, sql).await
}

pub(super) async fn recover(paths: &Paths, store: &Store) {
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    crate::recovery::run(paths, store, &git, &crate::config::Settings::default())
        .await
        .unwrap();
}
