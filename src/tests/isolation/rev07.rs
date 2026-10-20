use super::endpoint::FakeAnthropic;
use super::fixture::MaliciousPr;
use super::harness::{Mode, observe};
use super::inspect::{assert_reached, bodies_contain};

#[tokio::test]
#[ignore = "needs the real claude CLI"]
async fn rev07_outside_import_canary_is_not_leaked() {
    let secondary = FakeAnthropic::start().await;
    let pr = MaliciousPr::build(&secondary.url);

    let control = observe(&pr, &secondary, Mode::Control).await;
    assert_reached(&control, "control");
    assert!(
        bodies_contain(&control, pr.canary("import")),
        "control vacuous: import"
    );

    let isolated = observe(&pr, &secondary, Mode::Isolated).await;
    assert_reached(&isolated, "isolated");
    assert!(!bodies_contain(&isolated, pr.canary("import")));
    assert!(!bodies_contain(&isolated, pr.canary("claude_md")));
}
