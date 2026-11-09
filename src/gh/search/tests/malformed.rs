use super::{page, pull};
use serde_json::json;

#[test]
fn malformed_elements_are_skipped_and_still_counted() {
    let missing_draft = json!({
        "number": 1,
        "repository": {"nameWithOwner": "acme/repo"},
        "url": "https://github.com/acme/repo/pull/1",
        "state": "OPEN",
    });
    let bad_draft = json!({
        "number": 2,
        "repository": {"nameWithOwner": "acme/repo"},
        "url": "https://github.com/acme/repo/pull/2",
        "isDraft": "yes",
        "state": "OPEN",
    });
    let missing_url = json!({
        "number": 3,
        "repository": {"nameWithOwner": "acme/repo"},
        "isDraft": false,
        "state": "OPEN",
    });
    let bad_repo = json!({
        "number": 5,
        "repository": "acme/repo",
        "url": "https://github.com/acme/repo/pull/5",
        "isDraft": false,
        "state": "OPEN",
    });
    let good = pull(4, "acme/repo", "acme", "repo", false, "OPEN");
    let found = page(vec![missing_draft, bad_draft, missing_url, bad_repo, good]);
    assert_eq!(found.len, 5);
    assert_eq!(found.kept.len(), 1);
    assert_eq!(found.kept[0].pr.number, 4);
}
