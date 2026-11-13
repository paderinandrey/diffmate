mod body;
mod confirm;
mod crash;
mod flow;
mod gates;
mod once;
mod script;
mod snap;
mod support;
mod sync;

use script::Script;
use support::{KEPT, batch, cell, open, process};

use super::execute_publish;

#[tokio::test]
async fn one_kept_finding_reaches_submitted() {
    let (_tmp, store) = open().await;
    batch(&store, KEPT).await;
    let script = Script::new(store.clone());
    let job = store
        .confirm_publication(1, "COMMENT", "other", &process())
        .await
        .unwrap();
    execute_publish(&store, &script, job, 2).await.unwrap();
    let seen = script.seen();
    assert_eq!(seen.len(), 1);
    let review = &seen[0];
    assert_eq!(review.commit_id, "abc123");
    assert_eq!(review.event, "COMMENT");
    assert_eq!(review.comments.len(), 1);
    let comment = &review.comments[0];
    assert_eq!(
        (comment.path.as_str(), comment.line, comment.side.as_str()),
        ("src/a.rs", 4, "RIGHT")
    );
    assert!(comment.body.contains("Title"), "{}", comment.body);
    assert!(comment.body.contains("Body text"), "{}", comment.body);
    assert!(
        comment.body.contains("<!-- diffmate:finding=f1 -->"),
        "{}",
        comment.body
    );
    let id = cell::<String>(&store, "SELECT id FROM publications").await;
    assert_eq!(id.len(), 32);
    assert!(id.bytes().all(|byte| byte.is_ascii_hexdigit()), "{id}");
    let marker = format!("<!-- diffmate:publication={id} -->");
    assert!(review.body.contains(&marker), "{}", review.body);
    assert_eq!(
        cell::<String>(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT github_review_id FROM publications").await,
        "9001"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT confirmed_sha FROM publications").await,
        "other"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT state FROM jobs WHERE kind = 'publish'").await,
        "succeeded"
    );
    assert_eq!(
        cell::<String>(&store, "SELECT origin FROM jobs WHERE kind = 'publish'").await,
        "manual"
    );
    assert_eq!(
        cell::<i64>(
            &store,
            "SELECT enqueued_by_pid FROM jobs WHERE kind = 'publish'"
        )
        .await,
        4
    );
}

#[tokio::test]
async fn confirm_rolls_back_when_the_job_insert_fails() {
    let (_tmp, store) = open().await;
    batch(&store, KEPT).await;
    let trigger = "CREATE TEMP TRIGGER fail_publish BEFORE INSERT ON jobs WHEN NEW.kind = 'publish' BEGIN SELECT RAISE(ROLLBACK, 'forced'); END;";
    batch(&store, trigger).await;
    let error = store
        .confirm_publication(1, "COMMENT", "other", &process())
        .await;
    assert!(error.is_err(), "{error:?}");
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM publications").await,
        0
    );
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM jobs WHERE kind = 'publish'").await,
        0
    );
}
