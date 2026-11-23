use super::fixture::{cell, plant, serve};
use super::publish::paths;
use super::queue::{self, open};
use crate::lock;

#[tokio::test]
async fn an_idle_tui_does_not_take_the_lock() {
    let (tmp, store) = open().await;
    queue::seed(&store, 1).await;
    let paths = paths(&tmp);
    let _session = serve(&store, &paths).await;
    assert!(!paths.lock().exists());
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
    let _session = serve(&store, &paths).await;
    assert!(!paths.lock().exists());
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE id = 1").await,
        "running"
    );
    assert!(lock::holder(&paths.lock()).await.is_none());
}

#[tokio::test]
async fn a_cancelled_sqlite_call_keeps_the_lock_until_its_write_finishes() {
    let (tmp, store) = open().await;
    plant(&store, "pending", "queued").await;
    let path = paths(&tmp).lock();
    let lock = crate::lock::try_acquire(&path).await.unwrap().unwrap();
    let scoped = store.under_lock(lock);
    let (started, start) = tokio::sync::oneshot::channel();
    let (finish, gate) = std::sync::mpsc::channel();
    let task = tokio::spawn(async move {
        scoped
            .call(move |conn| {
                started.send(()).unwrap();
                gate.recv().unwrap();
                conn.execute("UPDATE jobs SET state = 'running' WHERE id = 2", [])
            })
            .await
    });
    start.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert!(crate::lock::try_acquire(&path).await.unwrap().is_none());
    finish.send(()).unwrap();
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE id = 2").await,
        "running"
    );
    serve(&store, &paths(&tmp)).await;
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "submitted"
    );
}
