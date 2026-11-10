use super::execute_publish;
use super::script::Script;
use super::support::{KEPT, batch, cell, open, process};
use crate::store::publish::ConfirmError;

#[tokio::test]
async fn snapshot_ignores_later_edits() {
    let (_tmp, store) = open().await;
    batch(&store, KEPT).await;
    let script = Script::new(store.clone());
    let job = store
        .confirm_publication(1, "COMMENT", "other", &process())
        .await
        .unwrap();
    let before = cell::<String>(&store, "SELECT snapshot FROM publications").await;
    store
        .save_finding_text("f1", "Changed", "Changed body")
        .await
        .unwrap();
    let after = cell::<String>(&store, "SELECT snapshot FROM publications").await;
    assert_eq!(before, after);
    execute_publish(&store, &script, job, 2).await.unwrap();
    let review = script.seen().into_iter().next().unwrap();
    let comment = &review.comments[0];
    assert!(comment.body.contains("Title"), "{}", comment.body);
    assert!(comment.body.contains("Body text"), "{}", comment.body);
    assert!(!comment.body.contains("Changed"), "{}", comment.body);
}

#[tokio::test]
async fn duplicate_publish_job_is_blocked() {
    let (_tmp, store) = open().await;
    batch(&store, KEPT).await;
    let trigger = "CREATE TEMP TRIGGER steal_publish BEFORE INSERT ON publications BEGIN INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, origin, publication_id) VALUES ('publish', NEW.target_id, 'queued', 1, 0, 0, 0, 'manual', NEW.id); END;";
    batch(&store, trigger).await;
    let error = store
        .confirm_publication(1, "COMMENT", "other", &process())
        .await
        .unwrap_err();
    assert!(matches!(&error, ConfirmError::Blocked), "{error}");
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM publications").await,
        0
    );
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM jobs WHERE kind = 'publish'").await,
        0
    );
}
