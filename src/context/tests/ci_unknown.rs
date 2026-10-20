use super::ci_cases::{fake, run};
use super::fake_gh::{FakeGh, pr};
use crate::context::ci::text;

#[tokio::test]
async fn no_check_runs_and_a_failed_call_both_read_as_unknown() {
    let none = text(&fake(vec![], 0), &pr(), "abc").await;
    let failed = text(&FakeGh::default(), &pr(), "abc").await;

    assert_eq!(
        none,
        "No check runs were reported for abc. CI status unknown.\n"
    );
    assert_eq!(failed, "CI status unknown: gh api: boom [31m second line\n");
}

#[tokio::test]
async fn a_failed_annotation_call_is_stated_in_the_file() {
    let gh = FakeGh {
        annotations_fail: true,
        ..fake(vec![run(1, "t", "completed", Some("failure"))], 1)
    };

    let ci = text(&gh, &pr(), "abc").await;

    assert!(
        ci.contains("Annotations unavailable: gh api: boom [31m second line"),
        "{ci}"
    );
}
