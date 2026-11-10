use super::execute_publish;
use super::script::Script;
use super::support::{KEPT, batch, cell, open, process};
use crate::store::publish::ConfirmError;

#[tokio::test]
async fn second_confirm_creates_nothing() {
    for state in ["pending", "sending", "unknown"] {
        let (_tmp, store) = open().await;
        batch(&store, KEPT).await;
        store
            .confirm_publication(1, "COMMENT", "other", &process())
            .await
            .unwrap();
        if state != "pending" {
            batch(
                &store,
                &format!("UPDATE publications SET state = '{state}'"),
            )
            .await;
        }
        let error = store
            .confirm_publication(1, "COMMENT", "other", &process())
            .await
            .unwrap_err();
        assert!(matches!(&error, ConfirmError::Blocked), "{state} {error}");
        assert_eq!(
            cell::<i64>(&store, "SELECT count(*) FROM publications").await,
            1,
            "{state}"
        );
        assert_eq!(
            cell::<i64>(&store, "SELECT count(*) FROM jobs WHERE kind = 'publish'").await,
            1,
            "{state}"
        );
    }
}

#[tokio::test]
async fn submitted_draft_is_refused() {
    let (_tmp, store) = open().await;
    batch(&store, KEPT).await;
    let script = Script::new(store.clone());
    let job = store
        .confirm_publication(1, "COMMENT", "other", &process())
        .await
        .unwrap();
    execute_publish(&store, &script, job, 2).await.unwrap();
    store
        .save_finding_text("f1", "Changed", "Changed body")
        .await
        .unwrap();
    let error = store
        .confirm_publication(1, "COMMENT", "other", &process())
        .await
        .unwrap_err();
    assert!(matches!(&error, ConfirmError::DraftSubmitted), "{error}");
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM publications").await,
        1
    );
}

#[tokio::test]
async fn rejected_allows_another_confirm() {
    for state in ["rejected", "abandoned"] {
        let (_tmp, store) = open().await;
        batch(&store, KEPT).await;
        store
            .confirm_publication(1, "APPROVE", "other", &process())
            .await
            .unwrap();
        batch(
            &store,
            &format!("UPDATE publications SET state = '{state}'"),
        )
        .await;
        store
            .confirm_publication(1, "APPROVE", "other", &process())
            .await
            .unwrap();
        assert_eq!(
            cell::<i64>(&store, "SELECT count(*) FROM publications").await,
            2,
            "{state}"
        );
    }
}
