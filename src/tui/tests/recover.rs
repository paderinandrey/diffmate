use super::fixture::{cell, plant, serve};
use super::publish::paths;
use super::queue::{self, open};

#[tokio::test]
async fn a_running_pending_publish_resumes_on_startup() {
    let (tmp, store) = open().await;
    plant(&store, "pending", "running").await;
    let session = serve(&store, &paths(&tmp)).await;
    assert!(session.running());
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
