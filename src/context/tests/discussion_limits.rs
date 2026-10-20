use super::discussion_cases::{empty, gh, note};
use super::fake_gh::{FakeGh, pr};
use crate::context::discussion::text;
use crate::gh::Discussion;

#[tokio::test]
async fn an_oversized_discussion_gets_the_size_marker_first() {
    let big = "word ".repeat(60_000);
    let found = Discussion {
        comments: vec![note("a", &big)],
        ..empty()
    };

    let text = text(&gh(found), &pr()).await;

    assert!(
        text.starts_with("TRUNCATED: discussion.md kept "),
        "{}",
        &text[..80]
    );
}

#[tokio::test]
async fn a_failed_call_is_stated_with_a_bounded_single_line_reason() {
    let text = text(&FakeGh::default(), &pr()).await;

    assert_eq!(
        text,
        "Discussion unavailable: gh api: boom [31m second line\n"
    );
}
