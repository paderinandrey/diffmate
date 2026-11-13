use super::fixture::{cell, serve};
use super::publish::paths;
use super::queue::{self, open};
use crate::lock;

#[tokio::test]
async fn an_idle_tui_does_not_take_the_lock() {
    let (tmp, store) = open().await;
    queue::seed(&store, 1).await;
    let paths = paths(&tmp);
    let session = serve(&store, &paths).await;
    assert!(!session.running());
    assert!(lock::holder(&paths.lock()).await.is_none());
}

#[tokio::test]
async fn a_running_review_does_not_make_the_tui_the_runner() {
    let (tmp, store) = open().await;
    queue::seed(&store, 1).await;
    store
        .call(|conn| conn.execute("UPDATE jobs SET state = 'running' WHERE id = 1", []))
        .await
        .unwrap();
    let paths = paths(&tmp);
    let session = serve(&store, &paths).await;
    assert!(!session.running());
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE id = 1").await,
        "running"
    );
    assert!(lock::holder(&paths.lock()).await.is_none());
}
