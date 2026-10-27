use super::fake_gh::{FakeGh, pr};
use crate::context::discussion::text;
use crate::gh::{Comment, Discussion, Review, Thread};

pub(super) fn note(author: &str, body: &str) -> Comment {
    Comment {
        author: author.into(),
        body: body.into(),
        created_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn thread(resolved: bool, outdated: bool, line: Option<u32>) -> Thread {
    Thread {
        resolved,
        outdated,
        path: Some("src/lib.rs".into()),
        line,
        comments: vec![note("bob", "Fixed.")],
    }
}

pub(super) fn gh(discussion: Discussion) -> FakeGh {
    FakeGh {
        discussion: Some(discussion).into(),
        ..FakeGh::default()
    }
}

pub(super) fn empty() -> Discussion {
    Discussion {
        comments: vec![],
        reviews: vec![],
        threads: vec![],
        truncated: false,
    }
}

#[tokio::test]
async fn comments_reviews_and_threads_are_listed_with_their_state() {
    let review = Review {
        author: "carol".into(),
        state: "CHANGES_REQUESTED".into(),
        body: "No.".into(),
        submitted_at: "2026-01-02".into(),
    };
    let found = Discussion {
        comments: vec![note("alice", "Look at the bound.\n")],
        reviews: vec![review],
        threads: vec![thread(true, false, Some(3)), thread(false, true, None)],
        truncated: false,
    };

    let text = text(&gh(found), &pr()).await;

    assert!(text.starts_with("# Discussion\n\n## Comments\n\n### alice at 2026-01-01T00:00:00Z\n\nLook at the bound.\n"), "{text}");
    assert!(
        text.contains("### carol [CHANGES_REQUESTED] at 2026-01-02\n\nNo.\n"),
        "{text}"
    );
    assert!(
        text.contains("### src/lib.rs:3 [resolved]\n\n### bob at"),
        "{text}"
    );
    assert!(
        text.contains("### src/lib.rs [unresolved] (outdated)\n"),
        "{text}"
    );
}

#[tokio::test]
async fn an_empty_discussion_says_so_and_a_further_page_is_marked() {
    let plain = text(&gh(empty()), &pr()).await;
    let more = text(
        &gh(Discussion {
            truncated: true,
            ..empty()
        }),
        &pr(),
    )
    .await;

    assert_eq!(plain.matches("None.\n").count(), 3);
    assert!(!plain.contains("TRUNCATED"));
    assert!(
        more.starts_with("TRUNCATED: discussion.md lists only the first 100 comments"),
        "{more}"
    );
}
