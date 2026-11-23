use super::fixture::{cell, plant, serve};
use super::publish::paths;
use super::queue::{self, open};

#[tokio::test]
async fn a_running_pending_publish_resumes_on_startup() {
    let (tmp, store) = open().await;
    plant(&store, "pending", "running").await;
    let paths = paths(&tmp);
    let _session = serve(&store, &paths).await;
    assert!(
        crate::lock::try_acquire(&paths.lock())
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    let succeeded = "SELECT count(*) FROM jobs WHERE kind = 'publish' AND state = 'succeeded'";
    assert_eq!(queue::count(&store, succeeded).await, 1);
}

#[tokio::test]
async fn a_running_send_becomes_unknown_on_startup() {
    let (tmp, store) = open().await;
    plant(&store, "sending", "running").await;
    let _session = serve(&store, &paths(&tmp)).await;
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "unknown"
    );
    assert_eq!(
        cell(&store, "SELECT failure_reason FROM jobs WHERE id = 2").await,
        "outcome unknown"
    );
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE id = 2").await,
        "failed"
    );
}

#[tokio::test]
async fn recovering_an_exhausted_automatic_review_notifies_once() {
    let (tmp, store) = open().await;
    plant(&store, "pending", "running").await;
    store
        .call(|conn| {
            conn.execute_batch(
                "INSERT INTO jobs (id, kind, target_id, state, origin, retry_count, max_retries,
         enqueued_by_pid, enqueued_by_started_at, created_at, updated_at)
         VALUES (3, 'review', 1, 'running', 'automatic', 2, 2, 1, 0, 0, 0)",
            )
        })
        .await
        .unwrap();
    let paths = paths(&tmp);
    let mut session = super::fixture::open_session(&store, &paths.lock()).await;
    let gh = super::fake::Fake::new(queue::HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = crate::config::Settings::default();
    let mode = super::fixture::mode(&gh, &git, &paths, &settings);
    let deliveries = std::cell::Cell::new(0);
    for _ in 0..2 {
        crate::tui::runner::drive_with(&mut session, &store, &mode, async |notice| {
            let crate::notify::Notice::GaveUp { label, reason } = notice else {
                panic!("unexpected ready notification");
            };
            assert_eq!(label, "o/r#1");
            assert!(
                reason.contains("interrupted: crash") && reason.contains("no retries left"),
                "{reason}"
            );
            deliveries.set(deliveries.get() + 1);
            Ok(())
        })
        .await
        .unwrap();
    }
    assert_eq!(deliveries.get(), 1);
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE id = 3").await,
        "failed"
    );
}
