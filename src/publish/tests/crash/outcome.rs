use std::time::Duration;

use super::harness::{self, ready};
use crate::store::publish::ConfirmError;

#[tokio::test(start_paused = true)]
async fn timeout_is_unknown() {
    let (_tmp, store, remote, job) = ready().await;
    remote.hang();
    let ran = tokio::time::timeout(
        Duration::from_secs(31),
        crate::publish::execute_publish(&store, remote.as_ref(), job, 0),
    )
    .await;
    assert!(ran.is_ok(), "create_review was not bounded");
    ran.unwrap().unwrap();
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "unknown"
    );
    assert_eq!(
        harness::text(
            &store,
            "SELECT failure_reason FROM jobs WHERE kind = 'publish'"
        )
        .await,
        "timed out after 30s"
    );
    assert_eq!(remote.posts(), 1);
    let err = store
        .confirm_publication(
            store.displayed_draft(1).await,
            "COMMENT",
            "abc123",
            &crate::publish::tests::support::process(),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, ConfirmError::Blocked));
    assert_eq!(remote.posts(), 1);
}

#[tokio::test]
async fn client_error_rejects() {
    let (_tmp, store, remote, job) = ready().await;
    remote.fail(422);
    crate::publish::execute_publish(&store, remote.as_ref(), job, 2)
        .await
        .unwrap();
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "rejected"
    );
    assert_eq!(
        harness::text(&store, "SELECT state FROM jobs WHERE kind = 'publish'").await,
        "failed"
    );
    assert_eq!(
        harness::text(&store, "SELECT reason FROM publications").await,
        "HTTP 422: validation failed"
    );
    assert_eq!(remote.posts(), 1);
}

#[tokio::test]
async fn server_error_is_unknown() {
    let (_tmp, store, remote, job) = ready().await;
    remote.fail(500);
    crate::publish::execute_publish(&store, remote.as_ref(), job, 2)
        .await
        .unwrap();
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "unknown"
    );
    assert_eq!(
        harness::text(
            &store,
            "SELECT failure_reason FROM jobs WHERE kind = 'publish'"
        )
        .await,
        "HTTP 500: internal server error"
    );
    assert_eq!(remote.posts(), 1);
}

#[tokio::test]
async fn a_decode_failure_stays_unknown() {
    let (_tmp, store, remote, job) = ready().await;
    remote.decode();
    crate::publish::execute_publish(&store, remote.as_ref(), job, 0)
        .await
        .unwrap();
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "unknown"
    );
    assert_eq!(
        harness::text(&store, "SELECT reason FROM publications").await,
        "unreadable review response"
    );
    assert_eq!(remote.posts(), 1);
    let err = store
        .confirm_publication(
            store.displayed_draft(1).await,
            "COMMENT",
            "abc123",
            &crate::publish::tests::support::process(),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, ConfirmError::Blocked));
}
