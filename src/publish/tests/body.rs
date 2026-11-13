use super::flow::{BASE, marker, publish};
use super::support::cell;

fn shaped(included: bool) -> String {
    let flag = i64::from(included);
    format!(
        "{BASE}
INSERT INTO drafts (id, run_id, original_output, limitation, limitation_included, future_improvements, created_at) VALUES (1, 1, '{{}}', 'missing CI', {flag}, '[\"ship it\"]', 0);
INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('drop', 1, 1, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'Dropped', 'drop body', 'dropped');
INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, title, body, decision) VALUES ('edit', 1, 1, 2, 'low', 'src/loose.rs', 'Edited', 'edited body', 'edited');"
    )
}

#[tokio::test]
async fn body_omits_dropped_and_excluded_limitation() {
    let (_tmp, store, review) = publish(&shaped(false), "COMMENT").await;
    let id = cell::<String>(&store, "SELECT id FROM publications").await;
    assert!(review.body.contains("Edited"), "{}", review.body);
    assert!(review.body.contains("edited body"), "{}", review.body);
    assert!(review.body.contains("src/loose.rs"), "{}", review.body);
    assert!(
        !review.body.contains("Dropped") && !review.body.contains("drop body"),
        "{}",
        review.body
    );
    assert!(!review.body.contains("Limitation:"), "{}", review.body);
    assert!(
        review.body.contains("Future improvements") && review.body.contains("ship it"),
        "{}",
        review.body
    );
    assert!(review.comments.is_empty(), "{:?}", review.comments);
    let end = review.body.rfind(&marker(&id)).unwrap();
    assert!(
        review.body[end..].ends_with(&marker(&id)),
        "{}",
        review.body
    );
}

#[tokio::test]
async fn body_includes_limitation_before_improvements() {
    let (_tmp, _store, review) = publish(&shaped(true), "COMMENT").await;
    let limit_at = review.body.find("Limitation: missing CI").unwrap();
    let heading_at = review.body.find("Future improvements").unwrap();
    assert!(limit_at < heading_at, "{}", review.body);
    assert!(review.comments.is_empty(), "{:?}", review.comments);
}
