use super::support::{load_live_prs, review_url};

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
