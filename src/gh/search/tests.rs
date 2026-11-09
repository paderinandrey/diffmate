mod malformed;
mod view;

use super::parse::parse_page;
use crate::error::Error;
use serde_json::{Value, json};

pub(super) fn pull(
    number: u64,
    name: &str,
    owner: &str,
    repo: &str,
    draft: bool,
    state: &str,
) -> Value {
    json!({
        "number": number,
        "repository": {"nameWithOwner": name},
        "url": format!("https://github.com/{owner}/{repo}/pull/{number}"),
        "isDraft": draft,
        "state": state,
    })
}

pub(super) fn page(values: Vec<Value>) -> super::parse::Page {
    parse_page(&serde_json::to_vec(&Value::Array(values)).unwrap()).unwrap()
}

#[test]
fn a_draft_and_a_ready_pull_request_are_kept() {
    let found = page(vec![
        pull(1, "acme/repo", "acme", "repo", true, "OPEN"),
        pull(2, "ACME/REPO", "acme", "repo", false, "Open"),
    ]);
    assert_eq!(found.len, 2);
    assert!(found.kept[0].is_draft);
    assert_eq!(found.kept[0].pr.number, 1);
    assert!(!found.kept[1].is_draft);
    assert_eq!(found.kept[1].pr.number, 2);
    assert_eq!(found.kept[1].pr.owner, "acme");
    assert_eq!(found.kept[1].pr.repo, "repo");
}

#[test]
fn a_team_url_is_dropped_and_still_counted() {
    let team = json!({
        "number": 1,
        "repository": {"nameWithOwner": "acme/repo"},
        "url": "https://github.com/orgs/acme/teams/reviewers",
        "isDraft": false,
        "state": "OPEN",
    });
    let found = page(vec![
        team,
        pull(2, "acme/repo", "acme", "repo", false, "OPEN"),
    ]);
    assert_eq!(found.len, 2);
    assert_eq!(found.kept.len(), 1);
    assert_eq!(found.kept[0].pr.number, 2);
}

#[test]
fn a_repository_that_disagrees_with_the_url_is_dropped() {
    let found = page(vec![pull(1, "other/repo", "acme", "repo", false, "OPEN")]);
    assert_eq!(found.len, 1);
    assert_eq!(found.kept.len(), 0);
}

#[test]
fn a_number_that_disagrees_with_the_url_is_dropped() {
    let bad = json!({
        "number": 9,
        "repository": {"nameWithOwner": "acme/repo"},
        "url": "https://github.com/acme/repo/pull/1",
        "isDraft": false,
        "state": "OPEN",
    });
    assert_eq!(page(vec![bad]).kept.len(), 0);
}

#[test]
fn a_closed_pull_request_is_dropped() {
    let found = page(vec![pull(1, "acme/repo", "acme", "repo", false, "CLOSED")]);
    assert_eq!(found.kept.len(), 0);
}

#[test]
fn invalid_json_and_a_non_array_are_errors() {
    assert!(matches!(parse_page(b"not json"), Err(Error::Json(_))));
    assert!(matches!(parse_page(b"{}"), Err(Error::Json(_))));
}
