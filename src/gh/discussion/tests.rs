use super::raw::parse;
use crate::error::Error;

fn page(nodes: &str, more: bool) -> String {
    format!("{{\"pageInfo\":{{\"hasNextPage\":{more}}},\"nodes\":[{nodes}]}}")
}

fn response(comments: &str, threads: &str, more: bool) -> String {
    let (reviews, none) = (page("", false), page("", false));
    let (comments, threads) = (page(comments, more), page(threads, false));
    let pull = format!(
        "\"comments\":{comments},\"reviews\":{reviews},\"reviewThreads\":{threads},\"x\":{none}"
    );
    format!("{{\"data\":{{\"repository\":{{\"pullRequest\":{{{pull}}}}}}}}}")
}

const COMMENT: &str = "{\"author\":{\"login\":\"alice\"},\"body\":\"Check this.\",\"createdAt\":\"2026-01-01T00:00:00Z\"}";
const GHOST: &str = "{\"author\":null,\"body\":\"Gone.\",\"createdAt\":\"2026-01-02T00:00:00Z\"}";

#[test]
fn comments_threads_and_missing_authors_are_read() {
    let thread_comments = page(COMMENT, false);
    let thread = format!(
        "null,{{\"isResolved\":true,\"isOutdated\":true,\"path\":\"a.rs\",\"line\":null,\"comments\":{thread_comments}}}"
    );
    let json = response(&format!("{COMMENT},null,{GHOST}"), &thread, false);

    let discussion = parse(json.as_bytes()).unwrap();

    let authors: Vec<_> = discussion
        .comments
        .iter()
        .map(|c| c.author.as_str())
        .collect();
    assert_eq!(authors, ["alice", "ghost"]);
    assert_eq!(discussion.threads.len(), 1);
    let thread = &discussion.threads[0];
    assert!(thread.resolved && thread.outdated && thread.line.is_none());
    assert_eq!(thread.path.as_deref(), Some("a.rs"));
    assert_eq!(thread.comments[0].body, "Check this.");
    assert!(!discussion.truncated);
}

#[test]
fn another_page_marks_the_discussion_truncated() {
    let discussion = parse(response(COMMENT, "", true).as_bytes()).unwrap();

    assert!(discussion.truncated);
}

#[test]
fn a_response_without_a_pull_request_is_an_error() {
    let json = "{\"data\":{\"repository\":{\"pullRequest\":null}}}";

    assert!(matches!(parse(json.as_bytes()), Err(Error::Command { .. })));
    assert!(matches!(parse(b"not json"), Err(Error::Json(_))));
}
