use super::endpoint::{FakeAnthropic, Recorded};
use super::fixture::MaliciousPr;
use super::harness::{EXPECTED_TOOLS, Mode, observe};
use super::inspect::{assert_reached, bodies_contain, tool_names};

fn vacuous(seen: &[Recorded], pr: &MaliciousPr, canary: &str) {
    assert_reached(seen, "control");
    assert!(
        bodies_contain(seen, pr.canary(canary)),
        "control vacuous: {canary}"
    );
}

#[tokio::test]
#[ignore = "needs the real claude CLI"]
async fn rev06_worktree_claude_md_canary_is_not_loaded() {
    let secondary = FakeAnthropic::start().await;
    let pr = MaliciousPr::build(&secondary.url);

    let control = observe(&pr, &secondary, Mode::Control).await;
    vacuous(&control, &pr, "claude_md");
    vacuous(&control, &pr, "context_root_claude_md");
    pr.set_claude_md(false);
    let agents_control = observe(&pr, &secondary, Mode::Control).await;
    vacuous(&agents_control, &pr, "agents_md");
    let agents_isolated = observe(&pr, &secondary, Mode::Isolated).await;
    pr.set_claude_md(true);
    assert_reached(&agents_isolated, "isolated without CLAUDE.md");
    assert!(!bodies_contain(&agents_isolated, pr.canary("agents_md")));

    let isolated = observe(&pr, &secondary, Mode::Isolated).await;
    assert_reached(&isolated, "isolated");
    for canary in [
        "claude_md",
        "agents_md",
        "context_root_claude_md",
        "context_rules_claude_md",
    ] {
        assert!(
            !bodies_contain(&isolated, pr.canary(canary)),
            "{canary} leaked"
        );
    }
    let expected = EXPECTED_TOOLS.iter().map(ToString::to_string).collect();
    assert_eq!(tool_names(&isolated), expected);
    assert!(bodies_contain(&isolated, "You review one pull request."));
}
