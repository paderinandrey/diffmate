use super::support::{load_live_prs, review_url};
use tokio::process::Command;

#[tokio::test]
#[ignore = "needs the real claude CLI, gh login and the owner's fixture PRs; makes one model call"]
async fn live_clean_pr_has_no_findings() {
    let url = load_live_prs()["clean"]["url"]
        .as_str()
        .unwrap()
        .to_string();

    let run = review_url(&url).await;

    let shown = run.describe();
    assert_eq!(run.state, "succeeded", "{shown}");
    assert_eq!(run.result.as_deref(), Some("no_findings"), "{shown}");
}

#[tokio::test]
#[ignore = "needs the real claude CLI, gh login and the owner's fixture PRs; makes one model call"]
async fn live_obvious_bug_is_anchored_to_line_7() {
    let prs = load_live_prs();
    let url = prs["obvious_bug"]["url"].as_str().unwrap();

    let run = review_url(url).await;

    let shown = run.describe();
    assert_eq!(run.state, "succeeded", "{shown}");
    assert_eq!(run.result.as_deref(), Some("findings"), "{shown}");
    assert!(
        run.has_finding_at(&prs["obvious_bug"]["expected_anchor"]),
        "{shown}"
    );
}

#[tokio::test]
#[ignore = "needs the real claude CLI, gh login and the owner's fixture PRs; makes one model call"]
async fn live_fork_pr_is_reviewed_the_same_way() {
    let prs = load_live_prs();
    let url = prs["fork"]["url"].as_str().unwrap();

    let run = review_url(url).await;

    let shown = run.describe();
    assert_eq!(run.state, "succeeded", "{shown}");
    assert!(
        run.has_finding_at(&prs["fork"]["expected_anchor"]),
        "{shown}"
    );
    let args = [
        "pr",
        "view",
        url,
        "--json",
        "headRefOid",
        "--jq",
        ".headRefOid",
    ];
    let output = Command::new("gh").args(args).output().await.unwrap();
    assert!(output.status.success());
    let head = String::from_utf8(output.stdout).unwrap();
    assert_eq!(run.head_sha, head.trim(), "{shown}");
}
