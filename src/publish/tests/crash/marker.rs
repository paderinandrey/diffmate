use super::harness::{self, PUBLICATION, unknown};
use crate::gh::ListedReview;

fn review(body: &str) -> ListedReview {
    ListedReview {
        id: "77".into(),
        author: "tester".into(),
        state: "COMMENTED".into(),
        body: body.into(),
    }
}

#[tokio::test]
async fn reconcile_marker_submits() {
    let (_tmp, store, remote, job) = unknown().await;
    let marker = format!("<!-- diffmate:publication={PUBLICATION} -->");
    remote.reviews(vec![review(&marker)]);
    crate::publish::execute_reconcile(&store, remote.as_ref(), job)
        .await
        .unwrap();
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    assert_eq!(
        harness::text(&store, "SELECT github_review_id FROM publications").await,
        "77"
    );
    assert_eq!(
        harness::text(&store, "SELECT state FROM jobs WHERE kind = 'reconcile'").await,
        "succeeded"
    );
    assert_eq!(remote.posts(), 0);
}

#[tokio::test]
async fn reconcile_missing_marker_stays_unknown() {
    let (_tmp, store, remote, job) = unknown().await;
    remote.reviews(vec![review("no marker")]);
    crate::publish::execute_reconcile(&store, remote.as_ref(), job)
        .await
        .unwrap();
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "unknown"
    );
    assert_ne!(
        harness::text(&store, "SELECT state FROM publications").await,
        "rejected"
    );
    assert_eq!(
        harness::text(&store, "SELECT state FROM jobs WHERE kind = 'reconcile'").await,
        "succeeded"
    );
    assert_eq!(remote.posts(), 0);
}

#[tokio::test]
async fn abandon_unknown_sets_confirmed_by_me() {
    let (_tmp, store, remote, _job) = unknown().await;
    store.abandon_unknown(PUBLICATION).await.unwrap();
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "abandoned"
    );
    assert_eq!(
        harness::text(&store, "SELECT reason FROM publications").await,
        "confirmed by me"
    );
    let again = store.abandon_unknown(PUBLICATION).await;
    assert!(again.is_err(), "{again:?}");
    assert_eq!(remote.posts(), 0);
}
