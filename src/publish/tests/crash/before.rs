use std::sync::Arc;

use super::harness::{self, ready};
use crate::crash::Hold;

#[tokio::test]
async fn crash_before_sending_posts_once() {
    let hold = Hold::point("publish-before-sending").await;
    let (tmp, store, remote, job) = ready().await;
    let gh = Arc::clone(&remote);
    let running = store.clone();
    let hung = tokio::spawn(async move {
        let _ = crate::publish::execute_publish(&running, gh.as_ref(), job, 0).await;
    });
    for _ in 0..1_000 {
        if crate::crash::parked("publish-before-sending") {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert!(crate::crash::parked("publish-before-sending"));
    harness::recover(&harness::paths(&tmp), &store).await;
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "pending"
    );
    assert_eq!(
        harness::text(&store, "SELECT state FROM jobs WHERE kind = 'publish'").await,
        "queued"
    );
    hung.abort();
    let _ = hung.await;
    drop(hold);
    store.start_job(job).await.unwrap();
    crate::publish::execute_publish(&store, remote.as_ref(), job, 0)
        .await
        .unwrap();
    assert_eq!(remote.posts(), 1);
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "submitted"
    );
}
