use super::flow::{BASE, marker, publish};
use super::support::{batch, cell, open, process};
use crate::store::publish::ConfirmError;

#[tokio::test]
async fn approve_and_request_changes_use_the_run_head() {
    for event in ["APPROVE", "REQUEST_CHANGES"] {
        let sql = format!(
            "{BASE}
INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);
INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('f1', 1, 1, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'Title', 'Body text', 'kept');"
        );
        let (_tmp, _store, review) = publish(&sql, event).await;
        assert_eq!(
            (
                review.event.as_str(),
                review.commit_id.as_str(),
                review.comments.len()
            ),
            (event, "abc123", 1)
        );
    }
}

#[tokio::test]
async fn pre_checks_create_nothing() {
    let (_tmp, store) = open().await;
    let sql = format!(
        "{BASE}
INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);
INSERT INTO findings (id, run_id, draft_id, ordinal, severity, title, body, decision) VALUES ('loose', 1, 1, 1, 'low', 'Loose', 'body', 'undecided');
INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('pinned', 1, 1, 2, 'high', 'src/a.rs', 4, 'RIGHT', 'Pinned', 'body', 'undecided');
INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('kept', 1, 1, 3, 'low', 'src/a.rs', 8, 'RIGHT', 'Kept', 'body', 'kept');"
    );
    batch(&store, &sql).await;
    let error = store
        .confirm_publication(1, "COMMENT", "other", &process())
        .await
        .unwrap_err();
    let ConfirmError::Undecided(ids) = error else {
        panic!("expected undecided, got {error}");
    };
    assert_eq!(ids, ["pinned", "loose"]);
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM publications").await,
        0
    );
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM jobs WHERE kind = 'publish'").await,
        0
    );
}

#[tokio::test]
async fn empty_draft_confirms() {
    let sql = format!(
        "{BASE}\nINSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);"
    );
    let (_tmp, store, review) = publish(&sql, "COMMENT").await;
    let id = cell::<String>(&store, "SELECT id FROM publications").await;
    assert_eq!(review.body, marker(&id));
    assert!(review.comments.is_empty(), "{:?}", review.comments);
}
