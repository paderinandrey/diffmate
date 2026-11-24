use super::support::{load_live_prs, review_fixture};

#[tokio::test]
#[ignore = "needs the real grok CLI, gh login and the owner's fixture PRs; makes one model call"]
async fn live_clean_pr_has_no_findings() {
    let run = review_fixture(&load_live_prs()["clean"]).await;

    let shown = run.describe();
    assert_eq!(run.state, "succeeded", "{shown}");
    assert_eq!(run.result.as_deref(), Some("no_findings"), "{shown}");
}

#[tokio::test]
#[ignore = "needs the real grok CLI, gh login and the owner's fixture PRs; makes one model call"]
async fn live_obvious_bug_is_anchored_to_line_7() {
    let prs = load_live_prs();
    let run = review_fixture(&prs["obvious_bug"]).await;

    let shown = run.describe();
    assert_eq!(run.state, "succeeded", "{shown}");
    assert_eq!(run.result.as_deref(), Some("findings"), "{shown}");
    assert!(
        run.has_finding_at(&prs["obvious_bug"]["expected_anchor"]),
        "{shown}"
    );
}

#[tokio::test]
#[ignore = "needs the real grok CLI, gh login and the owner's fixture PRs; makes one model call"]
async fn live_fork_pr_is_reviewed_the_same_way() {
    let prs = load_live_prs();
    let run = review_fixture(&prs["fork"]).await;

    let shown = run.describe();
    assert_eq!(run.state, "succeeded", "{shown}");
    assert!(
        run.has_finding_at(&prs["fork"]["expected_anchor"]),
        "{shown}"
    );
}
