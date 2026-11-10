use std::sync::Arc;

use super::Script;
use super::support::{self, batch, cell, open, process};
use crate::crash::Hold;
use crate::paths::Paths;
use crate::store::Store;

#[tokio::test]
async fn crash_after_sending_is_unknown() {
    let _hold = Hold::point("publish-after-sending").await;
    let (tmp, store) = open().await;
    batch(&store, support::KEPT).await;
    let script = Arc::new(Script::new(store.clone()));
    let job = store
        .confirm_publication(1, "COMMENT", "abc123", &process())
        .await
        .unwrap();
    store.start_job(job).await.unwrap();
    let running = store.clone();
    let gh = Arc::clone(&script);
    let hung = tokio::spawn(async move {
        let _ = crate::publish::execute_publish(&running, gh.as_ref(), job, 2).await;
    });
    wait_sending(&store).await;
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let second = Store::open(&paths).await.unwrap();
    recover(&paths, &second).await;
    assert_eq!(publication(&second).await, "unknown");
    assert_eq!(publish_job(&second).await, "failed");
    assert_eq!(reason(&second).await, "outcome unknown");
    assert_eq!(reconcile_jobs(&second, "queued").await, 1);
    assert_eq!(script.posts(), 0);
    recover(&paths, &second).await;
    assert_eq!(reconcile_jobs(&second, "queued").await, 1);
    crate::tui::serve(&second, script.as_ref(), 2)
        .await
        .unwrap();
    hung.abort();
    assert_eq!(script.posts(), 0);
    assert_eq!(publication(&second).await, "unknown");
    assert_eq!(publish_job(&second).await, "failed");
}

async fn recover(paths: &Paths, store: &Store) {
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    crate::recovery::run(paths, store, &git, &crate::config::Settings::default())
        .await
        .unwrap();
}

async fn wait_sending(store: &Store) {
    for _ in 0..1_000 {
        if publication(store).await == "sending" {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("publication did not reach sending");
}

async fn publication(store: &Store) -> String {
    cell(store, "SELECT state FROM publications").await
}

async fn publish_job(store: &Store) -> String {
    cell(store, "SELECT state FROM jobs WHERE kind = 'publish'").await
}

async fn reason(store: &Store) -> String {
    cell(
        store,
        "SELECT failure_reason FROM jobs WHERE kind = 'publish'",
    )
    .await
}

async fn reconcile_jobs(store: &Store, state: &str) -> i64 {
    let sql = format!("SELECT count(*) FROM jobs WHERE kind = 'reconcile' AND state = '{state}'");
    cell(store, &sql).await
}
