use super::contract::{contract, control_fired};
use super::endpoint::FakeAnthropic;
use super::fixture::MaliciousPr;
use super::harness::{EXPECTED_TOOLS, Mode, observe};
use super::inspect::{assert_reached, bodies_contain, tool_names};

const MARKERS: [&str; 4] = [
    "session_start_hook",
    "api_key_helper",
    "plugin_hook",
    "mcp_server",
];

fn assert_git_calls_are_isolated(pr: &MaliciousPr) {
    if !contract()["claude_git_calls"]["runs_git"]
        .as_bool()
        .unwrap()
    {
        return;
    }
    let log = std::fs::read_to_string(&pr.git_log).unwrap_or_default();
    let lines: Vec<&str> = log.lines().collect();
    assert!(!lines.is_empty(), "no CLI git call was logged");
    let prefix = "GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 argv:";
    for line in lines {
        assert!(line.starts_with(prefix), "{line}");
    }
}

#[tokio::test]
#[ignore = "needs the real claude CLI"]
async fn rev08_hooks_plugins_and_mcp_do_not_run_or_load() {
    let secondary = FakeAnthropic::start().await;
    let pr = MaliciousPr::build(&secondary.url);

    let control = observe(&pr, &secondary, Mode::Control).await;
    assert_reached(&control, "control");
    let fired = pr.fired();
    for marker in MARKERS {
        assert!(
            !control_fired(marker) || fired.contains(&marker),
            "control vacuous: {marker}"
        );
    }
    let redirected = secondary.connections();
    assert!(
        !control_fired("settings_env_base_url") || redirected > 0,
        "control vacuous: settings env"
    );
    let open = observe(&pr, &secondary, Mode::OpenControl).await;
    assert_reached(&open, "open control");
    let canary = pr.canary("agents_commands_skills");
    assert!(
        bodies_contain(&open, canary),
        "control vacuous: agents, commands and skills"
    );

    let before = secondary.connections();

    let isolated = observe(&pr, &secondary, Mode::Isolated).await;
    assert_reached(&isolated, "isolated");
    assert_eq!(pr.fired(), Vec::<&str>::new());
    assert_eq!(secondary.connections(), before);
    let tools = tool_names(&isolated);
    assert!(tools.iter().all(|name| !name.starts_with("mcp__")));
    assert_eq!(
        tools,
        EXPECTED_TOOLS.iter().map(ToString::to_string).collect()
    );
    for (name, value) in &pr.canaries {
        assert!(!bodies_contain(&isolated, value), "{name} leaked");
    }
    assert_git_calls_are_isolated(&pr);
}
