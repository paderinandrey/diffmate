use super::*;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tempfile::TempDir;

fn hold(path: &Path) -> File {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(path)
        .unwrap();
    file.try_lock().unwrap();
    file
}

#[tokio::test(start_paused = true)]
async fn acquire_waits_while_the_lock_is_held_and_notifies_once() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("runner.lock");
    let holder = hold(&path);
    let waits = Arc::new(AtomicU32::new(0));
    let counter = Arc::clone(&waits);
    let waiter = tokio::spawn(async move {
        let notify = || {
            counter.fetch_add(1, Ordering::SeqCst);
        };
        acquire(&path, LOCK_POLL, notify).await
    });

    while waits.load(Ordering::SeqCst) == 0 {
        tokio::task::yield_now().await;
    }
    tokio::time::sleep(Duration::from_secs(5)).await;

    assert!(!waiter.is_finished());
    drop(holder);
    assert!(waiter.await.unwrap().is_ok());
    assert_eq!(waits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn try_acquire_reports_the_holder_and_creates_a_private_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("runner.lock");

    let first = try_acquire(&path).await.unwrap();
    let second = try_acquire(&path).await.unwrap();
    assert!(first.is_some() && second.is_none());
    drop(first);
    let third = try_acquire(&path).await.unwrap();

    assert!(third.is_some());
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}
